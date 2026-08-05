//! Adapter eksekusi binary `fastboot` eksternal.

use std::collections::HashMap;
use std::path::Path;

use app_core::device::Device;
use app_core::error::{Error, Result};
use app_core::tool::{run_command, ToolAdapter, ToolOutput, TOOL_TIMEOUT_MS};

/// Binary default (`fastboot` di PATH). Konfigurasi bisa menunjuk ke
/// `tools/fastboot/fastboot.exe` lewat [`FastbootAdapter::new`].
pub const FASTBOOT_DEFAULT_BINARY: &str = "fastboot";

/// Adapter Fastboot — membungkus binary eksternal di balik trait [`ToolAdapter`].
#[derive(Debug, Clone)]
pub struct FastbootAdapter {
    binary: String,
}

impl FastbootAdapter {
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    pub fn with_default_binary() -> Self {
        Self::new(FASTBOOT_DEFAULT_BINARY)
    }

    /// Path binary yang dipakai adapter ini.
    pub fn binary(&self) -> &str {
        &self.binary
    }
}

impl ToolAdapter for FastbootAdapter {
    fn name(&self) -> &'static str {
        "fastboot"
    }

    fn run(&self, args: &[&str]) -> Result<ToolOutput> {
        run_command(&self.binary, args, TOOL_TIMEOUT_MS)
    }

    fn detect(&self) -> Result<Vec<Device>> {
        let out = self.run(&["devices"])?;
        Ok(crate::parser::parse_devices(&out.stdout))
    }
}

impl FastbootAdapter {
    /// Ambil semua var via `fastboot -s <serial> getvar all`.
    pub fn getvar_all(&self, serial: &str) -> Result<HashMap<String, String>> {
        let out = self.run(&["-s", serial, "getvar", "all"])?;
        Ok(crate::parser::parse_getvar_all(&out.stdout))
    }

    /// Ambil satu var; `None` bila device tidak mengenal var tersebut.
    pub fn getvar(&self, serial: &str, var: &str) -> Result<Option<String>> {
        let out = self.run(&["-s", serial, "getvar", var])?;
        let vars = crate::parser::parse_getvar_all(&out.stdout);
        Ok(vars.get(var).cloned().filter(|v| !v.is_empty()))
    }

    /// Ukuran partisi (byte) dari `partition-size:<name>`; `None` bila tidak dilaporkan.
    pub fn partition_size(&self, serial: &str, name: &str) -> Result<Option<u64>> {
        let raw = self.getvar(serial, &format!("partition-size:{name}"))?;
        let Some(raw) = raw else { return Ok(None) };
        let hex = raw.trim().trim_start_matches("0x");
        let parsed = u64::from_str_radix(hex, 16).map_err(|_| {
            Error::Tool(format!("ukuran partisi tidak valid: {raw}"))
        })?;
        Ok(Some(parsed))
    }

    /// Flash partisi dengan validasi ukuran (FB-02): menolak bila file melebihi
    /// ukuran partisi yang dilaporkan device.
    pub fn flash(&self, serial: &str, partition: &str, file: &Path) -> Result<()> {
        let meta = std::fs::metadata(file).map_err(|e| {
            Error::Device(format!("file {file:?} tidak terbaca: {e}"))
        })?;
        if let Some(max) = self.partition_size(serial, partition)? {
            if meta.len() > max {
                return Err(Error::Device(format!(
                    "file {} ({} B) melebihi ukuran partisi {partition} ({} B)",
                    file.display(),
                    meta.len(),
                    max
                )));
            }
        }

        let file_str = file.to_string_lossy();
        let out = self.run(&["-s", serial, "flash", partition, &file_str])?;
        ensure_ok(&format!("flash {partition}"), &out)
    }

    /// Set slot aktif A/B (`fastboot set_active <slot>`).
    pub fn set_active(&self, serial: &str, slot: &str) -> Result<()> {
        let out = self.run(&["-s", serial, "set_active", slot])?;
        ensure_ok("set_active", &out)
    }

