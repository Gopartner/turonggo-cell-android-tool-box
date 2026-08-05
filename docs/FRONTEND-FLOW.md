# Front-end Flow — SPD Backup Tool

Dokumentasi alur (flow) aplikasi desktop Tauri + React/TypeScript.
Front-end = UI React di `src/`; back-end = Rust di `src-tauri/` + `crates/spd-core`.

## 1. Arsitektur komunikasi

```
+--------------------------------------------------+
|  React UI (src/)                                 |
|  views / hooks / components                      |
+--------------------------------------------------+
   |  invoke("command", args)   (request/response)
   v  listen("event")            (progress/log/push)
+--------------------------------------------------+
|  Tauri core (src-tauri/src/lib.rs)               |
|  command handlers + event emitter                |
+--------------------------------------------------+
   |
   v
+--------------------------------------------------+
|  spd-core (crates/spd-core)                      |
|  session BROM, framing, chipdb, manifest         |
+--------------------------------------------------+
```

- Semua panggilan back-end **hanya lewat IPC** (tidak ada HTTP/socket sendiri).
- Operasi panjang (backup/restore) berjalan **async** di back-end dan melaporkan
  kemajuan melalui **Tauri events**; front-end cukup mendengarkan dan merender.
- Front-end **tidak pernah** memblok menunggu hasil operasi panjang; `invoke`
  hanya dipakai untuk memulai/menghentikan operasi dan query status cepat.

## 2. Kontrak IPC (target yang akan diimplementasikan di src-tauri)

> **Sumber kebenaran**: `docs/API.md`. Bagian di bawah adalah ringkasan alur
> & tipe front-end; untuk bentuk JSON aktual, command, event, dan error baku,
> lihat `docs/API.md` (dikelola back-end). Tipe kanonik di `src/types/ipc.ts`
> disalin dari API.md.

### 2.1 Commands (front-end -> back-end)

| Command            | Argumen                        | Balikan                 | Keterangan                     |
|--------------------|--------------------------------|-------------------------|--------------------------------|
| `device_get_status`| —                              | `DeviceStatus`          | Deteksi mode (unisoc_download/fastboot/adb/none) + chip |
| `partition_list`   | —                              | `PartitionInfo[]`       | Daftar partisi dari device     |
| `backup_start`     | `BackupOptions`                | `string` (sessionId)    | Mulai backup; lanjut via event |
| `backup_cancel`    | `sessionId: string`            | `null`                  | Hentikan operasi yang berjalan |
| `restore_start`    | `RestoreOptions`               | `string` (sessionId)    | Mulai restore                 |
| `settings_get`     | —                              | `AppSettings`           | Konfigurasi yang tersimpan     |
| `settings_set`     | `AppSettings`                  | `null`                  | Simpan konfigurasi             |
| `app_version`      | —                              | `string`                | Versi aplikasi                 |
| `dialog_pick_folder`| —                             | `string \| null`        | Pilih folder (dialog OS)       |
| `backup_read_manifest`| `dir: string`                | `BackupManifest`        | Baca manifest hasil backup     |
| `device_get_ports`* | —                              | `UsbPortInfo[]`         | Daftar port USB (lihat REQ-002) |
| `device_get_health`*| —                              | `DeviceHealth`          | Health score device (REQ-003)  |

`*` = masih `pending` di `docs/REQUESTS.md`, belum diimplementasikan back-end.

### 2.2 Events (back-end -> front-end)

| Channel               | Payload                          | Arti                                  |
|-----------------------|----------------------------------|---------------------------------------|
| `device://status`     | `DeviceSummary`                     | Deteksi USB colok/cabut/perubahan mode; front-end mengomposisi `DeviceStatus` |
| `device://ports`*     | `UsbPortInfo[]`                  | Daftar port USB berubah (REQ-002)     |
| `session://progress`  | `SessionProgress`                | Kemajuan operasi (persen, partisi, byte) |
| `session://log`       | `LogEntry`                       | Baris log (info/warn/error)           |
| `session://done`      | `{ sessionId, summary }`         | Operasi selesai sukses                |
| `session://error`     | `{ sessionId, error: string }`   | Operasi gagal                         |

`*` = pending di `docs/REQUESTS.md`.

### 2.3 Tipe payload (cermin dari Rust)

