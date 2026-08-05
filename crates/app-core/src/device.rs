//! Tipe perangkat & mode koneksi yang dikenali aplikasi (PRD §5.1).

/// Mode koneksi perangkat.
///
/// Nama varian mengikuti set kanonik yang disepakati dengan front-end
/// (REQ-001): `none | adb | fastboot | recovery | unisoc_download` untuk MVP,
/// plus cadangan `mtk_brom | qcom_edl | samsung_download` & `mtk_preloader`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceMode {
    None,
    Adb,
    Fastboot,
    Recovery,
    QcomEdl,
    MtkBrom,
    MtkPreloader,
    UnisocDownload,
    SamsungDownload,
}

impl DeviceMode {
    /// Label kanonik mode, dipakai di IPC & log.
    pub fn label(&self) -> &'static str {
        match self {
            DeviceMode::None => "none",
            DeviceMode::Adb => "adb",
            DeviceMode::Fastboot => "fastboot",
            DeviceMode::Recovery => "recovery",
            DeviceMode::QcomEdl => "qcom_edl",
            DeviceMode::MtkBrom => "mtk_brom",
            DeviceMode::MtkPreloader => "mtk_preloader",
            DeviceMode::UnisocDownload => "unisoc_download",
            DeviceMode::SamsungDownload => "samsung_download",
        }
    }
}

/// Satu perangkat yang terdeteksi.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Device {
    pub mode: DeviceMode,
    pub serial: String,
    /// State mentah dari tool, mis. `device`, `unauthorized`, `fastboot`.
    pub state: String,
    /// Pasangan key/value tambahan dari `adb devices -l` / `fastboot getvar`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}

impl Device {
    pub fn new(mode: DeviceMode, serial: &str) -> Self {
        Self {
            mode,
            serial: serial.to_string(),
            state: String::new(),
            extra: Vec::new(),
        }
    }
}

/// Ringkasan hasil scan, dipakai payload `device://status` (FRONTEND-FLOW §2).
/// Nama field `camelCase` mengikuti konvensi payload front-end.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSummary {
    /// True jika ada minimal satu perangkat aktif.
    pub connected: bool,
    pub devices: Vec<Device>,
    /// Durasi scan terakhir (ms).
    pub last_scan_ms: u64,
    /// Peringatan per adapter (mis. binary tidak ditemukan), tidak fatal.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl DeviceSummary {
    /// Buat summary dari hasil scan; `connected` dihitung otomatis.
    pub fn scan(devices: Vec<Device>, last_scan_ms: u64) -> Self {
        let connected = devices.iter().any(|d| !d.serial.is_empty());
        Self {
            connected,
            devices,
            last_scan_ms,
            warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_label_snake_case() {
        assert_eq!(DeviceMode::Adb.label(), "adb");
        assert_eq!(DeviceMode::MtkBrom.label(), "mtk_brom");
        assert_eq!(DeviceMode::UnisocDownload.label(), "unisoc_download");
        assert_eq!(DeviceMode::QcomEdl.label(), "qcom_edl");
    }

    #[test]
    fn mode_serializes_to_label() {
        let json = serde_json::to_string(&DeviceMode::UnisocDownload).unwrap();
        assert_eq!(json, "\"unisoc_download\"");
    }

    #[test]
    fn device_serializes_without_empty_extra() {
        let d = Device::new(DeviceMode::Adb, "ABC123");
        let json = serde_json::to_value(d).unwrap();
        assert!(json.get("extra").is_none());
        assert_eq!(json["serial"], "ABC123");
    }

    #[test]
    fn summary_reports_connected_and_duration() {
        let s = DeviceSummary::scan(vec![Device::new(DeviceMode::Adb, "SER1")], 42);
        let json = serde_json::to_value(&s).unwrap();
        assert_eq!(json["connected"], true);
        assert_eq!(json["devices"].as_array().unwrap().len(), 1);
        assert_eq!(json["lastScanMs"], 42);
        assert!(!DeviceSummary::scan(vec![], 0).connected);
    }
}
