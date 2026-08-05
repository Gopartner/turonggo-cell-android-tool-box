//! Manifest backup: daftar partisi + status + hash untuk resume.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Status satu partisi dalam backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionEntry {
    pub name: String,
    pub size: u64,
    /// Hash SHA-256 dari hasil dump (diisi setelah selesai).
    pub sha256: Option<String>,
    pub done: bool,
}

/// Manifest satu sesi backup (format v1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub chip: String,
    pub target_dir: String,
    pub partitions: Vec<PartitionEntry>,
}

impl Manifest {
    pub fn new(chip: &str, target_dir: &Path) -> Self {
        Self {
            format: 1,
            chip: chip.to_string(),
            target_dir: target_dir.to_string_lossy().to_string(),
            partitions: Vec::new(),
        }
    }

    pub fn path_for(target_dir: &Path) -> PathBuf {
        target_dir.join("manifest.json")
    }

    /// Simpan ke `<target_dir>/manifest.json`.
    pub fn save(&self, target_dir: &Path) -> crate::Result<()> {
        let path = Self::path_for(target_dir);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Muat manifest dari direktori.
    pub fn load(target_dir: &Path) -> crate::Result<Self> {
        let json = std::fs::read(Self::path_for(target_dir))?;
        Ok(serde_json::from_slice(&json)?)
    }

    pub fn add_partition(&mut self, name: &str, size: u64) {
        self.partitions.push(PartitionEntry {
            name: name.to_string(),
            size,
            sha256: None,
            done: false,
        });
    }

    /// Tandai partisi selesai (menghitung hash dari file hasil dump).
    pub fn mark_done(&mut self, name: &str, file: &Path) -> crate::Result<()> {
        let hash = sha256_file(file)?;
        for p in &mut self.partitions {
            if p.name == name {
                p.sha256 = Some(hash);
                p.done = true;
                break;
            }
        }
        Ok(())
    }

    pub fn remaining(&self) -> Vec<&str> {
        self.partitions
            .iter()
            .filter(|p| !p.done)
            .map(|p| p.name.as_str())
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.partitions.iter().all(|p| p.done)
    }
}

/// SHA-256 dari isi file, format hex.
pub fn sha256_file(path: &Path) -> crate::Result<String> {
    let data = std::fs::read(path)?;
    Ok(sha256_bytes(&data))
}

/// SHA-256 dari byte, format hex.
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let dir = std::env::temp_dir().join("spd-manifest-test");
        std::fs::create_dir_all(&dir).unwrap();
        let mut m = Manifest::new("ums9230", &dir);
        m.add_partition("boot", 64 << 20);
        m.add_partition("nv", 2 << 20);
        m.save(&dir).unwrap();

        let loaded = Manifest::load(&dir).unwrap();
        assert_eq!(loaded.chip, "ums9230");
        assert_eq!(loaded.partitions.len(), 2);
        assert_eq!(loaded.remaining(), vec!["boot", "nv"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sha256_is_hex_64() {
        let h = sha256_bytes(b"hello");
        assert_eq!(h.len(), 64);
    }
}