```ts
// Set nama mode final — disepakati dengan back-end (REQUESTS.md REQ-001).
// MVP: none | adb | fastboot | recovery | unisoc_download.
// Cadangan chipset lain (PRD §5.4-5.7): mtk_brom | mtk_preloader | qcom_edl | samsung_download.
type DeviceMode =
  | "none"
  | "adb"
  | "fastboot"
  | "recovery"
  | "unisoc_download"
  | "mtk_brom"
  | "mtk_preloader"
  | "qcom_edl"
  | "samsung_download";

type BootloaderStatus = "locked" | "unlocked" | "unknown";

interface UsbPortInfo {
  name: string;
  state: "connected" | "error" | "idle";
  detail?: string;
}

interface DeviceStatus {
  mode: DeviceMode;
  chip?: string;          // contoh: "ums9230"
  chipId?: number;
  fdlLoaded?: boolean;    // hanya relevan saat mode unisoc_download
  transport?: string;     // "libusb" | "mock"
  manufacturer?: string;
  model?: string;
  productName?: string;
  codename?: string;
  androidVersion?: string;
  buildFingerprint?: string;
  securityPatch?: string;
  bootloaderStatus?: BootloaderStatus;
  slotActive?: string;
  isAbDevice?: boolean;
  driverOk?: boolean;
  ports?: UsbPortInfo[];
}

interface PartitionInfo {
  name: string;           // contoh: "nv", "prodnv", "boot"
  size: number;           // byte
  offset?: number;
  isUserPartition?: boolean;
}

interface BackupOptions {
  sessionId?: string;     // kosongkan untuk session baru
  partitions: string[];   // daftar nama partisi
  targetDir: string;      // direktori tujuan
  resume?: boolean;       // lanjut dari checkpoint
}

interface RestoreOptions {
  partitions: { name: string; file: string }[];
}

interface SessionProgress {
  sessionId: string;
  phase: "prepare" | "read" | "write" | "verify" | "finalize";
  partition?: string;
  bytesDone: number;
  bytesTotal: number;
  percent: number;        // 0..100
  checkpoint?: string;    // info posisi terakhir (untuk resume)
}

interface LogEntry {
  level: "info" | "warn" | "error";
  ts: number;             // epoch ms
  message: string;
}

interface AppSettings {
  chipset: string;        // dipilih dari chipdb
  workDir: string;        // direktori default hasil backup
  resumeEnabled: boolean;
  allowRestoreToStock: boolean; // gating keamanan (red warning)
}
```

## 3. Layar (views) dan navigasi

Tidak memakai router; navigasi = state `view` di `App.tsx` dengan sidebar.
Ada dua mode aplikasi (`AppMode`): **basic** dan **advanced** (PRD §4).

```
+---------------------+----------------------------------+
| SIDEBAR             |  KONTEN (view aktif)             |
|  [mode switch]      |                                  |
|  UTAMA              |                                  |
|   [Dashboard]       |                                  |
|  OPERASI            |                                  |
|   [Backup]          |                                  |
|   [Restore]         |                                  |
|  TEKNIS (Adv)       |                                  |
|   [Info Perangkat]  |                                  |
|  [Settings]         |                                  |
+---------------------+----------------------------------+
|  STATUS BAR: mode device, chip, session berjalan       |
+---------------------------------------------------------+
```

- Mode **basic**: view teknis (`device`) disembunyikan dari navigasi; alur
  dipandu (wizard). Mode **advanced**: semua view tampil + parameter teknis.
- Saat pindah ke basic dari view advanced-only, `view` kembali ke dashboard.

| View       | File                | Mode      | Fungsi                                        |
|------------|---------------------|-----------|-----------------------------------------------|
| Dashboard  | `views/Dashboard`   | basic+adv | Status device, ringkasan partisi, log terbaru |
| Backup     | `views/Backup`      | basic+adv | Pilih partisi, target dir, mulai/lanjut/stop  |
| Restore    | `views/Restore`     | basic+adv | Pilih file hasil backup, mulai restore        |
| DeviceInfo | `views/DeviceInfo`  | adv only  | Detail perangkat, protokol, driver & port USB |
| Settings   | `views/Settings`    | basic+adv | Chipset, work dir, opsi resume, gating restore |

Navigasi rule:
- `App.tsx` menyimpan `view`, `appMode`, dan `session` aktif; komponen lain menerima props/context.
- Semua view dapat berpindah kapan saja; operasi back-end **tidak bergantung** pada view aktif.
- Status bar selalu menampilkan state session aktif di view mana pun.

## 4. Alur utama

### 4.1 Startup & deteksi device