    /// Reboot perangkat keluar dari fastboot.
    pub fn reboot(&self, serial: &str) -> Result<()> {
        let out = self.run(&["-s", serial, "reboot"])?;
        ensure_ok("reboot", &out)
    }
}

/// Validasi output fastboot: kode 0 dan mengandung OKAY/Finished.
fn ensure_ok(action: &str, out: &ToolOutput) -> Result<()> {
    if out.code != 0 || (!out.stdout.contains("OKAY") && !out.stdout.contains("Finished.")) {
        return Err(Error::Device(format!(
            "{action} gagal ({}): {}",
            out.code,
            out.stderr.trim()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn write_fake_fastboot(script: &str) -> String {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "fastboot_test_{}_{}",
            std::process::id(),
            seq
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("fastboot.cmd");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(script.as_bytes()).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn detect_uses_fastboot_devices() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%1\"==\"devices\" (\r\n  echo R5CX123\tfastboot\r\n)\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        let devices = adapter.detect().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "R5CX123");
        assert_eq!(devices[0].state, "fastboot");
    }

    #[test]
    fn missing_binary_is_tool_error() {
        let adapter = FastbootAdapter::new("fastboot_tidak_ada_xyz");
        let err = adapter.detect().unwrap_err();
        assert!(err.to_string().contains("tool"));
    }

    #[test]
    fn getvar_all_parses_output() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"getvar\" (\r\n  echo ^(bootloader^) product:RMX3760\r\n  echo ^(bootloader^) slot-count:2\r\n  echo all: done\r\n)\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        let vars = adapter.getvar_all("R5CX123").unwrap();
        assert_eq!(vars.get("product").map(String::as_str), Some("RMX3760"));
        assert_eq!(vars.get("slot-count").map(String::as_str), Some("2"));
    }

    #[test]
    fn partition_size_parses_hex() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"getvar\" if \"%4\"==\"partition-size:boot\" echo ^(bootloader^) partition-size:boot: 0x100000\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        let size = adapter.partition_size("R5CX123", "boot").unwrap();
        assert_eq!(size, Some(0x100000));
    }

    #[test]
    fn flash_rejects_oversized_file() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"getvar\" if \"%4\"==\"partition-size:boot\" echo ^(bootloader^) partition-size:boot: 0x10\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        let big = std::env::temp_dir().join("fastboot_big.bin");
        std::fs::write(&big, vec![0u8; 1024]).unwrap();
        let err = adapter.flash("R5CX123", "boot", &big).unwrap_err();
        assert!(err.to_string().contains("melebihi ukuran"));
    }

    #[test]
    fn flash_succeeds_on_okay_output() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"flash\" echo Sending 'boot' (1 KB) OKAY\r\n  echo Writing 'boot' OKAY\r\n  echo Finished. Total time: 0.1s\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        let small = std::env::temp_dir().join("fastboot_small.bin");
        std::fs::write(&small, [0u8; 4]).unwrap();
        adapter.flash("R5CX123", "boot", &small).unwrap();
    }

    #[test]
    fn set_active_passes_slot() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"set_active\" if \"%4\"==\"a\" echo Setting current slot to 'a' OKAY\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        adapter.set_active("R5CX123", "a").unwrap();
    }

    #[test]
    fn reboot_okay() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"reboot\" echo Rebooting OKAY\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        adapter.reboot("R5CX123").unwrap();
    }

    #[test]
    fn reboot_failure_is_device_error() {
        let fake = write_fake_fastboot(
            "@echo off\r\nif \"%3\"==\"reboot\" echo FAILED (remote: device stuck)\r\n",
        );
        let adapter = FastbootAdapter::new(fake);
        let err = adapter.reboot("R5CX123").unwrap_err();
        assert!(err.to_string().contains("reboot"));
    }
}
