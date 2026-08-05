//! Konstanta protokol BROM/FDL dan pembangun paket.
//!
//! Port setia dari `spd_cmd.h` dan `common.c` (CE1CECL/spd_dump).

// --- Link Control ---
pub const BSL_CMD_CONNECT: u16 = 0x00;

// --- Data Download ---
pub const BSL_CMD_START_DATA: u16 = 0x01;
pub const BSL_CMD_MIDST_DATA: u16 = 0x02;
pub const BSL_CMD_END_DATA: u16 = 0x03;
pub const BSL_CMD_EXEC_DATA: u16 = 0x04;
pub const BSL_CMD_NORMAL_RESET: u16 = 0x05;
pub const BSL_CMD_READ_FLASH: u16 = 0x06;
pub const BSL_CMD_READ_CHIP_TYPE: u16 = 0x07;
pub const BSL_CMD_READ_NVITEM: u16 = 0x08;
pub const BSL_CMD_CHANGE_BAUD: u16 = 0x09;
pub const BSL_CMD_ERASE_FLASH: u16 = 0x0A;
pub const BSL_CMD_REPARTITION: u16 = 0x0B;
pub const BSL_CMD_READ_FLASH_TYPE: u16 = 0x0C;
pub const BSL_CMD_READ_FLASH_INFO: u16 = 0x0D;
pub const BSL_CMD_READ_SECTOR_SIZE: u16 = 0x0F;
pub const BSL_CMD_READ_START: u16 = 0x10;
pub const BSL_CMD_READ_MIDST: u16 = 0x11;
pub const BSL_CMD_READ_END: u16 = 0x12;
pub const BSL_CMD_KEEP_CHARGE: u16 = 0x13;
pub const BSL_CMD_EXTTABLE: u16 = 0x14;
pub const BSL_CMD_READ_FLASH_UID: u16 = 0x15;
pub const BSL_CMD_READ_SOFTSIM_EID: u16 = 0x16;
pub const BSL_CMD_POWER_OFF: u16 = 0x17;
pub const BSL_CMD_CHECK_ROOT: u16 = 0x19;
pub const BSL_CMD_READ_CHIP_UID: u16 = 0x1A;
pub const BSL_CMD_ENABLE_WRITE_FLASH: u16 = 0x1B;
pub const BSL_CMD_ENABLE_SECUREBOOT: u16 = 0x1C;
pub const BSL_CMD_IDENTIFY_START: u16 = 0x1D;
pub const BSL_CMD_IDENTIFY_END: u16 = 0x1E;
pub const BSL_CMD_READ_CU_REF: u16 = 0x1F;
pub const BSL_CMD_READ_REFINFO: u16 = 0x20;
pub const BSL_CMD_DISABLE_TRANSCODE: u16 = 0x21;
pub const BSL_CMD_WRITE_DATETIME: u16 = 0x22;
pub const BSL_CMD_CUST_DUMMY: u16 = 0x23;
pub const BSL_CMD_READ_RF_TRANSCEIVER_TYPE: u16 = 0x24;
pub const BSL_CMD_SET_DEBUGINFO: u16 = 0x25;
pub const BSL_CMD_DDR_CHECK: u16 = 0x26;
pub const BSL_CMD_SELF_REFRESH: u16 = 0x27;
pub const BSL_CMD_WRITE_RAW_DATA_ENABLE: u16 = 0x28;
pub const BSL_CMD_READ_NAND_BLOCK_INFO: u16 = 0x29;
pub const BSL_CMD_SET_FIRST_MODE: u16 = 0x2A;
pub const BSL_CMD_READ_PARTITION: u16 = 0x2D;
pub const BSL_CMD_DLOAD_RAW_START: u16 = 0x31;
pub const BSL_CMD_WRITE_FLUSH_DATA: u16 = 0x32;
pub const BSL_CMD_DLOAD_RAW_START2: u16 = 0x33;
pub const BSL_CMD_READ_LOG: u16 = 0x35;

