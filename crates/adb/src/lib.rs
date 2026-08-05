//! adb: adapter ADB (PRD §8 Tool Adapter).
//!
//! Menjalankan binary `adb` eksternal (dari PATH atau `tools/adb/`) dan
//! mengkonversi output menjadi struct. Murni library, tanpa dependensi Tauri.

pub mod adapter;
pub mod info;
pub mod parser;

pub use adapter::{AdbAdapter, RebootMode, ADB_DEFAULT_BINARY};
pub use info::AdbDeviceInfo;
pub use parser::parse_devices;
