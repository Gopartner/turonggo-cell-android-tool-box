//! Transport tiruan (mock) untuk pengujian tanpa hardware.
//!
//! `MockTransport` menjawab alur BROM/FDL dasar sehingga test bisa
//! menjalankan: connect → kirim FDL → list partisi → dump partisi,
//! tanpa memerlukan HP benar-benar tersambung.

use std::collections::VecDeque;

use super::{Transport, TransportError};
use crate::proto::brom::{self, name_from_utf16le, Partition};
use crate::proto::framing::{self, Message, MessageDecoder};

/// Skenario jawaban yang didukung mock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockStage {
    /// Fase BROM (checksum = crc16, transcode aktif).
    Brom,
    /// Fase FDL1/FDL2 (checksum = sum, transcode aktif).
    Fdl,
}

/// Device BROM/FDL tiruan yang berperilaku seperti HP di mode download.
pub struct MockTransport {
    /// Daftar partisi yang "dikenali" device.
    pub partitions: Vec<Partition>,
    /// Rekaman semua pesan yang diterima (didekode).
    pub received: Vec<Message>,
    /// Jawaban berantai yang sudah di-encode, dikirim saat `read`.
    outbox: VecDeque<u8>,
    /// Stage protokol saat ini.
    stage: MockStage,
    /// Ukuran FDL yang terakhir dipilih (misal 64MB), untuk read_midst.
    selected: Option<(String, u64)>,
}

impl MockTransport {
    pub fn new(partitions: Vec<Partition>) -> Self {
        Self {
            partitions,
            received: Vec::new(),
            outbox: VecDeque::new(),
            stage: MockStage::Brom,
            selected: None,
        }
    }

    /// Mock yang langsung berada di fase FDL (untuk test list/dump/write).
    pub fn fdl(partitions: Vec<Partition>) -> Self {
        Self {
            partitions,
            received: Vec::new(),
            outbox: VecDeque::new(),
            stage: MockStage::Fdl,
            selected: None,
        }
    }

    /// Jawaban ACK sederhana.
    fn reply_ack(&mut self) {
        self.reply(brom::BSL_REP_ACK, &[]);
    }

    fn reply_ver(&mut self) {
        // panjang payload harus genap: checksum sum C meng-swap byte untuk
        // len genap di encode (final=1) & decode (final=2), tapi inkonsisten
        // untuk len ganjil → device asli hanya membalas pesan genap.
        let payload = b"FDL1 mock ver 1.0!".to_vec();
        self.reply(brom::BSL_REP_VER, &payload);
    }

    fn reply(&mut self, msg_type: u16, payload: &[u8]) {
        // crc16 hanya di fase BROM; setelah itu pakai sum (seperti C).
        let use_crc = self.stage == MockStage::Brom;
        let framed = framing::encode_message(msg_type, payload, use_crc, true);
        self.outbox.extend(framed);
    }

