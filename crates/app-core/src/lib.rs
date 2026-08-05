//! app-core: lapisan aplikasi di atas protokol Unisoc.
//!
//! Menyediakan deteksi perangkat (ADB/Fastboot/Unisoc download mode),
//! kontrak adapter tool eksternal, dan (nanti) job manager & validasi.
//! Murni library tanpa dependensi Tauri, dipakai oleh `spd-cli` dan
//! frontend desktop melalui IPC.

pub mod device;
pub mod error;
pub mod manager;
pub mod tool;

pub use device::{Device, DeviceMode, DeviceSummary};
pub use error::{Error, Result};
pub use manager::DeviceManager;
pub use tool::{run_command, ToolAdapter, ToolOutput, TOOL_TIMEOUT_MS};
