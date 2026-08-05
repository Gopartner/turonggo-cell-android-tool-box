# AGENTS.md — SPD Backup Tool

Panduan untuk agen AI (opencode, dll.) yang mengerjakan repo ini. Tujuan repo:
aplikasi desktop backup/restore firmware Unisoc (Realme C53 / ums9230) dengan
Tauri 2 + React + TypeScript + library protokol Rust.

## Struktur workspace

```
app/
├── src/                 # front-end React (Vite + TS)
├── src-tauri/           # shell Tauri + command handler IPC (masih minimal)
├── crates/spd-core/     # library Rust murni: protokol BROM/FDL, framing, session, manifest, chipdb
├── crates/app-core/     # Application Core: deteksi device, enum mode, trait ToolAdapter
├── crates/adb/          # ADB adapter: `adb devices`, getprop, shell/reboot
├── crates/fastboot/     # Fastboot adapter: `fastboot devices`, getvar, flash
├── crates/spd-cli/      # CLI terminal memakai spd-core (transport mock)
└── docs/                # PRD, BACKEND.md, API.md, FRONTEND-FLOW.md, FRONTEND-RULES.md, REQUESTS.md
```

Rencana (belum dibuat): `crates/app-data` & modul `app-core::diagnostics` —
lihat `docs/BACKEND.md` §8 (pemetaan PRD) & §9 (roadmap MVP).
Pemetaan lengkap modul PRD → crate ada di `docs/BACKEND.md` §8.

**Front-end**: sebelum mulai, baca `docs/API.md` — sumber tunggal kontrak API
(command, event, tipe TS, error) dan status tiap fitur. Back-end wajib update
`docs/API.md` lebih dulu setiap menambah/mengubah command Rust.

Workspace Cargo: `members = ["src-tauri", "crates/spd-core", "crates/spd-cli", "crates/app-core", "crates/adb", "crates/fastboot"]`.

## Perintah verifikasi

```bash
# Rust — wajib setelah ubah kode Rust
cargo build --workspace          # build semua (bisa lama di build pertama Tauri)
cargo test -p spd-core           # unit test protokol (22 test, wajib hijau)

# Front-end — wajib setelah ubah kode src/
npm run build                    # tsc && vite build
```

Jangan abaikan warning Rust; selesaikan sampai bersih sebelum selesai.

## Konvensi & aturan kunci

- **Front-end**: patuhi `docs/FRONTEND-RULES.md` (struktur, naming, tipe,
  `invoke`/`listen` hanya lewat `src/lib/api.ts` & `src/lib/events.ts`).
- **Kontrak IPC**: definisi tunggal tipe payload ada di `src/types/ipc.ts`;
  setiap perubahan harus sinkron dengan struct serde di Rust. Kontrak final
  ada di `docs/FRONTEND-FLOW.md` §2.
- **Backend Rust**: bahasa komentar memakai Bahasa Indonesia, konsisten dengan
  kode yang ada. Jangan menambah dependency tanpa alasan tertulis.
- **spd-core adalah port setia** dari `../spd_src/spd_dump/` (common.c,
  spd_cmd.h). Jangan mengubah perilaku framing/checksum/mock agar "lebih
  masuk akal" — salin perilaku C apa adanya, termasuk quirk yang terdokumentasi:
  - checksum sum tidak konsisten untuk pesan panjang ganjil (device asli hanya
    membalas pesan genap; mock memakai payload genap).
  - divisor tabel partisi mengikuti loop C (`while (d > 0 && (size >> d) == 0)`).
- **Pindah fase**: transisi BROM→FDL terjadi setelah `EXEC_DATA`, dan ACK exec
  dikirim **sebelum** pindah fase (lihat `transport/mock.rs`).

## Aturan per tim

Setiap tim hanya boleh mengubah kode di **scope masing-masing**. Kolaborasi
lintas scope dilakukan lewat file `docs/REQUESTS.md`.

| Tim              | Scope yang boleh diubah                        | Tidak boleh diubah        |
| ---------------- | ---------------------------------------------- | ------------------------- |
| **Front-end**    | `src/`, `docs/FRONTEND-*.md`                   | `crates/`, `src-tauri/`   |
| **Back-end**     | `crates/`, `src-tauri/`, `docs/BACKEND.md`     | `src/`                    |

