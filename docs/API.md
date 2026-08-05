# API.md — Kontrak API Back-End → Front-End (Sumber Tunggal)

Dokumen ini adalah **satu-satunya** rujukan API untuk integrasi front-end
(Tauri 2 IPC) dan back-end (Rust). Front-end membangun UI terhadap bentuk data
di sini; back-end menjamin command/event di sini sesuai implementasi `src-tauri`.

> **Aturan wajib (back-end):** setiap kali menambah/mengubah command Rust atau
> mengubah bentuk payload, **update `docs/API.md` lebih dulu**, baru
> implementasikan di `src-tauri`. Jangan pernah mengubah perilaku API tanpa
> memperbarui dokumen ini.
>
> Sumber pendukung: `docs/FRONTEND-FLOW.md` (alur) & `src/types/ipc.ts`
> (definisi tipe TS kanonik). Front-end tidak mengubah `crates/`/`src-tauri/`;
> back-end tidak mengubah `src/`. Perubahan kontrak lewat `docs/REQUESTS.md`.

Status tiap fitur:

- **✅ READY** — logika back-end ada & teruji (`cargo test`); bentuk data final.
- **🚧 WIRING** — logika siap, command/event IPC belum di-wire di `src-tauri` (MVP-10).
- **🔜 PLANNED** — belum dibuat.

---

## 1. TypeScript Types / Interfaces (Request, Response, Error, Enums)

Semua nama field memakai **camelCase** (serde `rename_all = "camelCase"` di
Rust). Nilai enum JSON memakai **huruf kecil**.

### 1.1 `ApiResponse<T>` — envelope respon standar

**Setiap** command IPC membalas salah satu dari dua bentuk:

```ts
export interface ApiResponse<T> {
  success: boolean;
  data?: T;                 // ada jika success === true
  error?: {
    code: string;           // kode error baku (tabel §4)
    message: string;        // pesan siap tampil (Rust error string)
    suggestion?: string;    // saran tindakan untuk user
  };
}
```

Front-end cukup cek `res.success`, lalu baca `res.data` / `res.error`.

### 1.2 Enums

```ts
export type DeviceMode =
  | "none" | "adb" | "fastboot" | "recovery" | "unisoc_download"
  | "mtk_brom" | "mtk_preloader" | "qcom_edl" | "samsung_download";

export type AdbRebootTarget =
  | "system" | "recovery" | "bootloader" | "download" | "edl";

export type SessionPhase = "prepare" | "read" | "write" | "verify" | "finalize";
```

> ⚠️ **Gate**: UI hanya menyalakan Backup/Restore bila
> `mode === "unisoc_download"`; flash hanya saat `mode === "fastboot"`.

### 1.3 Response interfaces (diterima front-end)

```ts
// —— Satu perangkat hasil scan ——
export interface Device {
  mode: DeviceMode;
  serial: string;
  state: string;               // adb: "device"|"offline"|"unauthorized"; fastboot: "fastboot"|"fastbootd"
  extra?: [string, string][];  // pasangan key/value `adb devices -l` / `fastboot devices`; hilang bila kosong
}

// —— Hasil scan internal (sumber data untuk menyusun DeviceStatus) ——
export interface DeviceSummary {
  connected: boolean;
  devices: Device[];           // urutan: ADB → Fastboot (→ Unisoc nanti)
  lastScanMs: number;          // durasi scan (ms)
  warnings?: string[];         // error non-fatal per adapter (mis. adb tidak terpasang)
}

// —— Info perangkat dari getprop (adapter ADB) ——
export interface AdbDeviceInfo {
  serial: string;
  manufacturer?: string;
  model?: string;
  productName?: string;
  codename?: string;
  brand?: string;
  androidVersion?: string;
  buildFingerprint?: string;
  securityPatch?: string;
  sdk?: string;
}

// —— Partisi dari tabel BROM/FDL (size dalam byte) ——
export interface PartitionInfo {
  name: string;
  size: number;
  offset?: number;
  isUserPartition?: boolean;
}

// —— Port USB (REQ-002) ——
export interface UsbPortInfo {
  name: string;
  state: "connected" | "error" | "idle";
  detail?: string;
}

// —— Health score device (REQ-003) ——
export interface DeviceHealth {
  score: number;               // 0..100
  riskLevel: "low" | "medium" | "high";
  issues: { severity: "info" | "warn" | "critical"; message: string }[];
  recommendedAction?: string;
  checkedAt: number;           // epoch ms
}

// —— Info perangkat lengkap: payload device_get_status & event device://status ——
// (disusun back-end dari DeviceSummary + AdbDeviceInfo + getvar + handshake)
export interface DeviceStatus {
  mode: DeviceMode;
  manufacturer?: string;
  model?: string;
  productName?: string;
  codename?: string;
  androidVersion?: string;
  buildFingerprint?: string;
  securityPatch?: string;
  bootloaderStatus?: "locked" | "unlocked" | "unknown";
  slotActive?: string;
  isAbDevice?: boolean;
  driverOk?: boolean;
  ports?: UsbPortInfo[];
  chip?: string;
  chipId?: string;
  fdlLoaded?: boolean;
  transport?: string;
}
```

