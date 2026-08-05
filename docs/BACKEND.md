# Backend — SPD Backup Tool

Dokumen ini menjelaskan apa yang **sudah disediakan** oleh sisi back-end
(`crates/spd-core` + `crates/spd-cli`) dan bagaimana cara memakainya, supaya tim
front-end tahu apa yang bisa langsung digunakan dan apa yang masih dikerjakan.

Dokumen terkait: [Front-end Flow](FRONTEND-FLOW.md) (kontrak IPC) dan
[Front-end Rules](FRONTEND-RULES.md) (aturan penulisan front-end).

## 1. Status

| Bagian                                   | Status            | Keterangan                                  |
| ---------------------------------------- | ----------------- | ------------------------------------------- |
| `crates/spd-core` (library protokol)     | ✅ selesai        | 22 unit test hijau                          |
| `crates/spd-cli` (CLI terminal)          | ✅ selesai        | bisa dipakai dengan transport mock          |
| Transport **mock**                       | ✅ selesai        | menjalankan seluruh alur tanpa hardware     |
| Transport **libusb asli**                | ⏳ belum           | nanti; saat ini CLI menolak tanpa `--mock`  |
| `partition_list` / `dump` / `write`      | ✅ selesai        | teruji lewat mock                           |
| Manifest backup (resume)                 | ✅ selesai        | `spd_core::core::manifest`                  |
| File FDL dari `tools/chipsets/ums9230/`  | ⏳ integrasi      | belum diikat ke CLI/session                 |
| Tauri commands IPC (`src-tauri`)         | ⏳ belum           | `lib.rs` masih cuma `greet` (lihat §7)      |

## 2. Menjalankan backend

```bash
cd app

# build semua (termasuk CLI + shell Tauri)
cargo build --workspace

# jalankan semua unit test backend
cargo test -p spd-core

# pakai CLI dengan transport mock
cargo run -p spd-cli -- info
cargo run -p spd-cli -- connect --mock
cargo run -p spd-cli -- list --mock
cargo run -p spd-cli -- dump boot_a --mock
cargo run -p spd-cli -- dump-all --mock
```

Contoh output nyata:

```
> spd-cli list --mock
name                           size
prodnv                     67108864
miscdata                    1048576
boot_a                     67108864
boot_b                     67108864
nv                          2097152
super                    8388608000

> spd-cli dump boot_a --mock
dumped 67108864 bytes to boot_a.bin
```

## 3. Peta modul `crates/spd-core`

| Modul                              | Isi                                                              |
| ---------------------------------- | ---------------------------------------------------------------- |
| `src/lib.rs`                       | Re-export: `Transport`, `Result`, `Error`, `protocol_err!`       |
| `src/error.rs`                     | `Error` (Io / Protocol / Transport / Other) + alias `Result`     |
| `src/proto/framing.rs`             | Frame HDLC `0x7E...0x7E`, escape (`transcode`), `crc16`, `checksum` sum, `MessageDecoder` |
| `src/proto/brom.rs`                | Konstanta perintah/respons BSL, `Partition`, `select_partition`, `parse_partition_table`, konversi nama UTF-16LE |
| `src/transport/mod.rs`             | Trait `Transport` (`write` / `read`)                             |
| `src/transport/mock.rs`            | `MockTransport` + `MockStage::Brom/Fdl` + helper `part(name, mb)` |
| `src/core/session.rs`              | `Session`, `Flags`, alur handshake / FDL / list / dump / write / reset / poweroff |
| `src/core/manifest.rs`             | `Manifest` v1 + SHA-256, untuk resume backup                     |
| `src/chipdb.rs`                    | `ChipConfig` (exec/fdl1/fdl2 address), `ums9230()`, `from_cfg_line` |

## 4. API inti yang akan dipakai front-end

Semua tipe ini tersedia dari `spd_core`. Nanti dipanggil lewat Tauri commands
(§7), bukan langsung dari React.

