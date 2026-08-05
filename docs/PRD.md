# PRD — Android Service Software

**Versi:** 0.1
**Platform:** Windows 10/11
**Tipe Produk:** Desktop Android Service & Device Maintenance Tool
**Model Bisnis:** Freemium
**Target Pengguna:** Pengguna umum, konter servis, dan teknisi profesional

## 1. Ringkasan Produk

Android Service Software adalah aplikasi desktop untuk membantu proses diagnosis, maintenance, backup, pemulihan perangkat, flashing, dan perbaikan perangkat Android melalui berbagai mode koneksi dan protokol chipset.

Aplikasi mendukung arsitektur modular berbasis chipset agar dukungan terhadap Qualcomm, MediaTek, Unisoc/Spreadtrum, dan Samsung dapat dikembangkan secara independen tanpa mengubah inti aplikasi.

Aplikasi menyediakan dua tingkat antarmuka:

* **Basic Mode:** alur sederhana, aman, dan dipandu untuk pengguna umum.
* **Advanced Mode:** akses teknis yang lebih lengkap untuk teknisi profesional.

## 2. Tujuan Produk

### Tujuan Utama

1. Menyatukan berbagai fungsi service Android dalam satu aplikasi Windows.
2. Menyediakan deteksi otomatis terhadap perangkat, mode USB, dan chipset.
3. Mengurangi risiko kesalahan flashing melalui validasi perangkat dan partisi.
4. Menyediakan sistem backup, restore, logging, dan pemulihan yang terstruktur.
5. Mendukung pengembangan fitur baru melalui sistem plugin atau modul chipset.
6. Menyediakan fitur gratis yang berguna serta fitur premium untuk kebutuhan teknisi tingkat lanjut.

### Indikator Keberhasilan

* Perangkat dapat terdeteksi secara otomatis.
* Mode ADB, Fastboot, dan mode chipset dapat dibedakan dengan benar.
* Operasi baca/tulis partisi memiliki validasi ukuran dan checksum.
* Setiap proses menghasilkan log yang dapat diekspor.
* Kegagalan operasi dapat dijelaskan dengan kode error dan rekomendasi tindakan.
* Modul chipset dapat ditambahkan tanpa mengubah keseluruhan aplikasi.

## 3. Target Pengguna

### 3.1 Basic User

Karakteristik:

* Memiliki pengetahuan teknis terbatas.
* Membutuhkan proses yang sederhana dan aman.
* Menggunakan aplikasi untuk maintenance, backup, informasi perangkat, dan pemulihan dasar.

Kebutuhan:

* Tombol tindakan yang mudah dipahami.
* Wizard langkah demi langkah.
* Peringatan risiko yang jelas.
* Validasi otomatis sebelum proses berjalan.
* Tidak menampilkan parameter teknis secara default.

### 3.2 Advanced Technician

Karakteristik:

* Memahami ADB, Fastboot, partisi Android, firmware, bootloader, dan mode chipset.
* Membutuhkan kontrol parameter serta akses log teknis.

Kebutuhan:

* Pemilihan partisi manual.
* Akses command console.
* Informasi USB dan port secara detail.
* Pengaturan loader, alamat eksekusi, dan parameter protokol.
* Log komunikasi tingkat rendah.
* Kemampuan membuat dan menjalankan workflow service.

## 4. Mode Aplikasi

### 4.1 Basic Mode

Fitur utama:

* Auto Detect Device
* Device Information
* Device Health Check
* Reboot Device
* Backup dasar
* Restore backup yang tervalidasi
* Flash file yang kompatibel
* Recovery Assistant
* Debloater
* Export laporan perangkat

Batasan:

* Parameter protokol disembunyikan.
* Pemilihan partisi manual dibatasi.
* Operasi berisiko memerlukan konfirmasi tambahan.
* Hanya file firmware atau paket yang lolos validasi yang dapat digunakan.

### 4.2 Advanced Mode

Fitur utama:

* ADB Command Console
* Fastboot Command Console
* Pemilihan partisi manual
* Read Partition
* Write Partition
* Erase Partition
* Verify Partition
* Raw Backup
* Loader Configuration
* Protocol Log
* USB Port Monitor
* Custom Service Workflow

Batasan:

* Memerlukan aktivasi Advanced Mode.
* Menampilkan peringatan risiko sebelum operasi destruktif.
* Semua operasi dicatat pada audit log.

## 5. Fitur Fungsional

## 5.1 Device Detection Core

Fungsi:

* Mendeteksi perangkat USB secara otomatis.
* Mendeteksi status koneksi perangkat.
* Mengidentifikasi mode perangkat:

  * ADB
  * Fastboot
  * Recovery
  * Qualcomm EDL
  * MediaTek BROM
  * MediaTek Preloader
  * Unisoc/Spreadtrum Download Mode
  * Samsung Download Mode
* Mendeteksi informasi perangkat jika tersedia:

  * Manufacturer
  * Model
  * Product name
  * Device codename
  * Android version
  * Build fingerprint
  * Security patch
  * Chipset
  * Bootloader status
  * Slot aktif
  * A/B partition status
* Menampilkan status driver dan port USB.

Output:

* Device Summary
* Connection Status
* Detected Protocol
* Chipset Information
* Driver Diagnostic Result

## 5.2 ADB Module

Fungsi:

* Daftar perangkat ADB.
* Menampilkan informasi perangkat.
* Reboot ke:

  * System
  * Recovery
  * Bootloader/Fastboot
  * Download Mode jika didukung perangkat
* Install APK.
* Uninstall aplikasi pengguna.
* Debloater.
* Backup data yang didukung.
* Mengambil log perangkat.
* Screenshot dan screen recording jika tersedia.
* Menjalankan perintah ADB pada Advanced Mode.

## 5.3 Fastboot Module

Fungsi:

* Deteksi perangkat Fastboot.
* Menampilkan:

  * Product
  * Serial number
  * Slot aktif
  * Bootloader state
  * Anti-rollback information jika tersedia
* Flash partisi.
* Boot image sementara jika didukung.
* Backup informasi partisi yang tersedia.
* Mengubah slot aktif.
* Reboot ke system, recovery, atau bootloader.
* Menjalankan perintah Fastboot pada Advanced Mode.

## 5.4 Qualcomm Module

Target protokol:

* EDL 9008
* Sahara
* Firehose

Fungsi fase lanjutan:

* Deteksi Qualcomm EDL.
* Identifikasi port Qualcomm.
* Memuat programmer/Firehose yang sesuai.
* Membaca informasi perangkat.
* Membaca tabel partisi.
* Read partition.
* Write partition.
* Erase partition yang diizinkan.
* Backup dan restore partisi yang didukung.
* Firmware flashing berbasis paket yang kompatibel.

Catatan:

* Dukungan bergantung pada kompatibilitas programmer, konfigurasi perangkat, dan kebijakan keamanan perangkat.

## 5.5 MediaTek Module

Target protokol:

* BROM
* Preloader

Fungsi fase lanjutan:

* Deteksi MediaTek USB Port.
* Identifikasi chipset.
* Memuat Download Agent yang kompatibel.
* Membaca informasi storage.
* Membaca tabel partisi.
* Read partition.
* Write partition.
* Backup dan restore.
* Firmware flashing berbasis scatter atau paket yang didukung.
* Recovery workflow untuk perangkat yang kompatibel.

## 5.6 Unisoc/Spreadtrum Module

Target protokol:

* SPD Download Mode
* BootROM
* FDL1
* FDL2

Fungsi:

* Deteksi port Unisoc/Spreadtrum.
* Memuat FDL1 dan FDL2.
* Konfigurasi alamat loader.
* Membaca daftar partisi.
* Membaca ukuran partisi.
* Read partition.
* Write partition.
* Verify partition.
* Backup partisi.
* Restore partisi.
* Menampilkan log komunikasi loader.
* Menjalankan workflow flashing yang tervalidasi.