    fn handle(&mut self, msg: &Message) {
        match msg.msg_type {
            brom::BSL_CMD_CONNECT => self.reply_ack(),
            brom::BSL_CMD_CHECK_BAUD => self.reply_ver(),
            brom::BSL_CMD_EXEC_DATA => {
                // C meng-Ack exec masih di fase lama, barulah kemudian
                // protokol pindah ke fase FDL (checksum sum).
                self.reply_ack();
                self.stage = MockStage::Fdl;
            }
            brom::BSL_CMD_START_DATA
            | brom::BSL_CMD_MIDST_DATA
            | brom::BSL_CMD_END_DATA
            | brom::BSL_CMD_KEEP_CHARGE
            | brom::BSL_CMD_DISABLE_TRANSCODE
            | brom::BSL_CMD_NORMAL_RESET
            | brom::BSL_CMD_POWER_OFF => self.reply_ack(),
            brom::BSL_CMD_READ_START => {
                // payload = nama UTF-16LE (72B) + size LE (4/16B)
                let name = match name_from_utf16le(&msg.payload[..72]) {
                    Ok(n) => n,
                    Err(_) => return self.reply_ack(),
                };
                let size = if msg.payload.len() >= 76 {
                    u32::from_le_bytes(msg.payload[72..76].try_into().unwrap()) as u64
                } else {
                    0
                };
                self.selected = Some((name, size));
                self.reply_ack();
            }
            brom::BSL_CMD_READ_MIDST => {
                // payload = u32 len LE, u32 offset LE, (u32 offset_hi LE)
                if msg.payload.len() < 8 {
                    return self.reply_ack();
                }
                let len = u32::from_le_bytes(msg.payload[0..4].try_into().unwrap()) as usize;
                let offset = u32::from_le_bytes(msg.payload[4..8].try_into().unwrap()) as u64;
                let part_name = self.selected.as_ref().map(|s| s.0.clone());
                let data = match part_name {
                    Some(name) => {
                        // nol total + pola offset supaya bisa diverifikasi test
                        let max = self
                            .partitions
                            .iter()
                            .find(|p| p.name == name)
                            .map(|p| p.size)
                            .unwrap_or(len as u64);
                        let n = (len as u64).min(max.saturating_sub(offset)) as usize;
                        (0..n)
                            .map(|i| ((offset as usize + i) & 0xFF) as u8)
                            .collect::<Vec<_>>()
                    }
                    None => vec![0u8; len],
                };
                self.reply(brom::BSL_REP_READ_FLASH, &data);
            }
            brom::BSL_CMD_READ_END => self.reply_ack(),
            brom::BSL_CMD_READ_PARTITION => {
                // jawab tabel partisi (semua ukuran dalam unit ~MB-an)
                let mut table = Vec::new();
                for p in &self.partitions {
                    let mut e = vec![0u8; brom::PARTITION_ENTRY_SIZE];
                    let name = brom::name_to_utf16le(&p.name);
                    e[..name.len()].copy_from_slice(&name);
                    let mb = (p.size >> 20) as u32;
                    e[0x48..0x4C].copy_from_slice(&mb.to_le_bytes());
                    table.extend_from_slice(&e);
                }
                self.reply(brom::BSL_REP_READ_PARTITION, &table);
            }
            brom::BSL_CMD_READ_CHIP_UID => {
                self.reply(brom::BSL_REP_READ_CHIP_UID, b"mock-uid-1234");
            }
            _ => {
                // perintah lain dijawab ACK agar sesi tidak macet
                self.reply_ack();
            }
        }
    }
}

impl Transport for MockTransport {
    fn write(&mut self, data: &[u8]) -> Result<(), TransportError> {
        // BSL_CMD_CHECK_BAUD dikirim sebagai barisan 0x7E polos (bukan frame).
        let is_check_baud = data.iter().all(|&b| b == framing::HDLC_HEADER);
        let mut decoder = MessageDecoder::new(self.stage == MockStage::Brom, true);
        if is_check_baud {
            self.reply_ver();
            return Ok(());
        }
        let decoded = decoder
            .feed(data)
            .map_err(|e| TransportError::Read(e.to_string()))?;
        for msg in decoded {
            self.handle(&msg);
        }
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8], _timeout_ms: u64) -> Result<usize, TransportError> {
        let mut n = 0;
        while n < buf.len() {
            match self.outbox.pop_front() {
                Some(b) => {
                    buf[n] = b;
                    n += 1;
                }
                None => break,
            }
        }
        Ok(n)
    }
}

