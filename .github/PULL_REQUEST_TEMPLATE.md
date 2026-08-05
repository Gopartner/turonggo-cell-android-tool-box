## Deskripsi

Jelaskan perubahan ini secara singkat. Hubungkan dengan issue/request bila ada
(contoh: "Closes #12", "REQ-001").

## Jenis perubahan

- [ ] feat: fitur baru
- [ ] fix: perbaikan bug
- [ ] docs: dokumentasi
- [ ] refactor: perubahan tanpa mengubah perilaku
- [ ] test: penambahan/penyesuaian test
- [ ] chore: tugas pendukung (build, dependency, dll.)

## Scope tim

- [ ] Front-end (`src/`, `docs/FRONTEND-*.md`)
- [ ] Back-end (`crates/`, `src-tauri/`, `docs/BACKEND.md`)
- [ ] Umum (`AGENTS.md`, `README.md`, `docs/REQUESTS.md`, `docs/`)

## API contract (`docs/API.md`)

- [ ] Tidak ada perubahan command/event/tipe
- [ ] Ada perubahan dan **sudah update `docs/API.md` lebih dulu** (wajib back-end)
- [ ] Ada perubahan tapi belum sempat update — **jangan merge sebelum update**

## Verifikasi

- [ ] `npm run build` lolos (wajib untuk perubahan `src/`)
- [ ] `cargo test --workspace` lolos (wajib untuk perubahan Rust)
- [ ] `cargo fmt --all -- --check` dan `cargo clippy -- -D warnings` bersih

## Checklist

- [ ] Tidak menambahkan dependency tanpa alasan tertulis
- [ ] Tidak ada komentar berbahasa selain Indonesia (konvensi repo)
- [ ] Perubahan sesuai scope tim (lihat AGENTS.md "Aturan per tim")
