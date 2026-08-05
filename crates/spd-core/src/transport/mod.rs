//! Abstraksi transport: libusb / serial / mock.
//!
//! Seluruh protokol (proto/) hanya bergantung pada trait [`Transport`],
//! sehingga kita bisa menguji alur lengkap dengan [`mock::MockTransport`]
//! tanpa hardware, dan menyambungkan libusb asli nanti di fase berikutnya.

pub mod mock;

pub use mock::{part, MockTransport};

/// Error tingkat transport (tidak termasuk error protokol).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Tidak ada device terhubung / device dicabut.
    NoDevice(String),
    /// Gagal menulis.
    Write(String),
    /// Gagal membaca / timeout.
    Read(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::NoDevice(m) => write!(f, "no device: {m}"),
            TransportError::Write(m) => write!(f, "write: {m}"),
            TransportError::Read(m) => write!(f, "read: {m}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Transport byte-level yang digunakan oleh sesi BROM.
///
/// Semua data yang lewat di sini adalah byte mentah hasil framing
/// (sudah termasuk transcode/escape). Implementasi harus sinkron
/// dan blocking — pemanggil yang ingin non-blocking membungkusnya
/// di task async sendiri.
pub trait Transport: Send {
    /// Tulis seluruh `data`. Harus mengembalikan error bila tidak
    /// semua byte terkirim.
    fn write(&mut self, data: &[u8]) -> Result<(), TransportError>;

    /// Baca byte ke `buf`. Kembalikan jumlah byte yang terbaca
    /// (boleh kurang dari panjang `buf`). Menunggu paling lama
    /// `timeout` ms bila `buf` kosong — mengembalikan 0 saat timeout.
    fn read(&mut self, buf: &mut [u8], timeout_ms: u64) -> Result<usize, TransportError>;
}
