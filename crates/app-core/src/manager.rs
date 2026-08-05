//! DeviceManager: orkestrasi deteksi perangkat (PRD §5.1, BACKEND §8.3).
//!
//! Urutan deteksi: ADB → Fastboot → Unisoc download mode. Adapter di-inject
//! dari luar agar `app-core` tidak bergantung ke crate `adb`/`fastboot`
//! (mereka justru bergantung ke `app-core`).

use std::time::Instant;

use crate::device::DeviceSummary;
use crate::tool::ToolAdapter;

/// Pengelola deteksi perangkat.
#[derive(Default)]
pub struct DeviceManager {
    adb: Option<Box<dyn ToolAdapter>>,
    fastboot: Option<Box<dyn ToolAdapter>>,
    unisoc: Option<Box<dyn ToolAdapter>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_adb(mut self, adapter: Box<dyn ToolAdapter>) -> Self {
        self.adb = Some(adapter);
        self
    }

    pub fn with_fastboot(mut self, adapter: Box<dyn ToolAdapter>) -> Self {
        self.fastboot = Some(adapter);
        self
    }

    /// Adapter Unisoc download mode (handshake BROM/FDL via spd-core) — DET-04.
    pub fn with_unisoc(mut self, adapter: Box<dyn ToolAdapter>) -> Self {
        self.unisoc = Some(adapter);
        self
    }

    /// Jalankan seluruh adapter secara berurutan dan gabungkan hasilnya.
    ///
    /// Error satu adapter tidak menggagalkan yang lain; dicatat sebagai warning
    /// (mis. binary `adb` belum terpasang).
    pub fn scan(&self) -> DeviceSummary {
        let start = Instant::now();
        let mut devices = Vec::new();
        let mut warnings = Vec::new();

        for adapter in self.adapters() {
            match adapter.detect() {
                Ok(mut found) => devices.append(&mut found),
                Err(e) => warnings.push(format!("{}: {e}", adapter.name())),
            }
        }

        let mut summary = DeviceSummary::scan(devices, start.elapsed().as_millis() as u64);
        summary.warnings = warnings;
        summary
    }

    fn adapters(&self) -> Vec<&dyn ToolAdapter> {
        let mut out: Vec<&dyn ToolAdapter> = Vec::new();
        if let Some(a) = &self.adb {
            out.push(a.as_ref());
        }
        if let Some(a) = &self.fastboot {
            out.push(a.as_ref());
        }
        if let Some(a) = &self.unisoc {
            out.push(a.as_ref());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{Device, DeviceMode};
    use crate::error::{Error, Result};
    use crate::tool::ToolOutput;

    struct MockAdapter {
        name: &'static str,
        devices: Vec<Device>,
        error: Option<String>,
    }

    impl MockAdapter {
        fn ok(name: &'static str, devices: Vec<Device>) -> Self {
            Self {
                name,
                devices,
                error: None,
            }
        }

        fn failing(name: &'static str, msg: &str) -> Self {
            Self {
                name,
                devices: Vec::new(),
                error: Some(msg.to_string()),
            }
        }
    }

    impl ToolAdapter for MockAdapter {
        fn name(&self) -> &'static str {
            self.name
        }

        fn run(&self, _args: &[&str]) -> Result<ToolOutput> {
            Ok(ToolOutput::default())
        }

        fn detect(&self) -> Result<Vec<Device>> {
            match &self.error {
                Some(msg) => Err(Error::Tool(msg.clone())),
                None => Ok(self.devices.clone()),
            }
        }
    }

    fn device(mode: DeviceMode, serial: &str) -> Device {
        Device::new(mode, serial)
    }

    #[test]
    fn scan_aggregates_in_adb_fastboot_order() {
        let mgr = DeviceManager::new()
            .with_fastboot(Box::new(MockAdapter::ok(
                "fastboot",
                vec![device(DeviceMode::Fastboot, "FB1")],
            )))
            .with_adb(Box::new(MockAdapter::ok(
                "adb",
                vec![device(DeviceMode::Adb, "ADB1")],
            )));
        let s = mgr.scan();
        let serials: Vec<&str> = s.devices.iter().map(|d| d.serial.as_str()).collect();
        assert_eq!(serials, vec!["ADB1", "FB1"]);
        assert!(s.connected);
        assert!(s.warnings.is_empty());
    }

    #[test]
    fn scan_survives_failing_adapter() {
        let mgr = DeviceManager::new()
            .with_adb(Box::new(MockAdapter::failing(
                "adb",
                "binary adb tidak ditemukan",
            )))
            .with_fastboot(Box::new(MockAdapter::ok(
                "fastboot",
                vec![device(DeviceMode::Fastboot, "FB1")],
            )));
        let s = mgr.scan();
        assert_eq!(s.devices.len(), 1);
        assert!(s.connected);
        assert_eq!(s.warnings.len(), 1);
        assert!(s.warnings[0].contains("adb"));
    }

    #[test]
    fn scan_without_adapters_is_empty() {
        let s = DeviceManager::new().scan();
        assert!(!s.connected);
        assert!(s.devices.is_empty());
        assert!(s.warnings.is_empty());
    }
}
