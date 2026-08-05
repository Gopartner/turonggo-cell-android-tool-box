//! Kontrak adapter tool eksternal (ADB/Fastboot) + eksekusi aman.
//!
//! Desain (PRD §8): setiap tool dibungkus adapter yang tahu cara
//! menjalankan binary, mendeteksi perangkat, dan mengkonversi output
//! mentah menjadi hasil terstruktur. Error & timeout dinormalisasi ke
//! [`crate::error::Error`] agar aman dikirim melalui IPC.

use std::io::Read;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::device::Device;
use crate::error::{Error, Result};

/// Timeout default pemanggilan tool (deteksi harus cepat).
pub const TOOL_TIMEOUT_MS: u64 = 3000;

/// Output mentah sebuah proses tool eksternal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolOutput {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

impl ToolOutput {
    /// Baris-baris stdout (mengabaikan baris kosong).
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.stdout.lines().map(str::trim).filter(|l| !l.is_empty())
    }
}

/// Jalankan binary eksternal dengan timeout, kembalikan output terstruktur.
pub fn run_command(binary: &str, args: &[&str], timeout_ms: u64) -> Result<ToolOutput> {
    let mut child = Command::new(binary)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| Error::Tool(format!("gagal menjalankan {binary}: {e}")))?;

    let mut stdout = child.stdout.take().expect("stdout terpipe");
    let mut stderr = child.stderr.take().expect("stderr terpipe");
    let (out_tx, out_rx) = mpsc::channel::<String>();
    let (err_tx, err_rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = out_tx.send(String::from_utf8_lossy(&buf).into_owned());
    });
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        let _ = err_tx.send(String::from_utf8_lossy(&buf).into_owned());
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if start.elapsed().as_millis() as u64 > timeout_ms {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(Error::Tool(format!("gagal menunggu {binary}: {e}"))),
        }
    };

    let wait = Duration::from_millis(200);
    let stdout = out_rx.recv_timeout(wait).unwrap_or_default();
    let stderr = err_rx.recv_timeout(wait).unwrap_or_default();

    let Some(status) = status else {
        return Err(Error::Tool(format!(
            "{binary} timeout setelah {timeout_ms} ms (stdout: {stdout:?}, stderr: {stderr:?})"
        )));
    };
    Ok(ToolOutput {
        stdout,
        stderr,
        code: status.code().unwrap_or(-1),
    })
}

/// Kontrak adapter tool eksternal (PRD §8.2).
pub trait ToolAdapter: Send + Sync {
    /// Nama adapter untuk log/audit.
    fn name(&self) -> &'static str;
    /// Jalankan binary tool dengan argumen; timeout & error dibungkus di sini.
    fn run(&self, args: &[&str]) -> Result<ToolOutput>;
    /// Deteksi perangkat yang dikenali tool ini.
    fn detect(&self) -> Result<Vec<Device>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_binary() -> &'static str {
        if cfg!(windows) {
            "cmd"
        } else {
            "/bin/sh"
        }
    }

    fn shell_args(script: &'static str) -> Vec<&'static str> {
        if cfg!(windows) {
            vec!["/C", script]
        } else {
            vec!["-c", script]
        }
    }

    #[test]
    fn run_command_captures_stdout() {
        let out = run_command(shell_binary(), &shell_args("echo hello"), 3000).unwrap();
        assert_eq!(out.code, 0);
        assert!(out.stdout.contains("hello"));
    }

    #[test]
    fn run_command_nonzero_exit() {
        let out = run_command(
            shell_binary(),
            &shell_args(if cfg!(windows) { "exit /b 7" } else { "exit 7" }),
            3000,
        )
        .unwrap();
        assert_eq!(out.code, 7);
    }

    #[test]
    fn run_command_missing_binary_is_tool_error() {
        let err = run_command("binary_tidak_ada_xyz", &[], 1000).unwrap_err();
        assert!(err.to_string().contains("tool"));
    }

    #[test]
    fn run_command_timeout_kills_process() {
        let script = if cfg!(windows) {
            "ping -n 10 127.0.0.1"
        } else {
            "sleep 10"
        };
        let err = run_command(shell_binary(), &shell_args(script), 300).unwrap_err();
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn tool_output_lines_trim_and_skip_empty() {
        let out = ToolOutput {
            stdout: "  device  \n\n  \tserial123\t\n".to_string(),
            ..ToolOutput::default()
        };
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines, vec!["device", "serial123"]);
    }
}
