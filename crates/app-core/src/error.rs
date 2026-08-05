//! Tipe error terpadu untuk seluruh app-core.

/// Tipe error utama. Diserialisasi menjadi string sederhana agar mudah
/// dikirim melalui IPC Tauri maupun ditampilkan di terminal.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tool: {0}")]
    Tool(String),

    #[error("device: {0}")]
    Device(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