```rust
use spd_core::core::{Flags, Session};
use spd_core::transport::MockTransport;
use spd_core::chipdb::ChipConfig;

// Fase protokol menentukan checksum & framing yang dipakai.
Flags::brom(); // fase BROM  : crc16  + transcode
Flags::fdl();  // fase FDL1/2: checksum sum + transcode

// Alur masuk ke mode FDL2 (semua langkah sudah dikerjakan Session):
//   connect() -> send_fdl(fdl1, fdl1_addr, 528) -> EXEC_DATA
//   -> enter_fdl1() (CHECK_BAUD + CONNECT, retry 3x)
//   -> send_fdl(fdl2, fdl2_addr, 528) -> exec_fdl2()

Session::partition_list()  // -> Vec<Partition { name, size }>
Session::dump_partition(name, start, len, blk, writer) // -> u64 (byte)
Session::dump_all(partitions, open_fn)
Session::write_partition(name, data, blk)
Session::reset()
Session::poweroff()
```

`ChipConfig::ums9230()`: `exec_addr=0x65015f08`, `fdl1_addr=0x65000800`,
`fdl2_addr=0x9efffe00` (sumber: `tools/../chip.cfg`).

`Manifest` (untuk fitur resume backup):

```rust
let m = Manifest::new("ums9230", "target_dir");
m.add_partition("boot", size, sha256);
m.mark_done("boot");
m.remaining();   // partisi yang belum selesai
m.is_complete();
m.save() / Manifest::load(path)
```

## 5. Konsep protokol (ringkas, cukup untuk tim front-end)

- Pesan: `[type u16 BE][len u16 BE][payload][checksum u16 BE]`, dibungkus
  `0x7E` di awal/akhir. Byte `0x7E`/`0x7D` dalam payload di-escape menjadi
  `0x7D 0x5E` / `0x7D 0x5D`.
- Checksum fase BROM = **crc16** (polinom 0x11021, bitwise).
- Checksum fase FDL = **sum** (`spd_checksum`).
- Ganti fase terjadi setelah `EXEC_DATA`: BROM→FDL1→(eksekusi FDL2)→FDL2.
- Tabel partisi (respons `READ_PARTITION` 0xBA): tiap entry **0x4C byte**,
  nama **UTF-16LE 36 unit**, ukuran **LE32 di offset 0x48**. Ukuran di-respons
  dalam "unit" dan dikonversi via divisor.
- **Catatan penting** (sudah ditiru persis dari C, jangan diubah): untuk pesan
  ber-panjang-ganjil, checksum sum encode/decode tidak konsisten → device asli
  hanya membalas pesan ber-panjang-genap; mock memakai payload genap untuk itu.
- Mock `MockTransport::new(parts)` mulai di fase BROM; `::fdl(parts)` langsung
  fase FDL (dipakai test list/dump/write).

## 6. spd-cli — daftar perintah

| Perintah             | Fungsi                                       | Butuh transport? |
| -------------------- | -------------------------------------------- | ---------------- |
| `info`               | Info chipset & alamat FDL                    | tidak            |
| `connect --mock`     | Handshake BROM + boot FDL1/FDL2              | ya               |
| `list --mock`        | Ambil daftar partisi                         | ya               |
| `dump <name> --mock` | Dump satu partisi ke `<name>.bin`            | ya               |
| `dump-all --mock`    | Dump semua partisi ke `images/`              | ya               |
| `write <n> <f> --mock` | Tulis file ke partisi                      | ya               |
| `reset` / `poweroff` | Kirim perintah reset / matikan               | ya               |

Tanpa `--mock`, perintah selain `info` menolak (transport libusb belum ada).

## 7. Integrasi Tauri (langkah berikutnya)

Kontrak IPC di `docs/FRONTEND-FLOW.md` **belum** diimplementasikan.
`src-tauri/src/lib.rs` saat ini hanya menyediakan `greet`.

Yang akan dikerjakan berikutnya:

1. Tambahkan dependency `spd-core` ke `src-tauri/Cargo.toml`.
2. Buat state terkelola (Mutex) untuk `Session` + alur backup di background
   (thread/std), supaya `invoke` tidak memblok UI.
3. Implementasikan commands sesuai §2.1 FRONTEND-FLOW:
   `device_get_status`, `partition_list`, `backup_start`, `backup_cancel`,
   `restore_start`, `settings_get`, `settings_set`, `app_version`.