/// Internal: barisan 0x7E polos, bukan pesan ter-frame.
pub const BSL_CMD_CHECK_BAUD: u16 = 0x7E;
pub const BSL_CMD_END_PROCESS: u16 = 0x7F;

// --- Response ---
pub const BSL_REP_ACK: u16 = 0x80;
pub const BSL_REP_VER: u16 = 0x81;
pub const BSL_REP_INVALID_CMD: u16 = 0x82;
pub const BSL_REP_UNKNOW_CMD: u16 = 0x83;
pub const BSL_REP_OPERATION_FAILED: u16 = 0x84;
pub const BSL_REP_NOT_SUPPORT_BAUDRATE: u16 = 0x85;
pub const BSL_REP_DOWN_NOT_START: u16 = 0x86;
pub const BSL_REP_DOWN_MULTI_START: u16 = 0x87;
pub const BSL_REP_DOWN_EARLY_END: u16 = 0x88;
pub const BSL_REP_DOWN_DEST_ERROR: u16 = 0x89;
pub const BSL_REP_DOWN_SIZE_ERROR: u16 = 0x8A;
pub const BSL_REP_VERIFY_ERROR: u16 = 0x8B;
pub const BSL_REP_NOT_VERIFY: u16 = 0x8C;
pub const BSL_PHONE_NOT_ENOUGH_MEMORY: u16 = 0x8D;
pub const BSL_PHONE_WAIT_INPUT_TIMEOUT: u16 = 0x8E;
pub const BSL_PHONE_SUCCEED: u16 = 0x8F;
pub const BSL_PHONE_VALID_BAUDRATE: u16 = 0x90;
pub const BSL_PHONE_REPEAT_CONTINUE: u16 = 0x91;
pub const BSL_PHONE_REPEAT_BREAK: u16 = 0x92;
pub const BSL_REP_READ_FLASH: u16 = 0x93;
pub const BSL_REP_READ_CHIP_TYPE: u16 = 0x94;
pub const BSL_REP_READ_NVITEM: u16 = 0x95;
pub const BSL_REP_INCOMPATIBLE_PARTITION: u16 = 0x96;
pub const BSL_REP_SIGN_VERIFY_ERROR: u16 = 0xA6;
pub const BSL_REP_READ_CHIP_UID: u16 = 0xAB;
pub const BSL_REP_READ_PARTITION: u16 = 0xBA;
pub const BSL_REP_READ_LOG: u16 = 0xBB;
pub const BSL_REP_UNSUPPORTED_COMMAND: u16 = 0xFE;

/// Ukuran entry tabel partisi pada respons `READ_PARTITION` (0x4C).
pub const PARTITION_ENTRY_SIZE: usize = 0x4C;

/// Satu partisi hasil parsing tabel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    pub name: String,
    pub size: u64,
}

/// Konversi nama ke UTF-16LE (format `copy_to_wstr`).
pub fn name_to_utf16le(name: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(36 * 2);
    for ch in name.encode_utf16().take(35) {
        out.extend_from_slice(&ch.to_le_bytes());
    }
    while out.len() < 36 * 2 {
        out.push(0);
    }
    out
}

/// Konversi UTF-16LE (36 unit) kembali ke nama ASCII (format `copy_from_wstr`).
pub fn name_from_utf16le(buf: &[u8]) -> crate::Result<String> {
    let mut name = String::new();
    for chunk in buf.chunks_exact(2).take(36) {
        let c = u16::from_le_bytes([chunk[0], chunk[1]]);
        if c == 0 {
            break;
        }
        if c >> 8 != 0 {
            break;
        }
        name.push(c as u8 as char);
    }
    if name.is_empty() {
        return Err(crate::protocol_err!("empty partition name"));
    }
    Ok(name)
}

/// Paket `select_partition` — port `select_partition()` dari common.c.
///
/// Layout (little-endian):
///   name[36] UTF-16LE, u32 size, u32 size_hi (jika mode64), u64 dummy.
pub fn select_partition(name: &str, size: u64, mode64: bool) -> Vec<u8> {
    let mut pkt = name_to_utf16le(name);
    pkt.extend_from_slice(&(size as u32).to_le_bytes());
    if mode64 {
        pkt.extend_from_slice(&((size >> 32) as u32).to_le_bytes());
        pkt.extend_from_slice(&0u64.to_le_bytes());
    }
    pkt
}

