//! spd-core: inti logika tool backup/restore firmware Unisoc.
//!
//! Desain: library murni (tanpa dependensi UI/Tauri) agar bisa
//! dijalankan dari terminal (spd-cli), diuji dengan `cargo test`,
//! dan dipakai ulang oleh frontend desktop (Tauri) melalui IPC.

pub mod chipdb;
pub mod core;
pub mod error;
pub mod proto;
pub mod transport;

pub use error::{Error, Result};
pub use transport::{Transport, TransportError};

/// Versi library (ditampilkan di CLI & about).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
