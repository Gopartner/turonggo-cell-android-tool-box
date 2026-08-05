# Front-end Rules — SPD Backup Tool

Aturan penulisan kode front-end (React + TypeScript + Vite + Tauri 2).
Selalu patuhi aturan ini saat menambah/mengubah kode di `src/`.

## 1. Struktur folder `src/`

```
src/
├── main.tsx              # entrypoint (jangan diubah kecuali perlu)
├── App.tsx               # layout + navigasi view + providers (mode basic/advanced)
├── App.css               # hanya style global/theme
├── types/
│   ├── ipc.ts            # tipe payload kontrak IPC (SATU-SATUNYA sumber tipe)
│   └── view.ts           # tipe View + AppMode (navigasi, bukan IPC)
├── lib/
│   ├── api.ts            # semua pemanggilan invoke() (typed wrapper)
│   ├── events.ts         # semua pemanggilan listen()/emit (typed wrapper)
│   ├── format.ts         # format byte/persen/durasi (utilitas murni)
│   └── mock.ts           # simulasi back-end utk preview browser (hapus saat backend siap)
├── hooks/
│   ├── useDevice.tsx     # status device + daftar partisi (JSX provider => .tsx)
│   ├── useSession.tsx    # session aktif + progress + log (JSX provider => .tsx)
│   └── useSettings.tsx   # pengaturan app (JSX provider => .tsx)
├── components/
│   ├── layout/           # Sidebar (mode switch + nav grup), StatusBar, PageHeader
│   ├── common/           # Button, Card, Toast, ProgressBar, LogView, Table, Modal, ...
│   └── device/           # DeviceCard, ModeBadge, PartitionTable, DeviceInfo
└── views/
    ├── Dashboard.tsx
    ├── Backup.tsx
    ├── Restore.tsx
    ├── DeviceInfo.tsx    # Advanced Mode: detail perangkat, driver & port USB
    └── Settings.tsx
```

Catatan:
- Hook yang mengekspor provider JSX (context) memakai ekstensi `.tsx`.
- `lib/mock.ts` hanya dipakai untuk preview UI di browser (di luar runtime Tauri);
  dihapus/diarsipkan saat back-end IPC siap.

Rule:
- Satu fitur = satu file (atau satu folder untuk komponen kompleks).
- Komponen dalam `components/common/` harus **dapat dipakai ulang** dan bebas
  dari logika IPC (hanya menerima props).
- `views/` boleh memakai hooks & context; `components/common/` tidak.
- Jangan menambah folder baru tanpa kebutuhan nyata; kalau perlu, jelaskan di
  PR/commit.

## 2. Naming convention

| Hal          | Rule                                      | Contoh                      |
|--------------|-------------------------------------------|-----------------------------|
| File komponen| PascalCase, satu komponen utama per file  | `BackupView.tsx`            |
| File lain    | camelCase                                 | `useDevice.ts`, `format.ts` |
| Komponen     | PascalCase (function declaration)         | `function ProgressBar`      |
| Props        | camelCase, optional pakai `?`             | `onStart: () => void`       |
| Variabel     | camelCase                                 | `sessionId`, `bytesTotal`   |
| Konstanta    | UPPER_SNAKE                               | `MAX_LOG_LINES = 500`       |
| CSS class    | kebab-case (atau BEM sederhana)           | `.btn--primary`, `.log-row` |
| Events/channel | snake_case, prefix domain            | `session://progress`        |

## 3. TypeScript

- `strict: true` sudah aktif; **dilarang** `any` tanpa alasan terdokumentasi.
- Semua payload IPC didefinisikan **satu kali** di `src/types/ipc.ts`.
  Perubahan payload harus sinkron dengan definisi serde di Rust
  (`crates/spd-core` / `src-tauri`).
- Gunakan `interface` untuk objek, `type` untuk union/alias.
- Ekspor tipe yang dipakai lebih dari satu file.
- Jangan duplikasi tipe di file view; impor dari `types/ipc.ts`.

## 4. IPC (panggilan back-end)

- **Sumber kebenaran kontrak**: `docs/API.md` (satu-satunya). Tipe payload di
  `src/types/ipc.ts` disalin dari API.md; jangan mengubah field tanpa mengubah
  API.md lebih dulu (koordinasi via `docs/REQUESTS.md`).
- `invoke` HANYA lewat `src/lib/api.ts`. Komponen tidak pernah memanggil
  `invoke` langsung.
- `listen` HANYA lewat `src/lib/events.ts`.
- Setiap command membalas **envelope `ApiResponse<T>`** (lihat API.md §1.1).
  `src/lib/api.ts` memakai helper `unwrap()`: `success === true` → ambil
  `data`, selain itu lempar `ApiError` (`code` + `suggestion`).
