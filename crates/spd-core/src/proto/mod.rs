//! Protokol BROM/FDL Unisoc: framing + konstanta perintah + parser.
//!
//! Modul ini murni (tidak bergantung transport), sehingga bisa diuji
//! terpisah dan dipakai ulang oleh mock maupun implementasi libusb.

pub mod brom;
pub mod framing;