4. Emit events sesuai §2.2: `device://status`, `session://progress`,
   `session://log`, `session://done`, `session://error`.
5. Tipe payload di §2.3 FRONTEND-FLOW dibuat sebagai `serde` struct di Rust
   yang nama fieldnya menjadi kunci TS.

Sampai itu selesai, tim front-end bisa:
- Menjalankan `spd-cli --mock` untuk melihat perilaku backend yang akan di-IPC-kan.
- Menganggap `session://*` event dan command list di FRONTEND-FLOW sebagai
  kontrak final; bentuk event (`sessionId`, `progress`, `LogEntry`) sudah final.

## 8. Pemetaan PRD → arsitektur kode

Referensi: `docs/PRD.md` (visi produk). Tabel ini memetakan lapisan arsitektur
PRD (§7) ke crate/modul di repo, dan menandai mana yang sudah ada vs rencana.

| Lapisan PRD (§7)               | Implementasi sekarang            | Rencana (crate baru)                        |
| ------------------------------ | -------------------------------- | ------------------------------------------- |
| Desktop UI                     | `src/` (React) + `src-tauri/`    | —                                           |
| Protocol — **Unisoc Adapter**  | `crates/spd-core` (proto/framing, proto/brom, core/session) | — |
| Protocol — **ADB Adapter**     | —                                | `crates/adb` (tool adapter atas `adb`)      |
| Protocol — **Fastboot Adapter**| —                                | `crates/fastboot` (tool adapter atas `fastboot`) |
| Protocol — Qualcomm/MTK/Samsung| —                                | crate baru (Phase 2)                        |
| Service — Flash/Partition/Recovery | `crates/spd-core::core::session` (Unisoc) | dibagikan pola ke adapter lain  |
| App Core — Device Manager      | —                                | `crates/app-core::device` (deteksi mode USB) |
| App Core — Module Manager      | —                                | `crates/app-core::modules` (daftar adapter) |
| App Core — Job Manager         | —                                | `crates/app-core::job` (queue/progress/cancel) |
| App Core — Validation Engine   | checksum/size di `spd-core`      | `crates/app-core::validate` (model/codename/slot/A-B) |
| App Core — Backup Manager      | `spd_core::core::manifest`       | `crates/app-core::backup` (device-info, checksums/) |
| App Core — Firmware Manager    | —                                | `crates/app-core::firmware` (import/validasi/profile) |
| App Core — License Manager     | —                                | `crates/app-core::license` (gating premium) |
| Safety Layer                   | validasi checksum/ukuran di `spd-core` | `crates/app-core::validate` + audit log |
| Data Layer                     | —                                | `crates/app-data` (SQLite: history, profiles, preferences) |

### 8.1 Layout crate yang disarankan (untuk MVP)

```text
crates/
├── spd-core/        # ✅ ada — protokol Unisoc (FDL1/FDL2), transport trait, session, manifest
├── app-core/        # ✅ ada — device/mode, DeviceManager, trait ToolAdapter (diagnostics/job/validate 🔜)
├── adb/             # ✅ ada — `adb devices`, getprop/AdbDeviceInfo, shell/reboot
├── fastboot/        # ✅ ada — `fastboot devices`, getvar all, flash, set_active, reboot
├── app-data/        # 🔜 Data layer: SQLite (rusqlite), profiles, operation history
└── spd-cli/         # ✅ ada — CLI pemakaian backend (detect, adb-info, --mock)
```

Status `app-core` (DET-01..03, REQ-001):
- `crates/app-core::tool` — `ToolAdapter` trait + `ToolOutput` + `run_command` (timeout, capture).
- `crates/app-core::device` — enum `DeviceMode` (set kanonik REQ-001:
  `none | adb | fastboot | recovery | unisoc_download | qcom_edl | mtk_brom |
  mtk_preloader | samsung_download`), struct `Device` & `DeviceSummary`.
- `crates/app-core::manager` — `DeviceManager::scan()` urutan ADB→Fastboot→Unisoc,
  error adapter tidak fatal (jadi warning).
- `crates/app-core::error` — tipe `Error` terpadu (serializable string).