- Setiap wrapper diberi nama sesuai command, argumen & balikan ditulis
  dengan tipe eksplisit:

```ts
// src/lib/api.ts
import { invoke } from "@tauri-apps/api/core";

export function getDeviceStatus(): Promise<DeviceStatus> {
  return invoke<ApiResponse<DeviceSummary>>("device_get_status").then(unwrap).then(composeDeviceStatus);
}

export function startBackup(opts: BackupOptions): Promise<string> {
  return invoke<ApiResponse<{ sessionId: string }>>("backup_start", { options: opts })
    .then(unwrap)
    .then((d) => d.sessionId);
}
```

- Error: biarkan `ApiError` mengalir; tangkap di view/hook dan tampilkan
  `error.message` (+ `error.suggestion`) sebagai toast + log. Jangan `catch`
  untuk disimpan diam-diam.
- `DeviceStatus` (view-model gabungan) dikomposisi di service layer dari
  `DeviceSummary` + field `Device.extra`; jangan menambah command baru untuk
  itu tanpa persetujuan back-end.

## 5. Events

- Buat listener di dalam `useEffect` dan **wajib cleanup** (hapus listener
  saat unmount). Contoh:

```ts
useEffect(() => {
  const un = listenProgress((p) => setProgress(p));
  return () => { void un.then((f) => f()); };
}, []);
```

- Nama event pakai format `domain://action` (lihat `FRONTEND-FLOW.md`).
- Jangan kirim event dari front-end ke back-end kecuali benar-benar perlu;
  prefer `invoke` untuk aksi.

## 6. React

- Function components + hooks. Tidak ada class component.
- Tidak memakai router; navigasi via state `view` di `App.tsx`.
- StrictMode aktif di `main.tsx`; kode harus bersih dari double-effect issues
  (cleanup listener & interval).
- Hindari `useEffect` yang tidak perlu; prioritaskan derived state.
- Props komponen common: gunakan tipe eksplisit; komponen tanpa props boleh
  kosong `{}`.
- Jangan import React default bila tidak butuh JSX namespace (Vite baru);
  ikuti gaya file yang ada.

## 7. Styling

- CSS biasa (tidak memakai framework UI / CSS-in-JS untuk tahap ini).
- Warna & spacing pakai CSS custom properties di `App.css` (theme).
- Gunakan class, **bukan** inline style kecuali nilai dinamis (mis. lebar
  progress bar: `style={{ width: \`${percent}%\` }}`).
- Dark theme default; aksen & danger (restore) memakai warna khusus.
- Kata di UI pakai Bahasa Indonesia (label, tombol, pesan error).
- Komponen/common harus responsif dalam jendela desktop (min width 800px).

## 8. Progress & operasi panjang

- Progress di-render dari `SessionProgress.percent`; jangan menghitung ulang
  di front-end (back-end sumber kebenaran).
- Saat session berjalan: tombol aksi di view terkait dinonaktifkan, dan
  tombol "Hentikan" ditampilkan.
- Log panel menampung maks `MAX_LOG_LINES` baris (autoscroll).
- Saat `session://done`: reset state progress, tampilkan ringkasan.
- Saat `session://error`: tampilkan toast error, pertahankan log agar bisa
  dibaca pengguna.

## 9. Keamanan & gating

- Restore ke partisi stock memerlukan konfirmasi teks "LANJUT" + pengaturan
  `allowRestoreToStock` (lihat `FRONTEND-FLOW.md`).
- Jangan pernah menampilkan/redact nilai sensitif (IMEI, kunci) di log
  kecuali diperlukan.
- Semua path file dari dialog harus memakai Tauri dialog plugin; jangan
  biarkan user mengetik path bebas (mitigasi path traversal).
- `capabilities/default.json`: hanya izinkan permission yang dipakai
  (jangan menambah `fs:*` global jika tidak perlu).

## 10. Formatting & quality

- Ikuti prettier default (jika belum ada config, gunakan single quote,
  trailing comma, semi).
- Jalankan `npm run build` (tsc + vite) sebelum dianggap selesai.
- Tulis komentar singkat hanya bila logika tidak jelas; jangan komentar
  klise.
- Jangan menambahkan dependency baru tanpa persetujuan & alasan tertulis.

## 11. Definisi Done (untuk tiap tugas front-end)

1. Kode mengikuti aturan di atas (struktur, naming, tipe, IPC wrapper).
2. `npm run build` lolos tanpa error TypeScript.
3. Alur yang bersangkutan tercermin di `docs/FRONTEND-FLOW.md` (update bila
   ada perubahan flow).
4. Tidak ada `any`, tidak ada invoke/listen langsung di komponen, tidak ada
   CSS inline statis.
