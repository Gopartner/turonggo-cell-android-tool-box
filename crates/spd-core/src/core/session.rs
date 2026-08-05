//! Sesi BROM/FDL tingkat tinggi.
//!
//! Menggabungkan [`Transport`](crate::Transport) + framing + perintah
//! protokol menjadi operasi yang bermakna: handshake, kirim FDL,
//! list partisi, dump partisi, tulis partisi.

use std::io::Write;

use crate::error::Result;
use crate::proto::brom::{self, Partition};
use crate::proto::framing::{self, Message, MessageDecoder};
use crate::transport::Transport;

/// Default ukuran potongan pembacaan partisi.
pub const DEFAULT_BLK_SIZE: usize = 0xFF00;
/// Ukuran potongan saat mengirim FDL (tahap awal).
pub const FDL_STEP: usize = 528;

/// Flags framing aktif untuk sesi ini.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    pub crc16: bool,
    pub transcode: bool,
}

impl Flags {
    /// Fase BROM: crc16 + transcode.
    pub fn brom() -> Self {
        Self {
            crc16: true,
            transcode: true,
        }
    }
    /// Fase FDL1/FDL2: sum + transcode.
    pub fn fdl() -> Self {
        Self {
            crc16: false,
            transcode: true,
        }
    }
}

/// Sesi ke perangkat Unisoc dalam mode BROM/download.
pub struct Session {
    transport: Box<dyn Transport>,
    decoder: MessageDecoder,
    pub flags: Flags,
    pub timeout_ms: u64,
    pub verbose: bool,
}

impl Session {
    pub fn new(transport: Box<dyn Transport>, flags: Flags) -> Self {
        let decoder = MessageDecoder::new(flags.crc16, flags.transcode);
        Self {
            transport,
            decoder,
            flags,
            timeout_ms: 1000,
            verbose: false,
        }
    }

    fn set_flags(&mut self, flags: Flags) {
        self.flags = flags;
        self.decoder = MessageDecoder::new(flags.crc16, flags.transcode);
    }

    /// Kirim pesan ter-encode. `check_baud=true` mengirim barisan 0x7E polos.
    pub fn send(&mut self, msg_type: u16, payload: &[u8]) -> Result<()> {
        let raw =
            framing::encode_message(msg_type, payload, self.flags.crc16, self.flags.transcode);
        self.transport.write(&raw).map_err(crate::Error::from)?;
        if self.verbose {
            eprintln!("send: type=0x{msg_type:04x}, len={}", payload.len());
        }
        Ok(())
    }

    /// Kirim CHECK_BAUD (deretan 0x7E).
    fn send_check_baud(&mut self, len: usize) -> Result<()> {
        let raw = vec![framing::HDLC_HEADER; len];
        self.transport.write(&raw).map_err(crate::Error::from)?;
        Ok(())
    }

    /// Baca satu pesan penuh.
    pub fn recv(&mut self) -> Result<Message> {
        let mut buf = [0u8; 0x8000];
        loop {
            let n = self
                .transport
                .read(&mut buf, self.timeout_ms)
                .map_err(crate::Error::from)?;
            if n == 0 {
                return Err(crate::protocol_err!("timeout reached"));
            }
            let mut msgs = self.decoder.feed(&buf[..n])?;
            if !msgs.is_empty() {
                let msg = msgs.remove(0);
                if self.verbose {
                    eprintln!(
                        "recv: type=0x{:04x}, len={}",
                        msg.msg_type,
                        msg.payload.len()
                    );
                }
                return Ok(msg);
            }
        }
    }

    /// Kirim lalu tunggu `BSL_REP_ACK`.
    pub fn send_and_check(&mut self, msg_type: u16, payload: &[u8]) -> Result<()> {
        self.send(msg_type, payload)?;
        let msg = self.recv()?;
        if msg.msg_type != brom::BSL_REP_ACK {
            return Err(crate::protocol_err!(
                "unexpected response (0x{:04x})",
                msg.msg_type
            ));
        }
        Ok(())
    }

    /// Handshake BROM: CHECK_BAUD → VER, lalu CONNECT → ACK.
    pub fn connect(&mut self) -> Result<String> {
        self.set_flags(Flags::brom());
        self.send_check_baud(1)?;
        let ver = self.recv()?;
        if ver.msg_type != brom::BSL_REP_VER {
            return Err(crate::protocol_err!(
                "wrong command or wrong mode detected (0x{:04x})",
                ver.msg_type
            ));
        }
        let ver_str = String::from_utf8_lossy(&ver.payload).to_string();
        self.send_and_check(brom::BSL_CMD_CONNECT, &[])?;
        Ok(ver_str)
    }

