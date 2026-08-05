<div align="center">

# Turonggo Cell — Android Tool Box

**Backup & restore firmware lintas chipset** — Unisoc · MediaTek · Qualcomm · Samsung

Desktop app · Tauri 2 + React + TypeScript + Rust

[![CI](https://github.com/Gopartner/turonggo-cell-android-tool-box/actions/workflows/ci.yml/badge.svg)](https://github.com/Gopartner/turonggo-cell-android-tool-box/actions/workflows/ci.yml)
[![Release](https://github.com/Gopartner/turonggo-cell-android-tool-box/actions/workflows/release.yml/badge.svg)](https://github.com/Gopartner/turonggo-cell-android-tool-box/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Rust](https://img.shields.io/badge/Rust-1.97+-orange)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)

</div>

## Tentang

Aplikasi desktop untuk **backup dan restore firmware** perangkat Android dari
berbagai vendor chipset. Di balik UI-nya terdapat library protokol Rust murni
yang menangani komunikasi tingkat rendah (BROM/FDL, framing, checksum) dan
bisa dipakai langsung dari terminal (CLI).

> **Status:** dalam pengembangan aktif. Saat ini berfokus pada chipset Unisoc
> (target awal: Realme C53 / ums9230); jalur MediaTek, Qualcomm, dan Samsung
> sudah disiapkan dalam desain mode perangkat.

## Fitur

- 🔌 **Deteksi perangkat otomatis** — ADB, fastboot, dan mode download (BROM)
  dengan polling USB real-time
- 💾 **Backup & restore partisi** — mode Basic dan Advanced, resume dari
  checkpoint, verifikasi integritas
- 🧪 **Mock data** — seluruh UI bisa dikembangkan tanpa hardware/toolchain
  (browser `npm run dev` tanpa Tauri)
- 🛰️ **Arsitektur multi-chipset** — Unisoc (ums9230), MediaTek (BROM/Preloader),
  Qualcomm (EDL), Samsung (Download) dalam satu antarmuka
- 📦 **Library Rust murni** — `spd-core` port setia dari referensi `spd_dump`,
  dengan transport mock yang teruji (64+ unit test)

## Struktur workspace

```
app/
├── src/            # Front-end React (Vite + TypeScript)
├── src-tauri/      # Shell Tauri 2 + command handler IPC
├── crates/
│   ├── spd-core/   # Protokol BROM/FDL, framing, chipdb, session, manifest
│   ├── app-core/   # Deteksi device, enum mode, adapter ADB/Fastboot/Unisoc
│   ├── adb/        # Adapter ADB: devices, getprop, shell, reboot
│   ├── fastboot/   # Adapter Fastboot: getvar, flash, set_active
│   └── spd-cli/    # CLI terminal memakai spd-core (transport mock)
└── docs/           # PRD, kontrak API, alur front-end/back-end, request
```

## Quickstart

Persyaratan: [Node.js ≥ 20](https://nodejs.org), [Rust (stable)](https://rustup.rs).

```bash
npm install          # dependency front-end
npm run tauri dev    # jalankan aplikasi desktop (dev)
```

Preview UI tanpa Tauri (memakai mock data di browser):

```bash
npm run dev
```

Backend (test & CLI):

```bash
cargo test --workspace          # semua unit test
cargo run -p spd-cli -- detect   # deteksi perangkat
cargo run -p spd-cli -- list --mock   # daftar partisi (transport mock)
```

## Dokumentasi

| Dokumen | Isi |
| --- | --- |
| [API Contract](docs/API.md) | **Single source of truth** — command/event IPC, tipe TS, error codes |
| [PRD](docs/PRD.md) | Visi produk: Android Service Software (freemium, multi-chipset) |
| [Backend](docs/BACKEND.md) | Status & cara pakai `spd-core` + `spd-cli` |
| [Front-end Flow](docs/FRONTEND-FLOW.md) | Alur layar, kontrak IPC, event, state |
| [Front-end Rules](docs/FRONTEND-RULES.md) | Aturan penulisan kode front-end |
| [Team Requests](docs/REQUESTS.md) | Channel permintaan front-end ↔ back-end |

Panduan berkontribusi: [`CONTRIBUTING.md`](CONTRIBUTING.md) — trunk-based
development, conventional commits, PR wajib.

## Lisensi

[MIT](LICENSE) © Gopartner
