//! Parser output `adb devices -l`.

use app_core::device::{Device, DeviceMode};

/// Parse output `adb devices -l` menjadi daftar [`Device`].
///
/// Format:
/// ```text
/// List of devices attached
/// R5CX1234567    device product:RMX3760 model:RMX3760 device:RMX3760 transport_id:1
/// ```
/// Baris banner daemon (`* daemon not running; starting now on port 5037 *`) dan
/// header diabaikan. Token pertama = serial, kedua = state, sisanya pasangan
/// `key:value` disimpan di `extra`.
pub fn parse_devices(output: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in output.lines() {
        if line.trim() == "List of devices attached" {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() || tokens[0].starts_with('*') {
            continue;
        }
        if tokens.len() < 2 {
            continue;
        }
        let mut d = Device::new(DeviceMode::Adb, tokens[0]);
        d.state = tokens[1].to_string();
        for kv in &tokens[2..] {
            if let Some((k, v)) = kv.split_once(':') {
                d.extra.push((k.to_string(), v.to_string()));
            }
        }
        devices.push(d);
    }
    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_device_with_attributes() {
        let out = concat!(
            "List of devices attached\n",
            "R5CX1234567    device product:RMX3760 model:RMX3760 device:RMX3760 transport_id:1\n",
        );
        let devices = parse_devices(out);
        assert_eq!(devices.len(), 1);
        let d = &devices[0];
        assert_eq!(d.serial, "R5CX1234567");
        assert_eq!(d.state, "device");
        assert_eq!(
            d.extra,
            vec![
                ("product".to_string(), "RMX3760".to_string()),
                ("model".to_string(), "RMX3760".to_string()),
                ("device".to_string(), "RMX3760".to_string()),
                ("transport_id".to_string(), "1".to_string()),
            ]
        );
    }

    #[test]
    fn parses_multiple_states() {
        let out = concat!(
            "List of devices attached\n",
            "R5CX1234567\t\tdevice product:RMX3760\n",
            "0123456789abcdef\toffline transport_id:2\n",
            "emulator-5554\tunauthorized usb:1\n",
        );
        let devices = parse_devices(out);
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[1].state, "offline");
        assert_eq!(devices[2].state, "unauthorized");
        assert_eq!(devices[2].extra, vec![("usb".to_string(), "1".to_string())]);
    }

    #[test]
    fn skips_daemon_banner_and_header() {
        let out = concat!(
            "* daemon not running; starting now at tcp:5037\n",
            "* daemon started successfully\n",
            "\n",
            "List of devices attached\n",
            "\n",
            "SERIAL1    device\n",
        );
        let devices = parse_devices(out);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "SERIAL1");
    }

    #[test]
    fn empty_output_yields_no_devices() {
        assert!(parse_devices("").is_empty());
        assert!(parse_devices("List of devices attached\n\n").is_empty());
    }

    #[test]
    fn malformed_line_without_state_is_skipped() {
        let out = "List of devices attached\nlonely-serial\n";
        assert!(parse_devices(out).is_empty());
    }
}