### 1.4 Request interfaces (dikirim front-end)

```ts
// BackupOptions / RestoreOptions / AppSettings / BackupManifest —
// lihat FRONTEND-FLOW.md §2.3 (definisi kanonik di src/types/ipc.ts).

export interface BackupOptions {
  sessionId?: string;
  partitions: string[];        // nama partisi yang dipilih
  targetDir: string;
  resume?: boolean;
}

export interface RestoreOptions {
  partitions: { name: string; file: string }[];
}

export interface SessionProgress {
  sessionId: string;
  phase: SessionPhase;
  percent: number;
  bytesRead: number;
  bytesTotal: number;
  currentPartition?: string;
}
```

### 1.5 Contoh JSON aktual (ground truth dari serde Rust)

```jsonc
// DeviceSummary
{
  "connected": true,
  "devices": [ { "mode": "adb", "serial": "R5CX1234567", "state": "device",
                 "extra": [["model", "RMX3760"]] } ],
  "lastScanMs": 42,
  "warnings": ["adb: tool: gagal menjalankan adb: ..."]
}

// AdbDeviceInfo (field yang tidak ada = null)
{
  "serial": "R5CX1234567", "manufacturer": "Realme", "model": "RMX3760",
  "productName": "RMX3760", "codename": "RE5D4L", "brand": "realme",
  "androidVersion": "13",
  "buildFingerprint": "realme/RMX3760/RMX3760:13/RKQ1.211103.002/...:user/release-keys",
  "securityPatch": "2024-01-05", "sdk": "33"
}

// fastboot getvar all → map key→value (key bisa mengandung ':')
{ "version": "0.4", "product": "RMX3760", "slot-count": "2",
  "current-slot": "a", "partition-type:boot": "raw" }

// PartitionInfo[]
[ { "name": "prodnv", "size": 67108864 },
  { "name": "boot_a", "size": 67108864 },
  { "name": "super",  "size": 8388608000 } ]
```

---

## 2. Daftar Tauri Commands (Nama, Input, Contoh Return Value)

> Nama command memakai konvensi `kata_kerja_obyek` dari FRONTEND-FLOW §2.
> Dipanggil lewat `invoke("nama_command", args)` → selalu balas `ApiResponse<T>`.

### A. Device Detection

#### `device_get_status` — status & deteksi perangkat
- **Input:** *(none)*
- **Return:**
```json
{ "success": true, "data": {
  "mode": "unisoc_download", "chip": "ums9230", "chipId": "0x00130000",
  "fdlLoaded": true, "transport": "libusb", "manufacturer": "Realme",
  "model": "RMX3760", "productName": "realme C53", "codename": "RMX3760",
  "androidVersion": "14", "securityPatch": "2025-03-05",
  "buildFingerprint": "realme/RMX3760/RMX3760:14/...:user/release-keys",
  "bootloaderStatus": "unlocked", "slotActive": "a", "isAbDevice": true,
  "driverOk": true, "ports": [ { "name": "SPRD U2S Diag (COM3)",
    "state": "connected", "detail": "SPD Download Mode" } ] } }
```
- **Status:** 🚧 WIRING (logika ✅ `app-core::DeviceManager::scan`; field
  `chip/chipId/fdlLoaded/transport` terisi hanya setelah handshake BROM di mode `unisoc_download`)

