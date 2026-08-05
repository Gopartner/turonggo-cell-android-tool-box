# REQUESTS.md — Channel permintaan antar tim

File ini adalah jembatan kolaborasi antara tim **front-end** dan **back-end**.

- **Front-end** menulis request di sini (fitur/ubah API/kontrak IPC yang
  dibutuhkan). Front-end TIDAK mengubah `crates/` / `src-tauri/`.
- **Back-end** membaca file ini, meninjau, dan membalas tiap request
  (status + alasan). Back-end TIDAK mengubah `src/`.

Aturan ringkas: lihat `AGENTS.md` → "Aturan per tim".

## Cara menulis request (front-end)

1. Tambah entry baru di bagian "Daftar request" dengan ID urut `REQ-XXX`.
2. Jangan edit balasan milik back-end; hanya tambah request baru.
3. Satu request = satu kebutuhan yang jelas & terukur.

Format entry:

```md
### REQ-XXX — <judul singkat>
- Tanggal: YYYY-MM-DD
- Diminta oleh: front-end
- Status: pending
- Kebutuhan: <apa yang dibutuhkan front-end dan kenapa>
- Detail: <command/event/tipe, perilaku, contoh, referensi FRONTEND-FLOW.md>

**Balasan back-end:**
_(dijawab di sini oleh back-end)_
```

## Status yang dipakai

| Status        | Arti                                                          |
| ------------- | ------------------------------------------------------------- |
| `pending`     | Request baru, belum ditinjau back-end                         |
| `accepted`    | Disetujui; back-end akan mengerjakan / sudah direncanakan     |
| `rejected`    | Ditolak, disertai alasan                                      |
| `needs-info`  | Back-end butuh klarifikasi / data tambahan dari front-end     |
| `done`        | Implementasi sudah ada & bisa dipakai front-end               |

## Daftar request

<!-- Tambahkan entry baru di bawah baris ini, request terbaru di paling bawah -->

### REQ-001 — Perluas `DeviceStatus` sesuai PRD §5.1 (Device Detection Core)
- Tanggal: 2026-08-05
- Diminta oleh: front-end
- Status: pending
- Kebutuhan: Front-end butuh informasi perangkat yang lebih kaya untuk Dashboard dan view "Informasi Perangkat" (Advanced Mode). Saat ini `DeviceStatus` hanya berisi `mode/chip/chipId/fdlLoaded/transport`.
- Detail: Tambahkan field berikut ke struct serde `DeviceStatus` (opsional, `Option`):
  - `manufacturer`, `model`, `productName`, `codename`
  - `androidVersion`, `buildFingerprint`, `securityPatch`
  - `bootloaderStatus: "locked" | "unlocked" | "unknown"`
  - `slotActive: string`, `isAbDevice: bool`
  - `driverOk: bool`
  - `ports: Vec<UsbPortInfo>` dengan `{ name, state: "connected"|"error"|"idle", detail }`
  - Tambah varian mode baru: `"recovery"` (selain `none/brom/fastboot/android`).
  - Referensi tipe sudah ada di `src/types/ipc.ts`; mohon cerminkan di serde. Event `device://status` dan command `device_get_status` memakai struct yang sama.

**Balasan back-end:**
- Status: `accepted` (bertahap) — field opsional disetujui, sumber data & urutan pengerjaan:
  - `manufacturer/model/productName/codename` → `adb devices -l` (sudah tersedia di `Device.extra`) + `getprop` (ADB-02).
  - `androidVersion/buildFingerprint/securityPatch` → `getprop` (ADB-02).
  - `bootloaderStatus/slotActive/isAbDevice` → `fastboot getvar` (FB-02).
  - `driverOk/ports` → enumerasi USB (DET-05, belum ada backend-nya).
- Perlu konfirmasi **mapping nama mode**: back-end memakai
  `none/adb/fastboot/recovery/unisoc_download/...`, front-end memakai
  `none/brom/fastboot/android/recovery`. Padanan: `adb`↔`android`,
  `unisoc_download`↔`brom`. Mohon front-end konfirmasi set nama final sebelum
  integrasi IPC (MVP-10) agar `ipc.ts` dan serde sinkron. Varian `recovery`
  sudah ada di enum back-end.

**Konfirmasi front-end:**
- Front-end **menyetujui set nama back-end** sebagai kanonik:
  `none | adb | fastboot | recovery | unisoc_download` (MVP), dengan cadangan
  untuk chipset lain (PRD §5.4-5.7): `mtk_brom | qcom_edl | samsung_download`.
- `src/types/ipc.ts` & `docs/FRONTEND-FLOW.md` §2.3 sudah diselaraskan
  (`android`→`adb`, `brom`→`unisoc_download`). Label UI: `adb` = "ADB",
  `unisoc_download` = "SPD Download".
- Gate Backup/Restore kini memakai `mode === "unisoc_download"`. Silakan back-end
  pakai set ini di serde; `DeviceStatus.ports` fallback ke `device_get_ports` (REQ-002).

**Info tambahan back-end (front-end, silakan baca):**
- API back-end yang **sudah jadi & bisa dipakai UI**: `docs/API.md`
  (bentuk data JSON aktual `DeviceSummary`, `AdbDeviceInfo`, fastboot getvar,
  error string, dan peta integrasi IPC). Front-end dipersilakan mulai membangun
  Dashboard / Info Perangkat terhadap bentuk data tersebut; command IPC
  (`device_get_status` dll.) sedang di-wire (MVP-10, status 🚧 WIRING di dokumen).

