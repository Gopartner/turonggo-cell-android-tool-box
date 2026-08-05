//! Adapter eksekusi binary `adb` eksternal.

use std::collections::HashMap;

use app_core::device::Device;
use app_core::error::{Error, Result};
use app_core::tool::{run_command, ToolAdapter, ToolOutput, TOOL_TIMEOUT_MS};

use crate::info::AdbDeviceInfo;

/// Binary default (`adb` di PATH). Konfigurasi bisa menunjuk ke
/// `tools/adb/adb.exe` lewat [`AdbAdapter::new`].
pub const ADB_DEFAULT_BINARY: &str = "adb";

/// Adapter ADB — membungkus binary eksternal di balik trait [`ToolAdapter`].
#[derive(Debug, Clone)]
pub struct AdbAdapter {
    binary: String,
}

impl AdbAdapter {
    pub fn new(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    pub fn with_default_binary() -> Self {
        Self::new(ADB_DEFAULT_BINARY)
    }

    /// Path binary yang dipakai adapter ini.
    pub fn binary(&self) -> &str {
        &self.binary
    }
}

impl ToolAdapter for AdbAdapter {
    fn name(&self) -> &'static str {
        "adb"
    }

    fn run(&self, args: &[&str]) -> Result<ToolOutput> {
        run_command(&self.binary, args, TOOL_TIMEOUT_MS)
    }

    fn detect(&self) -> Result<Vec<Device>> {
        let out = self.run(&["devices", "-l"])?;
        Ok(crate::parser::parse_devices(&out.stdout))
    }
}

impl AdbAdapter {
    /// Ambil seluruh property via `adb -s <serial> shell getprop`.
    pub fn getprop(&self, serial: &str) -> Result<HashMap<String, String>> {
        let out = self.run(&["-s", serial, "shell", "getprop"])?;
        Ok(crate::info::parse_getprop(&out.stdout))
    }

    /// Kumpulkan info perangkat (REQ-001) dari getprop.
    pub fn device_info(&self, serial: &str) -> Result<AdbDeviceInfo> {
        let props = self.getprop(serial)?;
        Ok(AdbDeviceInfo::from_props(serial, &props))
    }

    /// Jalankan shell command dan kembalikan stdout mentah.
    pub fn shell(&self, serial: &str, command: &str) -> Result<String> {
        let out = self.run(&["-s", serial, "shell", command])?;
        Ok(out.stdout)
    }

    /// Reboot perangkat ke mode tertentu (ADB-03).
    ///
    /// `Download` memakai `adb reboot download` — cara masuk SPD Download mode
    /// dari Android (dipakai sebelum handshake BROM).
    pub fn reboot(&self, serial: &str, mode: RebootMode) -> Result<()> {
        let mut args = vec!["-s", serial, "reboot"];
        if let Some(arg) = mode.arg() {
            args.push(arg);
        }
        let out = self.run(&args)?;
        if out.code != 0 {
            return Err(Error::Device(format!(
                "reboot gagal ({}): {}",
                out.code,
                out.stderr.trim()
            )));
        }
        Ok(())
    }
}

/// Mode reboot ADB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebootMode {
    Normal,
    Bootloader,
    Recovery,
    Download,
    Edl,
}

impl RebootMode {
    fn arg(&self) -> Option<&'static str> {
        match self {
            RebootMode::Normal => None,
            RebootMode::Bootloader => Some("bootloader"),
            RebootMode::Recovery => Some("recovery"),
            RebootMode::Download => Some("download"),
            RebootMode::Edl => Some("edl"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn write_fake_adb(script: &str) -> String {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("adb_test_{}_{}", std::process::id(), seq));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("adb.cmd");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(script.as_bytes()).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn detect_uses_adb_devices_l() {
        let fake = write_fake_adb(
            "@echo off\r\nif \"%1\"==\"devices\" (\r\n  echo List of devices attached\r\n  echo R5CX123\tdevice product:RMX3760\r\n)\r\n",
        );
        let adapter = AdbAdapter::new(fake);
        let devices = adapter.detect().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "R5CX123");
        assert_eq!(devices[0].state, "device");
    }

    #[test]
    fn getprop_parses_and_maps_info() {
        let fake = write_fake_adb(
            "@echo off\r\nif \"%4\"==\"getprop\" (\r\n  echo [ro.product.manufacturer]: [Realme]\r\n  echo [ro.build.version.release]: [13]\r\n)\r\n",
        );
        let adapter = AdbAdapter::new(fake);
        let info = adapter.device_info("R5CX123").unwrap();
        assert_eq!(info.serial, "R5CX123");
        assert_eq!(info.manufacturer.as_deref(), Some("Realme"));
        assert_eq!(info.android_version.as_deref(), Some("13"));
    }

    #[test]
    fn shell_runs_command_and_returns_stdout() {
        let fake = write_fake_adb(
            "@echo off\r\nif \"%~4\"==\"echo hello\" echo hello-from-device\r\n",
        );
        let adapter = AdbAdapter::new(fake);
        let out = adapter.shell("R5CX123", "echo hello").unwrap();
        assert!(out.contains("hello-from-device"));
    }

    #[test]
    fn reboot_download_passes_download_arg() {
        let fake = write_fake_adb(
            "@echo off\r\nif \"%3\"==\"reboot\" if \"%4\"==\"download\" echo Rebooting into download mode\r\n",
        );
        let adapter = AdbAdapter::new(fake);
        adapter.reboot("R5CX123", RebootMode::Download).unwrap();
    }

    #[test]
    fn reboot_normal_has_no_mode_arg() {
        let fake = write_fake_adb(
            "@echo off\r\nif \"%3\"==\"reboot\" if not \"%4\"==\"download\" echo Reboot\r\n",
        );
        let adapter = AdbAdapter::new(fake);
        adapter.reboot("R5CX123", RebootMode::Normal).unwrap();
    }

    #[test]
    fn missing_binary_is_tool_error() {
        let adapter = AdbAdapter::new("adb_tidak_ada_xyz");
        let err = adapter.detect().unwrap_err();
        assert!(err.to_string().contains("tool"));
    }
}