```
App mount
  ├─ useDeviceStatus()
  │   ├─ invoke("device_get_status")  -> tampil mode/chip
  │   ├─ listen("device://status")    -> update saat berubah
  │   └─ refresh manual (tombol "Scan")
  ├─ useSession()
  │   ├─ listen("session://progress") -> state progress
  │   ├─ listen("session://log")      -> feed log panel
  │   ├─ listen("session://done")     -> tampilkan ringkasan
  │   └─ listen("session://error")    -> tampilkan error
  └─ render view aktif
```

- Status awal = `{ mode: "none" }` (device belum tersambung).
- Bila mode = `unisoc_download`, tombol Backup/Restore menjadi aktif (enable).
- Bila `fastboot`/`adb`, tampilkan peringatan: mode ini tidak didukung
  untuk read/write firmware, arahkan ke cara masuk mode Download.

### 4.2 Alur Backup

```
[User] Backup view
  1. klik "Scan partisi"      -> invoke("partition_list") -> tabel
  2. centang partisi yang mau dibackup (default: semua)
  3. pilih targetDir (dialog folder)
  4. klik "Mulai Backup"      -> invoke("backup_start", opts)
       -> dapat sessionId; simpan di state session aktif
  5. UI masuk state "berjalan":
       - tombol jadi "Hentikan"
       - progress bar + nama partisi + byte + persen
       - feed dari session://progress & session://log
  6. selesai                  -> session://done -> ringkasan
                               (jumlah partisi, total byte, path, durasi)
       atau gagal             -> session://error -> toast error + log
```

Fitur resume:
- Backend menulis manifest + checkpoint per partisi (SHA-256).
- Bila backup terputus, list partisi yang `sudah selesai` diberi tanda centang.
- Tombol "Lanjutkan" mengirim `backup_start({ resume: true })`; partisi selesai dilewati.

### 4.3 Alur Restore

```
[User] Restore view
  1. pilih folder hasil backup -> tampilkan manifest (partisi, sha256, chipset)
  2. validasi chipset cocok dengan device terhubung
  3. (bila allowRestoreToStock = false) tampilkan konfirmasi merah + wajib
     mengetik "LANJUT" untuk mengizinkan restore partisi stock
  4. klik "Mulai Restore"  -> invoke("restore_start")
  5. progress & log sama seperti backup (session://*)
  6. selesai -> imbauan: putuskan USB, verifikasi boot
```

### 4.4 Alur batal (cancel)

```
[User] klik "Hentikan"
  -> invoke("backup_cancel", sessionId)
  -> backend berhenti pada batas aman, tulis checkpoint
  -> event session://log (info "dibatalkan di posisi X")
  -> UI kembali ke state idle; tombol "Lanjutkan" tersedia
```

## 5. State management (rule singkat)

- **Tidak ada** Redux/Zustand. State dibagi lewat hooks + React Context kecil:
  - `DeviceProvider` — status device & list partisi.
  - `SessionProvider` — session aktif, progress, log, hasil.
- Komponen leaf membaca dari context/hooks; **tidak** boleh memanggil
  `invoke`/`listen` langsung di JSX — semua dibungkus di `src/lib/`.
- Update state berfrekuensi tinggi (progress) boleh `setState` biasa; React 19
  + StrictMode tetap aman karena listener selalu dibersihkan di `useEffect` cleanup.

## 6. Penanganan error

- Back-end mengirim error sebagai **string** (lihat `spd-core::Error::serialize`).
- Rule: tidak ada error yang di-swallow; error ditampilkan minimal sebagai
  toast + dicatat ke log panel.
- Error yang bisa diprediksi user: device terlepas, timeout handshake, partisi
  tidak ditemukan, checksum mismatch (battery kosong / kabel lepas).

## 7. Lokasi file yang terlibat

| File                       | Peran                                |
|----------------------------|--------------------------------------|
| `src/App.tsx`              | Layout + navigasi view + providers   |
| `src/views/*`              | Layar (Dashboard/Backup/Restore/Settings) |
| `src/lib/api.ts`           | Pembungkus typed untuk `invoke`      |
| `src/lib/events.ts`        | Pembungkus typed untuk `listen`      |
| `src/types/ipc.ts`         | Tipe payload kontrak IPC (cermin Rust) |
| `src/hooks/*`              | Hook status device & session         |
| `src/components/*`         | Komponen UI (tombol, tabel, progress, log, sidebar) |
| `src-tauri/src/lib.rs`     | Command handler + emitter events     |

Lihat juga `docs/FRONTEND-RULES.md` untuk aturan penulisan kode front-end.