## 5.7 Samsung Module

Target:

* Download Mode
* Protokol flashing Samsung yang kompatibel

Fungsi fase lanjutan:

* Deteksi Samsung Download Mode.
* Membaca informasi perangkat yang tersedia.
* Validasi model dan firmware.
* Flash paket firmware yang kompatibel.
* Menampilkan status proses dan hasil validasi.

## 5.8 Flashing & Recovery Module

Fungsi:

* Flash satu partisi.
* Flash beberapa partisi.
* Flash paket firmware.
* Validasi model perangkat.
* Validasi codename.
* Validasi ukuran partisi.
* Validasi checksum file.
* Validasi slot A/B.
* Backup otomatis sebelum operasi.
* Restore partisi.
* Recovery Assistant.
* Bootloop Diagnostic.
* Softbrick Recovery Workflow.
* Resume atau retry jika protokol mendukung.

Partisi awal yang didukung:

* boot
* vendor_boot
* init_boot
* recovery
* vbmeta
* dtbo
* vendor
* persist

Catatan:

Dukungan partisi harus bergantung pada tabel partisi dan profil perangkat, bukan daftar statis.

## 5.9 Partition Service

Fungsi:

* List partition.
* Check partition.
* Get partition size.
* Read partition.
* Write partition.
* Verify partition.
* Compare image.
* Generate checksum.
* Backup metadata partisi.
* Menampilkan informasi slot A/B.

Perlindungan:

* Validasi ukuran file.
* Validasi target partisi.
* Validasi model.
* Konfirmasi operasi.
* Backup sebelum overwrite.
* Pencegahan penulisan ke partisi yang tidak kompatibel.

## 5.10 Backup & Restore

Fungsi:

* Backup partisi terpilih.
* Backup profil perangkat.
* Backup informasi sistem.
* Backup konfigurasi aplikasi.
* Restore backup yang kompatibel.
* Verifikasi integritas backup.
* Riwayat backup.
* Export backup manifest.

Format backup:

```text
backup/
├── manifest.json
├── device-info.json
├── partitions/
│   ├── boot_a.img
│   ├── boot_b.img
│   └── vendor_boot_a.img
├── checksums/
│   └── sha256.txt
└── logs/
    └── operation.log
```

## 5.11 Diagnostics Module

Fungsi:

* Device Health Check.
* Storage information.
* Boot status.
* Android system information.
* Baseband status.
* Wi-Fi status.
* Bluetooth status.
* Sensor status jika tersedia melalui sistem.
* Camera service diagnostic.
* Partition integrity check.
* Driver diagnostic.
* USB connection diagnostic.

Output:

* Health Score
* Detected Issues
* Risk Level
* Recommended Action
* Exportable Service Report

## 5.12 Maintenance Module

Fungsi:

* Debloater.
* Uninstall aplikasi pengguna.
* Disable aplikasi yang dipilih.
* Daftar aplikasi sistem.
* Backup daftar aplikasi.
* Cleanup file sementara jika didukung.
* Malware/adware scan berbasis aturan.
* Backup data pengguna melalui metode yang didukung.
* Export laporan maintenance.

## 5.13 Job & Logging System

Fungsi:

* Job queue.
* Progress bar.
* Estimated stage.
* Current operation.
* Success/failure status.
* Retry operation.
* Cancel operation jika aman.
* Detailed log.
* Raw protocol log pada Advanced Mode.
* Export log ke TXT atau JSON.
* Operation history.

Status job:

```text
QUEUED
DETECTING_DEVICE
VALIDATING
PREPARING
BACKING_UP
EXECUTING
VERIFYING
COMPLETED
FAILED
CANCELLED
```

## 5.14 Firmware & Package Manager

Fungsi:

* Import firmware.
* Scan folder firmware.
* Parse metadata firmware.
* Menampilkan model dan codename.
* Validasi file.
* Validasi checksum.
* Menentukan modul/protokol yang sesuai.
* Menyimpan firmware profile.
* Menolak paket yang tidak kompatibel.

