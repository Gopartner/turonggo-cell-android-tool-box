//! Framing protokol BROM/FDL — port setia dari `common.c` (spd_dump CE1CECL).
//!
//! Format pesan (sebelum escape):
//!   [type u16 BE][len u16 BE][payload len bytes][checksum u16 BE]
//! dibungkus: 0x7E ... 0x7E, dengan byte 0x7E/0x7D di-escape (transcode).

pub const HDLC_HEADER: u8 = 0x7e;
pub const HDLC_ESCAPE: u8 = 0x7d;

/// Checksum "final" untuk encode (port CHK_FIXZERO).
pub const CHK_FIXZERO: u8 = 1;
/// Checksum "final" untuk decode (port CHK_ORIG).
pub const CHK_ORIG: u8 = 2;

/// CRC16 yang dipakai BROM (polinom 0x11021, bitwise) — port `spd_crc16`.
pub fn crc16(mut crc: u32, data: &[u8]) -> u16 {
    for &b in data {
        crc ^= (b as u32) << 8;
        for _ in 0..8 {
            crc = (crc << 1) ^ (0u32.wrapping_sub(crc >> 15) & 0x11021);
        }
    }
    (crc & 0xffff) as u16
}

/// Checksum tipe "sum" yang dipakai FDL1 — port `spd_checksum`.
/// `fin` = 1 (encode) atau 2 (decode), mengikuti C.
pub fn checksum(mut crc: u32, data: &[u8], fin: u8) -> u16 {
    let mut i = 0;
    let mut len = data.len();
    while len > 1 {
        crc = crc.wrapping_add(((data[i + 1] as u32) << 8) | data[i] as u32);
        i += 2;
        len -= 2;
    }
    if len == 1 {
        crc = crc.wrapping_add(data[i] as u32);
    }
    if fin != 0 {
        crc = (crc >> 16) + (crc & 0xffff);
        crc += crc >> 16;
        crc = (!crc) & 0xffff;
        if (len as u8) < fin {
            crc = (crc >> 8) | ((crc & 0xff) << 8);
        }
    }
    crc as u16
}

/// Transcode/escape: 0x7E dan 0x7D didahului 0x7D, byte di-XOR 0x20.
/// Port `spd_transcode` (dst, src, len).
pub fn transcode(dst: &mut Vec<u8>, src: &[u8]) {
    for &a in src {
        if a == HDLC_HEADER || a == HDLC_ESCAPE {
            dst.push(HDLC_ESCAPE);
            dst.push(a ^ 0x20);
        } else {
            dst.push(a);
        }
    }
}

/// Pesan BROM yang sudah didekode.
#[derive(Debug, Clone)]
pub struct Message {
    pub msg_type: u16,
    pub payload: Vec<u8>,
}

/// Encode pesan menjadi byte siap kirim (termasuk STX/ETX dan escape).
/// Port `encode_msg`.
pub fn encode_message(
    msg_type: u16,
    payload: &[u8],
    use_crc: bool,
    use_transcode: bool,
) -> Vec<u8> {
    let mut raw = Vec::with_capacity(4 + payload.len() + 2);
    raw.extend_from_slice(&msg_type.to_be_bytes());
    raw.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    raw.extend_from_slice(payload);

    let chk = if use_crc {
        crc16(0, &raw)
    } else {
        checksum(0, &raw, CHK_FIXZERO)
    };
    raw.extend_from_slice(&chk.to_be_bytes());

    let mut out = Vec::with_capacity(raw.len() + 2);
    out.push(HDLC_HEADER);
    if use_transcode {
        transcode(&mut out, &raw);
    } else {
        out.extend_from_slice(&raw);
    }
    out.push(HDLC_HEADER);
    out
}

fn read_be16(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}

/// Dekoder pesan berbasis aliran byte — port `recv_msg_orig`.
///
/// `feed()` menerima byte mentah dari transport dan mengembalikan
/// pesan yang lengkap (bisa lebih dari satu, bisa nol).
#[derive(Debug)]
pub struct MessageDecoder {
    use_crc: bool,
    use_transcode: bool,
    raw: Vec<u8>,
    head: bool,
    esc: u8,
    plen: usize,
}