/// Parsing tabel partisi dari payload `BSL_REP_READ_PARTITION`.
///
/// Port logika `partition_list()` pada common.c: tiap entry 0x4C byte,
/// nama UTF-16LE (36 unit) + ukuran LE32 di offset 0x48. Ukuran di-respons
/// dalam "unit" yang perlu dikonversi via divisor (awal 10).
pub fn parse_partition_table(data: &[u8]) -> crate::Result<Vec<Partition>> {
    let len = data.len();
    if len % PARTITION_ENTRY_SIZE != 0 {
        return Err(crate::protocol_err!(
            "partition table not divisible by struct size (0x{len:x})"
        ));
    }
    let n = len / PARTITION_ENTRY_SIZE;

    let mut divisor = 10usize;
    for i in 0..n {
        let off = i * PARTITION_ENTRY_SIZE;
        let size = u32::from_le_bytes(data[off + 0x48..off + 0x4C].try_into().unwrap()) as u64;
        let mut d = divisor;
        while d > 0 && (size >> d) == 0 {
            d -= 1;
        }
        divisor = d;
    }

    let mut parts = Vec::with_capacity(n);
    for i in 0..n {
        let off = i * PARTITION_ENTRY_SIZE;
        let name = name_from_utf16le(&data[off..off + 72])?;
        let size = u32::from_le_bytes(data[off + 0x48..off + 0x4C].try_into().unwrap()) as u64;
        parts.push(Partition {
            name,
            size: (size << 20) >> divisor,
        });
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, size: u32) -> Vec<u8> {
        let mut e = vec![0u8; PARTITION_ENTRY_SIZE];
        let name_bytes = name_to_utf16le(name);
        e[..name_bytes.len()].copy_from_slice(&name_bytes);
        e[0x48..0x4C].copy_from_slice(&size.to_le_bytes());
        e
    }

    #[test]
    fn name_utf16le_roundtrip() {
        let b = name_to_utf16le("nv");
        assert_eq!(b.len(), 72);
        assert_eq!(name_from_utf16le(&b).unwrap(), "nv");
    }

    #[test]
    fn name_too_long_is_truncated_to_35() {
        let long = "a".repeat(40);
        let b = name_to_utf16le(&long);
        assert_eq!(b.len(), 72);
        assert_eq!(name_from_utf16le(&b).unwrap(), "a".repeat(35));
    }

    #[test]
    fn parse_partition_table_basic() {
        let mut table = Vec::new();
        table.extend(entry("boot", 64)); // 64 MB
        table.extend(entry("nv", 2)); // 2 MB
        table.extend(entry("super", 8000));
        // divisor: 64>>10=0 → ... 64>>6=1 → 6; 2: 2>>1=1; 8000>>10≠0
        // hasil akhir divisor = min → 1 (dari nv=2)
        let parts = parse_partition_table(&table).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].name, "boot");
        assert_eq!(parts[1].name, "nv");
        // (2 << 20) >> 1 = 1 MB
        assert_eq!(parts[1].size, 1 << 20);
        // (64 << 20) >> 1 = 32 MB
        assert_eq!(parts[0].size, 32 << 20);
        // (8000 << 20) >> 1
        assert_eq!(parts[2].size, 4000 << 20);
    }

    #[test]
    fn select_partition_layout() {
        let pkt = select_partition("boot", 0x1234, false);
        assert_eq!(pkt.len(), 72 + 4);
        assert_eq!(name_from_utf16le(&pkt[..72]).unwrap(), "boot");
        assert_eq!(u32::from_le_bytes(pkt[72..76].try_into().unwrap()), 0x1234);

        let pkt64 = select_partition("super", 0x1_0000_0000, true);
        assert_eq!(pkt64.len(), 72 + 16);
        assert_eq!(u32::from_le_bytes(pkt64[76..80].try_into().unwrap()), 0x1);
    }
}