## 6. Model Bisnis Freemium

### Free Tier

Fitur:

* Device Detection
* Device Information
* ADB Toolkit dasar
* Fastboot Toolkit dasar
* Reboot tools
* Device Health Check
* Debloater dasar
* Backup profil perangkat
* Log dan laporan dasar
* Dukungan modul chipset terbatas

### Premium Tier

Fitur:

* Advanced Mode
* Multi-partition flashing
* Advanced backup dan restore
* Firmware package manager
* Batch operation
* Custom workflow
* Detailed protocol logging
* Advanced diagnostics
* Priority update
* Premium chipset modules
* Service report branding

### Prinsip Monetisasi

* Fitur keselamatan dan informasi dasar tetap gratis.
* Tidak mengunci akses pengguna terhadap backup pribadi dan log dasar.
* Premium digunakan untuk fitur produktivitas, otomasi, workflow teknisi, dan kemampuan tingkat lanjut.

## 7. Arsitektur Sistem

```text
Android Service Software
│
├── Desktop UI
│   ├── Basic Mode
│   ├── Advanced Mode
│   ├── Device Dashboard
│   ├── Operation Center
│   └── Log Viewer
│
├── Application Core
│   ├── Device Manager
│   ├── Module Manager
│   ├── Job Manager
│   ├── Validation Engine
│   ├── Backup Manager
│   ├── Firmware Manager
│   └── License Manager
│
├── Protocol Layer
│   ├── ADB Adapter
│   ├── Fastboot Adapter
│   ├── Qualcomm Adapter
│   ├── MediaTek Adapter
│   ├── Unisoc Adapter
│   └── Samsung Adapter
│
├── Service Layer
│   ├── Flash Service
│   ├── Partition Service
│   ├── Recovery Service
│   ├── Diagnostic Service
│   └── Maintenance Service
│
├── Safety Layer
│   ├── Device Validation
│   ├── File Validation
│   ├── Partition Validation
│   ├── Backup Protection
│   ├── Audit Log
│   └── Risk Confirmation
│
└── Data Layer
    ├── SQLite
    ├── Local File Storage
    ├── Firmware Profiles
    ├── Device Profiles
    └── Operation History
```

## 8. Rekomendasi Teknologi

### Desktop Application

* Tauri 2
* Rust
* React
* TypeScript
* Vite
* Tailwind CSS

### Backend Core

* Rust
* Tokio untuk asynchronous task.
* Serial/USB communication abstraction.
* Plugin interface berbasis Rust.
* Command adapter untuk tool eksternal.

### Local Database

* SQLite

Data yang disimpan:

* Device history
* Operation history
* Backup manifest
* Firmware profile
* User preferences
* Module configuration
* License status

### Tool Adapter

Tool eksternal dapat dijalankan melalui adapter terisolasi:

```text
tools/
├── adb/
├── fastboot/
├── unisoc/
├── qualcomm/
├── mediatek/
└── samsung/
```

Aplikasi tidak bergantung langsung pada satu executable. Setiap tool memiliki adapter yang menyediakan:

* Detect
* Connect
* Execute
* Parse output
* Convert error
* Generate structured result

## 9. Prioritas Pengembangan

### MVP v1

Fokus:

* Windows 10/11
* Basic Mode
* Advanced Mode dasar
* Device Detection
* ADB
* Fastboot
* Device Information
* Job Manager
* Log System
* Backup Manifest
* Firmware Validation
* Unisoc/Spreadtrum Module awal
* Read/write/verify partisi yang didukung
* Device Health Report

### Phase 2

Fokus:

* Qualcomm EDL/Firehose
* MediaTek BROM/Preloader
* Samsung Download Mode
* Firmware Package Manager
* Multi-device support
* Batch operation
* Advanced diagnostics
* Custom workflow

### Phase 3

Fokus:

