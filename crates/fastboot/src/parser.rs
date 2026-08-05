//! Parser output `fastboot devices` & `fastboot getvar`.

use std::collections::HashMap;

use app_core::device::{Device, DeviceMode};

/// Parse output `fastboot devices` menjadi daftar [`Device`].
///
/// Format:
/// ```text
/// R5CX1234567    fastboot
/// ```
/// Token pertama = serial, kedua = state (`fastboot` / `fastbootd`), sisanya
/// pasangan tambahan (mis. `usb:1-2`) disimpan di `extra`.
pub fn parse_devices(output: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() || tokens[0].starts_with('*') {
            continue;
        }
        if tokens.len() < 2 {
            continue;
        }
        let mut d = Device::new(DeviceMode::Fastboot, tokens[0]);
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

/// Parse output `fastboot getvar all` menjadi map key→value.
///
/// Format: `(bootloader) key: value`. Baris `all: done`, `FAILED`, dan banner
/// diabaikan. Format tanpa prefix (`key: value`) juga didukung.
pub fn parse_getvar_all(output: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("all:")
            || line.starts_with("FAILED")
            || line.starts_with("error:")
        {
            continue;
        }
        let body = if let Some(rest) = line.strip_prefix('(') {
            match rest.split_once(") ") {
                Some((_prefix, body)) => body,
                None => continue,
            }
        } else {
            line
        };
        // rsplit agar key yang mengandung ':' (mis. `partition-size:boot`) utuh.
        let Some((key, value)) = body.rsplit_once(':') else {
            continue;
        };
        vars.insert(key.trim().to_string(), value.trim().to_string());
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fastboot_device() {
        let out = "R5CX1234567\tfastboot\n";
        let devices = parse_devices(out);
        assert_eq!(devices.len(), 1);
        let d = &devices[0];
        assert_eq!(d.serial, "R5CX1234567");
        assert_eq!(d.state, "fastboot");
    }

    #[test]
    fn parses_fastbootd_and_usb_extra() {
        let out = "ABC123\tfastbootd\nXYZ999\tfastboot usb:1-2\n";
        let devices = parse_devices(out);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].state, "fastbootd");
        assert_eq!(devices[1].state, "fastboot");
        assert_eq!(devices[1].extra, vec![("usb".to_string(), "1-2".to_string())]);
    }

    #[test]
    fn empty_and_malformed_lines_are_skipped() {
        assert!(parse_devices("").is_empty());
        assert!(parse_devices("only-serial\n").is_empty());
        assert!(parse_devices("* daemon message\n\n").is_empty());
    }

    #[test]
    fn parses_getvar_all_lines() {
        let out = concat!(
            "(bootloader) version:0.4\n",
            "(bootloader) product:RMX3760\n",
            "(bootloader) partition-type:boot:raw\n",
            "(fastboot) is-userspace:no\n",
            "all: done\n",
            "all: Done\n",
        );
        let vars = parse_getvar_all(out);
        assert_eq!(vars.get("product").map(String::as_str), Some("RMX3760"));
        assert_eq!(vars.get("version").map(String::as_str), Some("0.4"));
        assert_eq!(vars.get("is-userspace").map(String::as_str), Some("no"));
        assert_eq!(
            vars.get("partition-type:boot").map(String::as_str),
            Some("raw")
        );
        assert!(!vars.contains_key("all"));
    }

    #[test]
    fn parses_partition_size_key_with_colon() {
        let out = "(bootloader) partition-size:boot: 0x100000\n";
        let vars = parse_getvar_all(out);
        assert_eq!(
            vars.get("partition-size:boot").map(String::as_str),
            Some("0x100000")
        );
    }

    #[test]
    fn parses_unprefixed_getvar_and_ignores_failures() {
        let out = concat!(
            "product: RMX3760\n",
            "slot-count: 2\n",
            "FAILED (remote: unknown var)\n",
        );
        let vars = parse_getvar_all(out);
        assert_eq!(vars.get("product").map(String::as_str), Some("RMX3760"));
        assert_eq!(vars.get("slot-count").map(String::as_str), Some("2"));
        assert!(!vars.contains_key("FAILED"));
    }
}
