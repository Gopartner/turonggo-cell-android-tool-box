//! Database chipset Unisoc: alamat FDL/exec per chip.

use serde::{Deserialize, Serialize};

/// Konfigurasi satu chipset.
///
/// Format pada file `chip.cfg`:
///   `<CHIP> <EXEC_ADDR> <FDL1_ADDR> <FDL2_ADDR>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChipConfig {
    pub name: String,
    pub exec_addr: u32,
    pub fdl1_addr: u32,
    pub fdl2_addr: u32,
}

impl ChipConfig {
    pub fn ums9230() -> Self {
        Self {
            name: "ums9230".to_string(),
            exec_addr: 0x6501_5F08,
            fdl1_addr: 0x6500_0800,
            fdl2_addr: 0x9EFF_FE00,
        }
    }

    pub fn from_cfg_line(line: &str) -> crate::Result<Self> {
        let mut it = line.split_whitespace();
        let name = it
            .next()
            .ok_or_else(|| crate::protocol_err!("missing chip name"))?;
        let exec_addr = parse_hex(it.next())?;
        let fdl1_addr = parse_hex(it.next())?;
        let fdl2_addr = parse_hex(it.next())?;
        Ok(Self {
            name: name.to_string(),
            exec_addr,
            fdl1_addr,
            fdl2_addr,
        })
    }
}

fn parse_hex(s: Option<&str>) -> crate::Result<u32> {
    let s = s.ok_or_else(|| crate::protocol_err!("missing address in chip.cfg"))?;
    let clean = s.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(clean, 16).map_err(|_| crate::protocol_err!("invalid address: {s}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cfg_line() {
        let c = ChipConfig::from_cfg_line("ums9230 0x65015f08 0x65000800 0x9efffe00").unwrap();
        assert_eq!(c, ChipConfig::ums9230());
    }
}