* Plugin marketplace
* Cloud firmware catalog
* Online device profile database
* Remote technical support
* Team account
* Service-center management
* Advanced automation

## 10. Kebutuhan Non-Fungsional

### Performance

* Startup aplikasi maksimal 5 detik pada perangkat Windows standar.
* Deteksi perubahan USB maksimal 2 detik.
* UI tetap responsif selama operasi flashing.
* Operasi berat dijalankan pada background task.

### Reliability

* Tidak boleh menulis partisi sebelum validasi selesai.
* Setiap operasi memiliki status yang dapat dilacak.
* Log harus tetap disimpan ketika operasi gagal.
* Aplikasi harus menangani perangkat terputus secara aman.

### Security

* File firmware harus diperiksa.
* Modul premium harus memiliki verifikasi lisensi.
* License validation tidak boleh menghambat fitur keselamatan dasar.
* Audit log harus mencatat operasi sensitif.
* Data backup pengguna harus disimpan secara lokal secara default.

### Compatibility

* Windows 10 64-bit.
* Windows 11 64-bit.
* Dukungan driver USB melalui pemeriksaan dan panduan instalasi.
* Dukungan chipset ditentukan oleh modul.

### Usability

* Basic Mode menggunakan wizard.
* Advanced Mode menyediakan parameter teknis.
* Setiap operasi berisiko memiliki peringatan.
* Error harus menggunakan bahasa yang dapat dipahami dan menyertakan detail teknis.

## 11. Risiko Teknis

| Risiko                                | Dampak        | Mitigasi                                            |
| ------------------------------------- | ------------- | --------------------------------------------------- |
| Perbedaan protokol antar chipset      | Tinggi        | Arsitektur adapter dan plugin                       |
| Driver Windows tidak terpasang        | Tinggi        | Driver diagnostic dan panduan instalasi             |
| Firmware tidak sesuai                 | Sangat tinggi | Validasi model, codename, ukuran, dan checksum      |
| Perangkat terputus saat flashing      | Sangat tinggi | Job recovery, retry, dan backup otomatis            |
| Perbedaan layout partisi              | Tinggi        | Baca tabel partisi langsung dari perangkat          |
| Tool eksternal berubah                | Sedang        | Adapter terisolasi dan version manager              |
| Antivirus mendeteksi tool service     | Sedang        | Binary signing dan distribusi resmi                 |
| Kesalahan pengguna                    | Sangat tinggi | Basic Mode, wizard, validasi, dan confirmation gate |
| Dukungan banyak chipset terlalu luas  | Tinggi        | Pengembangan bertahap berdasarkan modul             |
| Ketergantungan pada loader/programmer | Tinggi        | Device profile dan compatibility database           |

## 12. Out of Scope untuk MVP

Fitur berikut tidak termasuk MVP:

* Dukungan semua merek Android.
* Dukungan semua model perangkat.
* Qualcomm EDL lengkap.
* MediaTek BROM lengkap.
* Samsung flashing lengkap.
* Cloud firmware download.
* Multi-device batch service.
* Remote service.
* Plugin marketplace.
* Custom workflow visual.
* Integrasi service-center.
* Fitur yang menghapus atau melewati perlindungan akun/perangkat tanpa verifikasi otorisasi.
* Perubahan identitas perangkat atau fungsi radio yang tidak sesuai dengan aturan dan otorisasi yang berlaku.

## 13. Keputusan Produk

Produk dikembangkan sebagai aplikasi desktop Windows dengan arsitektur modular berbasis chipset.

MVP tidak mengejar dukungan universal secara langsung. Dukungan universal dicapai melalui sistem plugin dan adapter, dengan implementasi chipset dilakukan secara bertahap.

Prioritas teknis awal:

1. Device Detection Core
2. ADB Module
3. Fastboot Module
4. Job & Logging System
5. Validation & Safety Layer
6. Backup & Restore
7. Unisoc/Spreadtrum Module
8. Qualcomm Module
9. MediaTek Module
10. Samsung Module
