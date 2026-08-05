//! Info perangkat via `getprop` (ADB-02, REQ-001).

use std::collections::HashMap;

/// Parse output `adb shell getprop` (`[key]: [value]`) menjadi map.
///
/// Baris yang tidak cocok pola diabaikan (daemon banner, error, dsb.).
pub fn parse_getprop(output: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix('[') else {
            continue;
        };
        let Some((key, value)) = rest.split_once("]: [") else {
            continue;
        };
        let Some(value) = value.strip_suffix(']') else {
            continue;
        };
        props.insert(key.to_string(), value.to_string());
    }
    props
}

/// Info perangkat yang dikumpulkan dari `getprop` (REQ-001).
///
/// Nama field mengikuti `DeviceStatus` di front-end (`camelCase`).
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbDeviceInfo {
    pub serial: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub product_name: Option<String>,
    pub codename: Option<String>,
    pub brand: Option<String>,
    pub android_version: Option<String>,
    pub build_fingerprint: Option<String>,
    pub security_patch: Option<String>,
    pub sdk: Option<String>,
}

impl AdbDeviceInfo {
    /// Bangun info dari map getprop; key kosong dianggap tidak ada.
    pub fn from_props(serial: &str, props: &HashMap<String, String>) -> Self {
        let prop =
            |key: &str| -> Option<String> { props.get(key).filter(|v| !v.is_empty()).cloned() };
        AdbDeviceInfo {
            serial: serial.to_string(),
            manufacturer: prop("ro.product.manufacturer"),
            model: prop("ro.product.model"),
            product_name: prop("ro.product.name"),
            codename: prop("ro.product.device"),
            brand: prop("ro.product.brand"),
            android_version: prop("ro.build.version.release"),
            build_fingerprint: prop("ro.build.fingerprint"),
            security_patch: prop("ro.build.version.security_patch"),
            sdk: prop("ro.build.version.sdk"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_getprop_lines() {
        let out = concat!(
            "[dalvik.vm.heapsize]: [256m]\n",
            "[ro.build.version.release]: [13]\n",
            "[ro.product.manufacturer]: [Realme]\n",
            "[persist.vendor.power]: []\n",
            "bukan_baris_prop\n",
        );
        let props = parse_getprop(out);
        assert_eq!(
            props.get("ro.build.version.release").map(String::as_str),
            Some("13")
        );
        assert_eq!(
            props.get("ro.product.manufacturer").map(String::as_str),
            Some("Realme")
        );
        assert_eq!(
            props.get("persist.vendor.power").map(String::as_str),
            Some("")
        );
        assert!(props.len() == 4);
    }

    #[test]
    fn empty_output_yields_empty_map() {
        assert!(parse_getprop("").is_empty());
        assert!(parse_getprop("List of devices attached\n").is_empty());
    }

    #[test]
    fn from_props_maps_known_keys_camel_case() {
        let mut props = HashMap::new();
        props.insert("ro.product.manufacturer".into(), "Realme".into());
        props.insert("ro.product.model".into(), "RMX3760".into());
        props.insert("ro.product.name".into(), "RMX3760".into());
        props.insert("ro.product.device".into(), "RE5D4L".into());
        props.insert("ro.build.version.release".into(), "13".into());
        props.insert("ro.build.fingerprint".into(), "realme/RMX3760".into());
        props.insert(
            "ro.build.version.security_patch".into(),
            "2024-01-05".into(),
        );
        props.insert("persist.vendor.empty".into(), "".into());

        let info = AdbDeviceInfo::from_props("R5CX123", &props);
        assert_eq!(info.serial, "R5CX123");
        assert_eq!(info.manufacturer.as_deref(), Some("Realme"));
        assert_eq!(info.model.as_deref(), Some("RMX3760"));
        assert_eq!(info.product_name.as_deref(), Some("RMX3760"));
        assert_eq!(info.codename.as_deref(), Some("RE5D4L"));
        assert_eq!(info.android_version.as_deref(), Some("13"));
        assert_eq!(info.build_fingerprint.as_deref(), Some("realme/RMX3760"));
        assert_eq!(info.security_patch.as_deref(), Some("2024-01-05"));
        assert_eq!(info.brand, None);
        assert_eq!(info.sdk, None);
    }

    #[test]
    fn from_props_ignores_unknown_and_empty() {
        let mut props = HashMap::new();
        props.insert("ro.unknown".into(), "x".into());
        props.insert("ro.build.version.sdk".into(), "".into());
        let info = AdbDeviceInfo::from_props("S", &props);
        assert_eq!(info.model, None);
        assert_eq!(info.sdk, None);
    }
}