    /// Kirim file FDL ke alamat RAM lalu eksekusi.
    pub fn send_fdl(&mut self, data: &[u8], addr: u32, step: usize) -> Result<()> {
        let mut hdr = Vec::with_capacity(8);
        hdr.extend_from_slice(&addr.to_be_bytes());
        hdr.extend_from_slice(&(data.len() as u32).to_be_bytes());
        self.send_and_check(brom::BSL_CMD_START_DATA, &hdr)?;

        let mut off = 0;
        while off < data.len() {
            let n = (data.len() - off).min(step);
            self.send_and_check(brom::BSL_CMD_MIDST_DATA, &data[off..off + n])?;
            off += n;
        }
        self.send_and_check(brom::BSL_CMD_END_DATA, &[])?;
        Ok(())
    }

    /// Transisi ke FDL1: set flags sum, CHECK_BAUD → VER (dengan retry), CONNECT.
    pub fn enter_fdl1(&mut self) -> Result<String> {
        self.set_flags(Flags::fdl());
        let mut ver = None;
        for _ in 0..3 {
            self.send_check_baud(1)?;
            let msg = self.recv()?;
            if msg.msg_type == brom::BSL_REP_VER {
                ver = Some(String::from_utf8_lossy(&msg.payload).to_string());
                break;
            }
        }
        let ver = ver.ok_or_else(|| crate::protocol_err!("CHECK_BAUD FDL1 failed"))?;
        self.send_and_check(brom::BSL_CMD_CONNECT, &[])?;
        Ok(ver)
    }

    /// Transisi ke FDL2 (eksekusi setelah FDL2 dikirim).
    pub fn exec_fdl2(&mut self) -> Result<()> {
        self.send(brom::BSL_CMD_EXEC_DATA, &[])?;
        let msg = self.recv()?;
        match msg.msg_type {
            brom::BSL_REP_ACK => Ok(()),
            brom::BSL_REP_INCOMPATIBLE_PARTITION => Ok(()),
            other => Err(crate::protocol_err!(
                "unexpected response after FDL2 exec (0x{other:04x})"
            )),
        }
    }

    /// Alur lengkap masuk ke FDL2 (opsional: custom exec di tahap awal).
    pub fn boot(
        &mut self,
        fdl1: &[u8],
        fdl1_addr: u32,
        fdl2: &[u8],
        fdl2_addr: u32,
        custom_exec: Option<(&[u8], u32)>,
    ) -> Result<String> {
        let ver_brom = self.connect()?;
        self.send_fdl(fdl1, fdl1_addr, FDL_STEP)?;
        if let Some((exec, addr)) = custom_exec {
            self.send_fdl(exec, addr, FDL_STEP)?;
        }
        self.send_and_check(brom::BSL_CMD_EXEC_DATA, &[])?;
        let _ = self.enter_fdl1()?;
        self.send_fdl(fdl2, fdl2_addr, FDL_STEP)?;
        self.exec_fdl2()?;
        Ok(ver_brom)
    }

    /// Ambil daftar partisi lewat `READ_PARTITION`.
    pub fn partition_list(&mut self) -> Result<Vec<Partition>> {
        self.send(brom::BSL_CMD_READ_PARTITION, &[])?;
        let msg = self.recv()?;
        if msg.msg_type != brom::BSL_REP_READ_PARTITION {
            return Err(crate::protocol_err!(
                "unexpected response (0x{:04x})",
                msg.msg_type
            ));
        }
        brom::parse_partition_table(&msg.payload)
    }

