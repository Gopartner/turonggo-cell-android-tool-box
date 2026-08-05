//! fastboot: adapter Fastboot (PRD §8 Tool Adapter).
//!
//! Menjalankan binary `fastboot` eksternal dan mengkonversi output menjadi
//! struct. Murni library, tanpa dependensi Tauri.

pub mod adapter;
pub mod parser;

pub use adapter::{FastbootAdapter, FASTBOOT_DEFAULT_BINARY};
pub use parser::parse_devices;
