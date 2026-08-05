# CONTRIBUTING.md — Alur Pengembangan & Version Control (VCA)

Repo ini mengikuti **Trunk-Based Development + Conventional Commits**, dengan
PR sebagai satu-satunya jalur menuju `main`.

## 1. Alur kerja (Trunk-Based Development)

```
main  ●──────●─────────────────●──────►  selalu releasable
       \      \               /
feature  ●────●─────────────●  branch pendek (hidup max 1-2 hari)
```

- **`main` selalu hijau & siap rilis.** Tidak ada kerja langsung di `main` —
  semua lewat Pull Request.
- **Branch fitur pendek** dari `main`, namanya deskriptif:
  - `feat/<inisial>-<nama>` → `feat/ab-device-detect`
  - `fix/<nama>` → `fix/flash-oversize`
  - `docs/<nama>` → `docs/api-contract`
  - `chore/<nama>` → `chore/github-setup`
- Setelah PR di-merge, **hapus branch** (jangan menumpuk).

## 2. Conventional Commits

Format: `<type>(<scope>): <deskripsi>`

```
feat(device): tambah deteksi mode unisoc_download
fix(fastboot): perbaiki parse getvar dengan colon di key
docs(api): perbarui kontrak device_get_status
refactor(app-core): pisahkan manager dari adapter
test(adb): tambah kasus unauthorized state
chore: tambah GitHub Actions CI
```

- Type: `feat` `fix` `perf` `docs` `refactor` `chore` `build` `test` `ci`.
- Deskripsi pakai **Bahasa Indonesia**, imperative, tanpa titik di akhir.
- Scope opsional tapi dianjurkan (nama crate/view).
- **`feat`/`fix` otomatis menaikkan versi & changelog** via release-please.

## 3. Pull Request

1. Branch dari `main`, commit memakai Conventional Commits.
2. Push → buka PR → isi template (jenis perubahan, scope tim, verifikasi).
3. **Wajib** memenuhi checklist PR template (scope tim & verifikasi).
4. CI `build-test` harus hijau: `npm run build`, `cargo test --workspace`,
   `cargo fmt`, `cargo clippy`.
5. Merge memakai **Squash** (linear history di `main`).

## 4. Branch protection `main` (GitHub)

- Wajib status check `build-test` (strict).
- Wajib review PR ≥ 1 (enforce admins).
- **Linear history** — tidak boleh merge commit.
- Tidak ada force-push / hapus branch `main`.
- Conversation harus resolved sebelum merge.

## 5. API contract — single source of truth `docs/API.md`

> **Back-end wajib:** setiap menambah/mengubah command, event, atau tipe di
> Rust, **update `docs/API.md` lebih dulu**, baru implementasi di `src-tauri`.
> PR yang mengubah API tanpa update dokumen akan ditolak CI/review.

> **Front-end:** salin interfaces dari `docs/API.md` ke `src/types/`, dan pakai
> Mock Data di service layer (`src/lib/mock.ts`) — tidak menunggu back-end.

Ketidaksesuaian/masukan API dibahas lewat **Issue**.

## 6. Rilis (release-please)

- Commit `feat`/`fix` di `main` memicu release-please → naik versi semantik,
  update changelog, buat GitHub Release.
- Tag otomatis: `v0.1.x`. Binary Windows (.exe NSIS) di-build di workflow
  `release.yml` dan dilampirkan ke Release.
- **Build production** (`build.yml`) berjalan **setiap push ke `main`** (bukan
  branch lain): hasil binary di-upload sebagai artifact di tab Actions
  (`spd-backup-tool-windows`), tersimpan 14 hari.