#### `device_get_ports` — daftar port USB (REQ-002)
- **Input:** *(none)*
- **Return:** `{ "success": true, "data": [ { "name": "COM9", "state": "connected", "detail": "Qualcomm..." } ] }`
- **Status:** 🔜 PLANNED (menunggu DET-05)

#### `device_get_health` — health score (REQ-003)
- **Input:** *(none)*
- **Return:** `{ "success": true, "data": { "score": 85, "riskLevel": "medium", "issues": [], "checkedAt": 1720000000000 } }`
- **Status:** 🔜 PLANNED (modul `app-core::diagnostics`)

### B. ADB & Fastboot

#### `device_adb_reboot` — reboot perangkat ADB
- **Input:** `{ "deviceId": "R5CX1234567", "target": "download" }`
- **Return:** `{ "success": true, "data": null }`
- **Status:** ✅ READY (logika `AdbAdapter::reboot`) — IPC 🚧
- **Error:** `DEVICE_NOT_FOUND`, `ADB_COMMAND_FAILED`
- **Catatan:** `target: "download"` = `adb reboot download` (masuk SPD Download mode — wajib sebelum handshake BROM).

#### `device_adb_shell` — jalankan shell command
- **Input:** `{ "deviceId": "...", "command": "cat /proc/cmdline" }`
- **Return:** `{ "success": true, "data": "console=ttyS1...\\n" }` (stdout mentah)
- **Status:** ✅ READY (logika) — IPC 🚧

#### `device_adb_info` — info perangkat dari getprop
- **Input:** `{ "deviceId": "R5CX1234567" }`
- **Return:** `{ "success": true, "data": { "serial": "...", "manufacturer": "Realme", "model": "RMX3760" } }`
- **Status:** ✅ READY (logika) — IPC 🚧

#### `device_fastboot_getvar` — baca var fastboot
- **Input:** `{ "deviceId": "...", "var": "all" }`
- **Return:** `{ "success": true, "data": { "slot-count": "2", "current-slot": "a" } }`
- **Status:** ✅ READY (logika) — IPC 🚧

#### `device_fastboot_flash` — flash partisi (validasi ukuran)
- **Input:** `{ "deviceId": "...", "partitionName": "boot", "filePath": "C:/fw/boot.img", "verifyChecksum": true }`
- **Return:** `{ "success": true, "data": { "bytesWritten": 67108864, "timeElapsedMs": 4210 } }`
- **Status:** ✅ READY (logika) — IPC 🚧
- **Error:** `PARTITION_OVERSIZE`, `DEVICE_NOT_FOUND`, `FLASH_FAILED`

### C. Partition & Backup/Restore

| Command | Input | Return (data) | Status |
| --- | --- | --- | --- |
| `partition_list` | *(none)* | `PartitionInfo[]` | 🚧 (logika ✅ spd-core mock) |
| `backup_start` | `BackupOptions` | `{ "sessionId": "..." }` | 🔜 |
| `backup_cancel` | `{ "sessionId": "..." }` | `null` | 🔜 |
| `restore_start` | `RestoreOptions` | `{ "sessionId": "..." }` | 🔜 |
| `backup_read_manifest` | `{ "dir": "..." }` | `BackupManifest` | 🔜 |

Contoh `backup_start`:
```jsonc
// request
{ "partitions": ["prodnv", "boot_a"], "targetDir": "C:/backup/2026-08" }
// response
{ "success": true, "data": { "sessionId": "a1b2c3" } }
// lanjut progress → subscribe event session://progress (§3)
```

### D. Settings & Lain-lain