impl MessageDecoder {
    pub fn new(use_crc: bool, use_transcode: bool) -> Self {
        Self {
            use_crc,
            use_transcode,
            raw: Vec::with_capacity(256),
            head: false,
            esc: 0,
            plen: 6,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> crate::Result<Vec<Message>> {
        let mut messages = Vec::new();
        for &a in bytes {
            if let Some(m) = self.step(a)? {
                messages.push(m);
            }
        }
        Ok(messages)
    }

    fn step(&mut self, a: u8) -> crate::Result<Option<Message>> {
        if self.use_transcode {
            // validasi byte setelah escape (port recv_msg_orig: esc tetap
            // aktif sampai byte tersimpan, lihat `raw_buf[nread++] = a ^ esc`).
            if self.esc != 0 && a != (HDLC_HEADER ^ 0x20) && a != (HDLC_ESCAPE ^ 0x20) {
                return Err(crate::protocol_err!("unexpected escaped byte (0x{a:02x})"));
            }
            if a == HDLC_HEADER {
                if !self.head {
                    self.head = true;
                    return Ok(None);
                }
                if self.raw.is_empty() {
                    return Ok(None);
                }
                if self.raw.len() < self.plen {
                    return Err(crate::protocol_err!("message too short"));
                }
                return Ok(Some(self.finish()?));
            } else if a == HDLC_ESCAPE {
                self.esc = 0x20;
                return Ok(None);
            } else {
                if !self.head {
                    return Ok(None);
                }
                if self.raw.len() >= self.plen {
                    return Err(crate::protocol_err!("message too long"));
                }
                self.raw.push(a ^ self.esc);
                self.esc = 0;
            }
        } else {
            if !self.head && a == HDLC_HEADER {
                self.head = true;
                return Ok(None);
            }
            if self.raw.len() == self.plen {
                if a != HDLC_HEADER {
                    return Err(crate::protocol_err!("expected end of message"));
                }
                return Ok(Some(self.finish()?));
            }
            self.raw.push(a);
        }

        if self.raw.len() == 4 {
            self.plen = read_be16(&self.raw[2..4]) as usize + 6;
        }
        Ok(None)
    }

    fn finish(&mut self) -> crate::Result<Message> {
        let expected = self.plen;
        let raw = std::mem::take(&mut self.raw);
        self.head = false;
        self.plen = 6;
        self.esc = 0;

        if raw.len() < 6 {
            return Err(crate::protocol_err!("message too short"));
        }
        if raw.len() != expected {
            return Err(crate::protocol_err!(
                "bad length ({}, expected {})",
                raw.len(),
                expected
            ));
        }

        let body = &raw[..raw.len() - 2];
        let chk = if self.use_crc {
            crc16(0, body)
        } else {
            checksum(0, body, CHK_ORIG)
        };
        let got = read_be16(&raw[raw.len() - 2..]);
        if chk != got {
            return Err(crate::protocol_err!(
                "bad checksum (0x{got:04x}, expected 0x{chk:04x})"
            ));
        }

        let msg_type = read_be16(&raw[..2]);
        let len = read_be16(&raw[2..4]) as usize;
        if raw.len() < 4 + len {
            return Err(crate::protocol_err!("payload truncated"));
        }
        Ok(Message {
            msg_type,
            payload: raw[4..4 + len].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc16_matches_reference() {
        // Port dari C: crc16(0, {0x00,0x00,0x00,0x00}) harus deterministik.
        let c = crc16(0, &[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(c, 0x0000);
        let c = crc16(0, &[0xff; 8]);
        // divalidasi dengan simulasi bitwise implementasi C spd_crc16
        assert_eq!(c, 0xA6E1);
    }

    #[test]
    fn encode_roundtrip_with_transcode() {
        let raw = vec![0x7e, 0x00, 0x00, 0x02, 0x80, 0x7d, 0x00, 0x7e];
        let msg = Message {
            msg_type: 0x8100,
            payload: raw.clone(),
        };
        let framed = encode_message(msg.msg_type, &msg.payload, true, true);
        // harus di-escape: tidak boleh ada 0x7e di tengah (0x7d wajar
        // muncul sebagai prefix escape)
        let inner = &framed[1..framed.len() - 1];
        assert!(!inner.contains(&HDLC_HEADER));

        let mut dec = MessageDecoder::new(true, true);
        let out = dec.feed(&framed).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].msg_type, msg.msg_type);
        assert_eq!(out[0].payload, raw);
    }

    #[test]
    fn decode_splits_multiple_messages() {
        let a = encode_message(0x0000, &[], true, true);
        let b = encode_message(0x0080, &[1, 2, 3], true, true);
        let mut dec = MessageDecoder::new(true, true);
        let out = dec.feed(&a).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].msg_type, 0x0000);
        let out = dec.feed(&b).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].payload, vec![1, 2, 3]);
    }

    #[test]
    fn decode_handles_split_frames() {
        let framed = encode_message(0x00ba, &[0xaa; 20], true, true);
        let mut dec = MessageDecoder::new(true, true);
        let mut out = Vec::new();
        for chunk in framed.chunks(3) {
            out.extend(dec.feed(chunk).unwrap());
        }
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].msg_type, 0x00ba);
        assert_eq!(out[0].payload, vec![0xaa; 20]);
    }

    #[test]
    fn bad_checksum_rejected() {
        let mut framed = encode_message(0x0080, &[1], true, true);
        let n = framed.len();
        framed[n - 2] ^= 0xff;
        let mut dec = MessageDecoder::new(true, true);
        assert!(dec.feed(&framed).is_err());
    }

    #[test]
    fn sum_checksum_mode_roundtrip() {
        // mode FDL1: checksum "sum", tanpa transcode.
        // payload genap: C meng-swap byte checksum untuk len genap di
        // encode (fin=1) dan decode (fin=2) sehingga round-trip konsisten.
        let framed = encode_message(0x0080, &[0x12, 0x34], false, false);
        let mut dec = MessageDecoder::new(false, false);
        let out = dec.feed(&framed).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].payload, vec![0x12, 0x34]);
    }
}