/// Pembuat `Partition` ringkas untuk test/mock.
pub fn part(name: &str, mb: u64) -> Partition {
    Partition {
        name: name.to_string(),
        size: mb << 20,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::brom::parse_partition_table;
    use crate::proto::framing::encode_message;

    #[test]
    fn mock_answers_connect_with_ack() {
        let mut t = MockTransport::new(vec![part("boot", 64)]);
        t.write(&encode_message(brom::BSL_CMD_CONNECT, &[], true, true))
            .unwrap();
        let mut buf = [0u8; 256];
        let n = t.read(&mut buf, 100).unwrap();
        assert!(n > 0);
        let mut dec = MessageDecoder::new(true, true);
        let msgs = dec.feed(&buf[..n]).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].msg_type, brom::BSL_REP_ACK);
    }

    #[test]
    fn mock_check_baud_returns_ver() {
        let mut t = MockTransport::new(vec![]);
        // CHECK_BAUD dikirim polos
        t.write(&[framing::HDLC_HEADER]).unwrap();
        let mut buf = [0u8; 256];
        let n = t.read(&mut buf, 100).unwrap();
        let mut dec = MessageDecoder::new(true, true);
        let msgs = dec.feed(&buf[..n]).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].msg_type, brom::BSL_REP_VER);
    }

    #[test]
    fn mock_read_partition_returns_table() {
        let mut t = MockTransport::new(vec![part("boot", 64), part("nv", 2)]);
        t.write(&encode_message(
            brom::BSL_CMD_READ_PARTITION,
            &[],
            true,
            true,
        ))
        .unwrap();
        let mut buf = [0u8; 1024];
        let n = t.read(&mut buf, 100).unwrap();
        let mut dec = MessageDecoder::new(true, true);
        let msgs = dec.feed(&buf[..n]).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].msg_type, brom::BSL_REP_READ_PARTITION);
        let parts = parse_partition_table(&msgs[0].payload).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name, "boot");
        assert_eq!(parts[1].name, "nv");
    }

    #[test]
    fn mock_select_and_read_midst_returns_data() {
        use crate::proto::brom::name_to_utf16le;

        let mut t = MockTransport::new(vec![part("nv", 1)]);
        let sel = name_to_utf16le("nv");
        let mut payload = sel;
        payload.extend_from_slice(&(1u32 << 20).to_le_bytes());
        t.write(&encode_message(
            brom::BSL_CMD_READ_START,
            &payload,
            true,
            true,
        ))
        .unwrap();

        // read_midst 16 byte dari offset 0
        let mut mp = Vec::new();
        mp.extend_from_slice(&16u32.to_le_bytes());
        mp.extend_from_slice(&0u32.to_le_bytes());
        t.write(&encode_message(brom::BSL_CMD_READ_MIDST, &mp, true, true))
            .unwrap();

        let mut buf = [0u8; 512];
        let mut dec = MessageDecoder::new(true, true);
        // mungkin butuh dua read (ack + data)
        let mut all = Vec::new();
        loop {
            let n = t.read(&mut buf, 100).unwrap();
            if n == 0 {
                break;
            }
            all.extend_from_slice(&buf[..n]);
        }
        let msgs = dec.feed(&all).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].msg_type, brom::BSL_REP_ACK);
        assert_eq!(msgs[1].msg_type, brom::BSL_REP_READ_FLASH);
        assert_eq!(msgs[1].payload.len(), 16);
        assert_eq!(msgs[1].payload[0], 0);
        assert_eq!(msgs[1].payload[1], 1);
    }

    #[test]
    fn mock_switches_to_sum_after_exec() {
        let mut t = MockTransport::new(vec![]);

        // CONNECT di fase BROM → dibalas pakai crc16
        t.write(&encode_message(brom::BSL_CMD_CONNECT, &[], true, true))
            .unwrap();
        let mut buf = [0u8; 256];
        let n = t.read(&mut buf, 100).unwrap();
        let mut dec = MessageDecoder::new(true, true);
        assert_eq!(dec.feed(&buf[..n]).unwrap()[0].msg_type, brom::BSL_REP_ACK);

        // EXEC masih dijawab crc16 (C meng-Ack exec sebelum pindah fase)
        t.write(&encode_message(brom::BSL_CMD_EXEC_DATA, &[], true, true))
            .unwrap();
        let n = t.read(&mut buf, 100).unwrap();
        let mut dec = MessageDecoder::new(true, true);
        assert_eq!(dec.feed(&buf[..n]).unwrap()[0].msg_type, brom::BSL_REP_ACK);

        // setelah exec, balasan memakai sum: decoder crc16 justru gagal
        t.write(&encode_message(brom::BSL_CMD_CONNECT, &[], false, true))
            .unwrap();
        let n = t.read(&mut buf, 100).unwrap();
        let mut dec = MessageDecoder::new(false, true);
        assert_eq!(dec.feed(&buf[..n]).unwrap()[0].msg_type, brom::BSL_REP_ACK);
        let mut dec2 = MessageDecoder::new(true, true);
        assert!(dec2.feed(&buf[..n]).is_err());
    }
}