| Command | Input | Return (data) | Status |
| --- | --- | --- | --- |
| `settings_get` | *(none)* | `AppSettings` | 🔜 |
| `settings_set` | `AppSettings` | `null` | 🔜 |
| `app_version` | *(none)* | `"0.1.0"` | 🚧 (VERSION siap) |
| `dialog_pick_folder` | *(none)* | `string \| null` | 🔜 |

---

## 3. Daftar Real-time Events (Nama, Payload)

Event dipancarkan back-end → front-end via pub/sub. Front-end pakai
`listen("channel", handler)` dan **wajib** `unlisten()` saat cleanup.

| Channel | Payload | Arti | Status |
| --- | --- | --- | --- |
| `device://status` | `DeviceStatus` | Deteksi USB colok/cabut/perubahan mode | 🚧 |
| `device://ports` | `UsbPortInfo[]` | Daftar port USB berubah (REQ-002) | 🔜 |
| `session://progress` | `SessionProgress` | Progress backup/restore/flash (persen + byte) | 🔜 |
| `session://log` | `LogEntry` | Baris log real-time | 🔜 |
| `session://done` | `SessionDone` | Operasi selesai sukses | 🔜 |
| `session://error` | `SessionError` | Operasi gagal | 🔜 |

Contoh (auto-detect USB + progress bar):

```ts
import { listen } from "@tauri-apps/api/event";

const un = await listen<DeviceStatus>("device://status", (e) => {
  // e.payload.mode / .devices → render dashboard & status bar
  const canBackup = e.payload.mode === "unisoc_download";
});

const un2 = await listen<SessionProgress>("session://progress", (e) => {
  setPercent(e.payload.percent);
  setPhase(e.payload.phase);
});

// cleanup di useEffect return: un(); un2();
```

---

## 4. Error (Kode Baku)

Semua error dibungkus `ApiResponse.error`. Front-end cukup tampilkan
`error.message` (+ `error.suggestion`) di Toast/Modal & catat ke panel log.

| Code | Arti | Saran (default) |
| --- | --- | --- |
| `DEVICE_NOT_FOUND` | Device/serial tidak terdeteksi | Scan ulang / periksa kabel USB |
| `ADB_COMMAND_FAILED` | Perintah adb ditolak/gagal | Cek prompt authorization di device |
| `FLASH_FAILED` | Flash partisi gagal | Cek ukuran file & koneksi fastboot |
| `PARTITION_OVERSIZE` | File melebihi ukuran partisi | Pilih file yang benar untuk partisi |
| `TOOL_NOT_FOUND` | Binary adb/fastboot tidak ditemukan | Pasang ADB/Fastboot atau atur path |
| `TOOL_TIMEOUT` | Panggilan tool melebihi timeout | Coba lagi / periksa device |
| `DEVICE_UNSUPPORTED` | Mode device tidak didukung operasi | Masuk mode SPD Download |
| `ERR_DRIVER_MISSING` | Driver USB tidak terpasang | Instal driver di Settings |
| `SESSION_ALREADY_RUNNING` | Ada operasi lain berjalan | Hentikan operasi aktif dulu |
| `INTERNAL` | Error tidak dikenal | Lihat log / laporkan |

Contoh:

```json
{
  "success": false,
  "error": {
    "code": "ERR_DRIVER_MISSING",
    "message": "Driver USB SPD Download tidak ditemukan.",
    "suggestion": "Instal driver via Settings > Install Drivers."
  }
}
```

---

## 5. Alur Update (Wajib Dibaca Back-End)

1. Ingin menambah/mengubah command, event, atau bentuk data di Rust?
2. **Edit `docs/API.md` ini lebih dulu** (nama, input, contoh return, status).
3. Implementasikan di `src-tauri`; pastikan JSON keluar persis seperti di sini.
4. Bila ada tipe TS baru, sinkronkan juga `src/types/ipc.ts` (lewat koordinasi
   `docs/REQUESTS.md` bila scope milik front-end).
5. Ubah status `🔜/🚧` → `✅ READY` hanya setelah `cargo test` hijau.