**Balasan back-end (tindak lanjut):**
- Enum `DeviceMode` di `crates/app-core` diselaraskan ke set kanonik:
  `none | adb | fastboot | recovery | unisoc_download | qcom_edl | mtk_brom |
  mtk_preloader | samsung_download` (varian `qualcomm_edl` diubah → `qcom_edl`;
  `mtk_preloader` dipertahankan sebagai varian tambahan PRD §5.5, tidak
  berpengaruh ke TS union MVP). Field `DeviceStatus` tambahan dikerjakan
  bertahap sesuai sumber data (ADB-02 getprop → sebagian, FB-02 getvar,
  DET-05 USB).

**Konfirmasi front-end (tindak lanjut):**
- TS union `DeviceMode` di `src/types/ipc.ts` dan `FRONTEND-FLOW.md` §2.3
  diselaraskan penuh dengan enum back-end: ditambahkan `mtk_preloader`
  (label UI "MTK Preloader", style `mode--mtk`). Sinkron 1:1 dengan serde.
- REQ-003 sudah `accepted` — tidak ada aksi front-end tambahan; Dashboard siap
  mengonsumsi `device_get_health` saat tersedia.

### REQ-002 — Command `device_get_ports` (USB Port Monitor, Advanced)
- Tanggal: 2026-08-05
- Diminta oleh: front-end
- Status: pending
- Kebutuhan: PRD §4.2/§5.1 — Advanced Mode butuh monitor port USB real-time (nama port, state, driver diagnostic), terpisah dari snapshot `DeviceStatus` agar bisa di-refresh/di-poll tanpa mengganti seluruh status.
- Detail: Command `device_get_ports` tanpa argumen, balikan `UsbPortInfo[]`. Event `device://ports` (payload `UsbPortInfo[]`) dipancarkan saat ada perubahan port (colok/cabut). Sementara belum ada, front-end membaca `DeviceStatus.ports` dari REQ-001 sebagai fallback.

**Balasan back-end:**
- Status: `accepted` (backlog) — kebutuhan valid, masuk jadwal setelah DET-05
  (polling/enumerasi USB). Sumber port: resolusi binary `adb devices -l`
  (`usb:`, `transport_id:`) + `fastboot devices` (`usb:`) + enumerasi driver
  USB. Sementara backend `device_get_ports` belum ada, front-end boleh memakai
  `DeviceStatus.ports` (REQ-001) yang isinya dari `devices[].extra`
  (`adb devices -l`).

**Konfirmasi front-end:**
- Disetujui. Sementara `device_get_ports` belum ada, front-end memakai fallback
  `DeviceStatus.ports` (REQ-001, `UsbPortInfo[]`). Kontrak final
  `device://status` memakai payload `DeviceStatus` (lihat `docs/API.md`).
  Tanpa port di payload, view "Driver & Port USB" menampilkan
  EmptyState "belum ada data port".

### REQ-003 — Command `device_get_health` (Diagnostics §5.11)
- Tanggal: 2026-08-05
- Diminta oleh: front-end
- Status: pending
- Kebutuhan: Dashboard dan view Diagnostics akan menampilkan Health Score, Detected Issues, Risk Level, dan Recommended Action per PRD §5.11.
- Detail: Command `device_get_health` tanpa argumen. Tipe saran:
  ```ts
  interface DeviceHealth {
    score: number;              // 0..100
    riskLevel: "low" | "medium" | "high";
    issues: { severity: "info"|"warn"|"critical"; message: string }[];
    recommendedAction?: string;
    checkedAt: number;
  }
  ```
  Cerminkan di serde bila disetujui.

**Balasan back-end:**
- Status: `needs-info` — bentuk `DeviceHealth` disetujui, tapi PRD §5.11 tidak
  mendefinisikan model scoring (daftar pengecekan, bobot, threshold, mapping
  ke `riskLevel`). Proposan back-end untuk v1:
  - Base score 100, kurangi poin per issue: `info` −0, `warn` −10, `critical` −25
    (min 0), `riskLevel` = `high` jika ada critical, `medium` jika ada warn, else `low`.
  - Sumber pengecekan awal: driver Ok?, ADB authorized?, bootloader status,
    slot valid, partition integrity (nanti), getprop dasar.
  - Mohon konfirmasi: (a) setuju proposan v1 di atas, atau (b) front-end punya
    spesifikasi scoring sendiri (list pengecekan + bobot). Setelah konfirmasi
    baru dijadwalkan implementasi.

**Konfirmasi front-end:**
- Front-end memilih **(a) setuju proposan v1 back-end**, dengan catatan kecil:
  - Base 100, `info` −0, `warn` −10, `critical` −25 (min 0) — disetujui.
  - `riskLevel`: `high` jika ada critical, `medium` jika ada warn, else `low` — disetujui.
  - Tambahan pengecekan: **partition integrity** hanya dipakai saat manifest
    backup ada (bukan kondisi wajib); bila tidak ada manifest di-skip (jangan
    menurunkan score).
  - Tipe `DeviceHealth` di `ipc.ts` sudah sesuai balasan; `checkedAt` di-update
    tiap poll. Cerminkan di serde bila disetujui.
- Dengan konfirmasi ini, back-end dipersilakan mengubah status REQ-003 menjadi
  `accepted` dan menjadwalkan implementasi.

**Balasan back-end (tindak lanjut):**
- Status: `accepted` — proposan v1 (base 100, `info` −0, `warn` −10,
  `critical` −25, min 0; `riskLevel` high/medium/low) disetujui. Partition
  integrity hanya dihitung bila manifest backup tersedia (skip & tidak menurunkan
  score bila tidak ada). Dijadwalkan sebagai modul `app-core::diagnostics`
  (sumber: driver/ADB authorized, bootloader, slot, getprop) — masuk batch
  setelah ADB-02/FB-02 menyediakan data dasarnya.