    /// Dump partisi ke `writer`. Mengembalikan jumlah byte yang dibaca.
    pub fn dump_partition<W: Write>(
        &mut self,
        name: &str,
        start: u64,
        len: u64,
        step: usize,
        writer: &mut W,
    ) -> Result<u64> {
        // kasus khusus nv: tanya panjang sebenarnya lewat partisi *_2
        let mut len = len;
        if name.contains("fixnv") || name.contains("runtimenv") {
            let name2 = {
                let mut s = name.to_string();
                if let Some(pos) = s.rfind('1') {
                    s.replace_range(pos..pos + 1, "2");
                }
                s
            };
            let sel = brom::select_partition(&name2, 8, false);
            self.send_and_check(brom::BSL_CMD_READ_START, &sel)?;
            let mut data = Vec::new();
            data.extend_from_slice(&8u32.to_le_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
            self.send(brom::BSL_CMD_READ_MIDST, &data)?;
            let msg = self.recv()?;
            if msg.msg_type != brom::BSL_REP_READ_FLASH {
                return Err(crate::protocol_err!(
                    "unexpected response while probing nv length (0x{:04x})",
                    msg.msg_type
                ));
            }
            if msg.payload.len() < 12 {
                return Err(crate::protocol_err!("nv probe payload too short"));
            }
            let nv_len = u32::from_le_bytes(msg.payload[8..12].try_into().unwrap()) as u64;
            len = 0x200 + nv_len;
            self.send_and_check(brom::BSL_CMD_READ_END, &[])?;
        }

        let mode64 = (start + len) >> 32 != 0;
        let sel = brom::select_partition(name, start + len, mode64);
        self.send_and_check(brom::BSL_CMD_READ_START, &sel)?;

        let mut offset = start;
        while offset < start + len {
            let n = ((start + len - offset) as usize).min(step);
            let mut data = Vec::with_capacity(if mode64 { 12 } else { 8 });
            data.extend_from_slice(&(n as u32).to_le_bytes());
            data.extend_from_slice(&(offset as u32).to_le_bytes());
            if mode64 {
                data.extend_from_slice(&((offset >> 32) as u32).to_le_bytes());
            }
            self.send(brom::BSL_CMD_READ_MIDST, &data)?;
            let msg = self.recv()?;
            if msg.msg_type != brom::BSL_REP_READ_FLASH {
                return Err(crate::protocol_err!(
                    "unexpected response while reading (0x{:04x})",
                    msg.msg_type
                ));
            }
            let nread = msg.payload.len();
            if n < nread {
                return Err(crate::protocol_err!("unexpected length ({nread} > {n})"));
            }
            writer.write_all(&msg.payload)?;
            offset += nread as u64;
            if n != nread {
                break;
            }
        }
        self.send_and_check(brom::BSL_CMD_READ_END, &[])?;
        Ok(offset - start)
    }

    /// Dump seluruh daftar partisi (dari `partition_list`) ke direktori.
    pub fn dump_all<W: Write>(
        &mut self,
        partitions: &[Partition],
        mut open: impl FnMut(&str) -> Result<W>,
    ) -> Result<()> {
        for p in partitions {
            let mut w = open(&p.name)?;
            let read = self.dump_partition(&p.name, 0, p.size, DEFAULT_BLK_SIZE, &mut w)?;
            w.flush()?;
            if self.verbose {
                eprintln!("dump {}: {read} bytes", p.name);
            }
        }
        Ok(())
    }

    /// Tulis file ke partisi.
    pub fn write_partition(&mut self, name: &str, data: &[u8], step: usize) -> Result<()> {
        let mode64 = (data.len() as u64) >> 32 != 0;
        let sel = brom::select_partition(name, data.len() as u64, mode64);
        self.send_and_check(brom::BSL_CMD_START_DATA, &sel)?;

        let mut off = 0;
        while off < data.len() {
            let n = (data.len() - off).min(step);
            self.send_and_check(brom::BSL_CMD_MIDST_DATA, &data[off..off + n])?;
            off += n;
        }
        self.send_and_check(brom::BSL_CMD_END_DATA, &[])?;
        Ok(())
    }

    /// Reset perangkat ke mode normal.
    pub fn reset(&mut self) -> Result<()> {
        self.send_and_check(brom::BSL_CMD_NORMAL_RESET, &[])
    }

    /// Matikan perangkat.
    pub fn poweroff(&mut self) -> Result<()> {
        self.send_and_check(brom::BSL_CMD_POWER_OFF, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::{part, MockTransport};

    #[test]
    fn full_boot_with_mock() {
        let parts = vec![part("boot", 64), part("nv", 2)];
        let t = Box::new(MockTransport::new(parts));
        let mut s = Session::new(t, Flags::brom());

        let fdl1 = vec![0xAA; 2048];
        let fdl2 = vec![0xBB; 2048];
        let ver = s.boot(&fdl1, 0x65000800, &fdl2, 0x9EFFFE00, None).unwrap();
        assert!(!ver.is_empty());
    }

    #[test]
    fn partition_list_from_mock() {
        let t = Box::new(MockTransport::fdl(vec![
            part("boot", 64),
            part("nv", 2),
            part("super", 8000),
        ]));
        let mut s = Session::new(t, Flags::fdl());
        let parts = s.partition_list().unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].name, "boot");
        // divisor akhir 1 (dari nv=2) → (2 << 20) >> 1 = 1 MB
        assert_eq!(parts[1].size, 1 << 20);
    }

    #[test]
    fn dump_partition_from_mock() {
        let t = Box::new(MockTransport::fdl(vec![part("boot", 64)]));
        let mut s = Session::new(t, Flags::fdl());

        let sel = brom::select_partition("boot", 64 << 20, false);
        s.send_and_check(brom::BSL_CMD_READ_START, &sel).unwrap();

        let mut out = Vec::new();
        let read = s
            .dump_partition("boot", 0, 64 << 20, 0x1000, &mut out)
            .unwrap();
        assert!(read > 0);
        assert_eq!(out.len() as u64, read);
        // data mock berpola offset: byte pertama = 0
        assert_eq!(out[0], 0);
        assert_eq!(out[1], 1);
    }

    #[test]
    fn write_partition_to_mock() {
        let t = Box::new(MockTransport::fdl(vec![part("boot", 64)]));
        let mut s = Session::new(t, Flags::fdl());
        let data = vec![0x55; 4096];
        s.write_partition("boot", &data, 0x1000).unwrap();
    }
}