Status `crates/adb` (ADB-01..03):
- parser `adb devices -l` → `Device` (serial, state, `extra` key/value).
- `AdbDeviceInfo` dari getprop (REQ-001: manufacturer/model/productName/codename/
  androidVersion/buildFingerprint/securityPatch/brand/sdk, serde `camelCase`).
- `shell`, `reboot(RebootMode)` — `download` = masuk SPD Download mode.

Status `crates/fastboot` (FB-01..03):
- parser `fastboot devices` → `Device` (serial, state, `extra`).
- `getvar_all` (key bisa mengandung `:` mis. `partition-size:boot`), `getvar`,
  `partition_size` (hex), `flash` (validasi ukuran file vs partisi).
- `set_active` (slot A/B), `reboot`.

### 8.2 Adapter tool eksternal (PRD §8 "Tool Adapter")

ADB & Fastboot dikerjakan sebagai **adapter terisolasi** terhadap binary
eksternal di `tools/adb/`, `tools/fastboot/` (bukan dependency langsung).
Setiap adapter mengimplementasikan satu trait umum yang menyediakan:

```rust
trait ToolAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, args: &[&str]) -> Result<ToolOutput>;      // jalankan + timeout
    fn detect(&self) -> Result<Vec<Device>>;                 // adb devices / fastboot devices
}
// ToolOutput { stdout, stderr, code } → masing-masing adapter punya parser
```
(Implementasi dasar sudah ada di `crates/app-core::tool::run_command`.)

Hasil dipakai oleh `app-core::device` untuk menebak mode perangkat
(ADB / Fastboot / Download mode) dan oleh Tauri commands.

### 8.3 Mode & deteksi device (PRD §5.1)

Urutan deteksi (di `crates/app-core::device`):
1. **ADB** — `adb devices` → daftar serial.
2. **Fastboot** — `fastboot devices` → daftar serial.
3. **Unisoc download mode** — `spd-core` transport (BROM/FDL1/FDL2) → handshake.
4. Mode lain (Recovery/EDL/BROM/preloader/Samsung) — fase lanjutan.

## 9. Roadmap MVP — batch berikutnya

Prioritas MVP v1 (PRD §9): Device Detection Core, ADB, Fastboot, Job & Logging,
Validation, Backup, Unisoc. Batch kerja terdekat:

| ID     | Task                                              | Crate           | Output                             |
| ------ | ------------------------------------------------- | --------------- | ---------------------------------- |
| MVP-01 | Init `crates/adb` + trait `ToolAdapter`           | adb ✅          | `adb devices` ter-parse             |
| MVP-02 | ADB: info device (`getprop`), reboot, shell       | adb ✅          | `AdbDeviceInfo` + `RebootMode`      |
| MVP-03 | Init `crates/fastboot` + adapter                  | fastboot ✅     | `fastboot devices` ter-parse        |
| MVP-04 | Fastboot: flash partition, slot, reboot           | fastboot ✅     | `flash` (validasi ukuran), `set_active` |
| MVP-05 | Init `crates/app-core::device` (deteksi mode)     | app-core ✅     | enum `DeviceMode` + `DeviceManager::scan` |
| MVP-06 | Job Manager (queue, progress, cancel)             | app-core        | status job per PRD §5.13            |
| MVP-07 | Validation Engine (model, codename, size, slot)   | app-core        | hasil validasi terstruktur          |
| MVP-08 | Firmware Manager (import, parse, validasi)        | app-core        | firmware profile                    |
| MVP-09 | Init `crates/app-data` (SQLite)                   | app-data        | history + preferences tersimpan     |
| MVP-10 | Integrasi Tauri commands/events (kontrak §7)      | src-tauri       | invoke + session://* siap           |

Status: MVP-01..05 selesai (64 test hijau: 22 spd-core + 12 app-core + 15 adb +
15 fastboot). Berikutnya: diagnostics (REQ-003), DET-04 (Unisoc ke deteksi),
DET-05 (polling USB + `device_get_ports`), ADB-04 (install/debloat), MVP-10.

Setiap item diselesaikan dengan pola yang sama: crate kecil → unit test →
verifikasi `cargo build --workspace` + `cargo test`. Detail breakdown per-item
dicatat saat pengerjaan dimulai (lihat `docs/REQUESTS.md` untuk permintaan
lintas tim).