- File lain yang dibolehkan untuk kedua tim: `AGENTS.md`, `README.md`,
  `docs/REQUESTS.md`, dan `docs/` secara umum untuk yang bukan scope tim lain.
- **Front-end** membutuhkan fitur/ubah API dari back-end? Tulis request di
  `docs/REQUESTS.md` dengan format yang sudah disediakan. Jangan mengubah
  `crates/` / `src-tauri/` sendiri.
- **Back-end** wajib membaca `docs/REQUESTS.md`, meninjau, dan membalas setiap
  request di file yang sama (status `accepted` / `rejected` / `needs-info` +
  alasan). Boleh menolak dengan penjelasan.
- Status request hanya ditandai selesai oleh back-end setelah implementasi
  benar-benar ada; jangan asumsikan `accepted` = langsung jadi.

## Standar integrasi API — `docs/API.md` = single source of truth

- **Back-end (wajib):** sebelum menambah/mengubah command, event, atau bentuk
  data di Rust (`src-tauri`/`crates`), **update `docs/API.md` lebih dulu**.
  Isi minimal: TS Types/Interfaces (request, response, error, enums), daftar
  Tauri Commands (nama, parameter input, contoh return value), daftar
  Real-time Events (nama + payload), dan error codes.
- **Front-end (acuan):** UI/UX mengikuti `docs/PRD.md`; kontrak integrasi
  mengikuti `docs/API.md`. Salin interfaces dari `docs/API.md` ke folder
  `src/types/`, dan pakai **Mock Data di service layer** sesuai contoh JSON di
  `docs/API.md` — pengerjaan UI (Basic/Advanced Mode, Log Viewer, Wizard) tidak
  boleh menunggu binary/build back-end.
- **Ketidaksesuaian / masukan API:** diskusikan lewat Issue; untuk perubahan
  scope lintas tim gunakan `docs/REQUESTS.md`.

## Status saat ini

- `spd-core`: selesai, 22 test hijau. `spd-cli`: selesai, jalan dengan `--mock`.
- `app-core`: `tool.rs` (trait `ToolAdapter`, `run_command` timeout),
  `device.rs` (enum `DeviceMode` set kanonik REQ-001, `Device`, `DeviceSummary`),
  `manager.rs` (`DeviceManager::scan`, urutan ADB→Fastboot→Unisoc) — 12 test.
- `crates/adb`: parser `adb devices -l`, `getprop`/`AdbDeviceInfo` (REQ-001),
  shell/reboot (`RebootMode` incl. `download`) — 15 test.
- `crates/fastboot`: parser `fastboot devices`, `getvar all`,
  `flash` (validasi ukuran partisi hex), `set_active`, `reboot` — 15 test.
- `spd-cli`: command `detect` & `adb-info [serial]`.
- `docs/REQUESTS.md`: REQ-001/002 accepted (bertahap), REQ-003 accepted
  (modul `app-core::diagnostics` belum dibuat).
- `src-tauri`: **hanya `greet`** — commands/events IPC sesuai FRONTEND-FLOW
  belum diimplementasikan.
- Transport libusb asli: **belum** — CLI menolak perintah tanpa `--mock`.
- FDL asli dari `tools/chipsets/ums9230/`: belum diikat ke CLI/session.
- Batch berikutnya: diagnostics (REQ-003), DET-04 (Unisoc ke deteksi),
  DET-05 (polling USB + `device_get_ports`), ADB-04 (install/debloat),
  integrasi Tauri IPC (MVP-10).
- Pemetaan modul PRD → crate: `docs/BACKEND.md` §8.

## Saat mengerjakan

1. Baca `docs/BACKEND.md` dulu untuk status, pemetaan PRD (§8), & roadmap (§9).
2. Baca `docs/FRONTEND-FLOW.md` untuk alur & kontrak IPC target.
3. Back-end: rujuk `docs/API.md` untuk kontrak command/event & tipe yang sudah
   siap dipakai front-end. Update `docs/API.md` **lebih dulu** sebelum
   menambah/mengubah command di `src-tauri`.
4. Ubah minimal, ikuti pola file di sekitarnya, verifikasi dengan perintah di atas.
