# SPD Backup Tool — Desktop App

Aplikasi desktop untuk backup/restore firmware perangkat Unisoc (Realme C53 /
ums9230). Dibangun dengan Tauri 2 + React + TypeScript, dengan logika protokol
di library Rust murni (`crates/spd-core`) yang juga bisa dipakai dari terminal.

## Struktur

- `src/` — front-end React (UI)
- `src-tauri/` — shell Tauri + command handler IPC
- `crates/spd-core/` — library Rust murni (protokol BROM/FDL, framing, chipdb, session, manifest)
- `crates/spd-cli/` — CLI terminal memakai spd-core (transport mock)

## Dokumentasi

- [PRD](docs/PRD.md) — visi produk: Android Service Software (freemium, multi-chipset)
- [Backend](docs/BACKEND.md) — status & cara pakai `spd-core` + `spd-cli`
- [Front-end Flow](docs/FRONTEND-FLOW.md) — alur layar, kontrak IPC, event, state
- [Front-end Rules](docs/FRONTEND-RULES.md) — aturan penulisan kode front-end
- [Team Requests](docs/REQUESTS.md) — channel permintaan front-end ↔ back-end

## Menjalankan

```bash
npm install
npm run tauri dev
```

Backend (CLI + test):

```bash
cargo build --workspace
cargo test -p spd-core
cargo run -p spd-cli -- list --mock
```
