# Digital Logic Forge (DLF) — Milestone Retrospectives

Dokumen ini mencatat evaluasi akhir (*Post-Sprint Retrospective*) dari setiap milestone yang telah diselesaikan.

---

## Milestone M0: Implementation Baseline

- **Tanggal Selesai:** 18 November 2026
- **Status:** **ACCEPTED** (Semua 8 Exit Criteria Lulus)
- **Git Checkpoint:** [`6135ffe`](https://github.com/Nyot-Nyot/digital-logic-forge/commit/6135ffe)

### 1. Apa yang Berjalan Baik (What Went Well)?
- Cargo workspace Rust edisi 2024 dan Godot 4.7.2 terhubung mulus melalui `godot-rust 0.5.5` dengan feature `api-4-7`.
- Pemisahan thread antara main thread UI dan worker Rust terbukti bersih: `dlf-simulation-worker` dapat menjalankan ratusan juta siklus tanpa menyentuh scene tree Godot.
- Snapshot polling non-blocking berjalan stabil tanpa menyebabkan data race atau frame freeze.
- Skrip otomasi PowerShell (`check.ps1`, `test.ps1`, `build_extension.ps1`) berhasil memvalidasi seluruh lapisan test (Rust unit test + Godot headless test) dalam satu perintah.

### 2. Apa Masalah / Tantangan yang Dihadapi (What Went Wrong)?
- Pada canvas prototype awal, terjadi drop FPS saat panning dan zoom out pada 1.200 komponen akibat tidak adanya frustum culling dan overhead *text shaping* `draw_string` di CPU.
- GitHub menolak push perdana karena proteksi privasi email, yang langsung diselesaikan dengan mengonfigurasi email noreply resmi GitHub.

### 3. Temuan Kunci & Pembelajaran (Key Learnings / SPIKE Results)
- Di Godot 2D immediate mode, memanggil `draw_string` dan `draw_circle` dalam jumlah ribuan setiap frame adalah anti-pattern performa.
- Penggunaan **Frustum Culling** + **LOD (Level of Detail)** + **Batching (`draw_multiline`)** wajib dipertahankan untuk arsitektur editor canvas pada Milestone M5. Hasil eksperimen dicatat di [SPIKE-001].

### 4. Tindakan Perbaikan untuk Milestone Berikutnya (Action Items)
- Terapkan alur **3-Phase Milestone Loop** (Pre-Sprint -> Daily Loop -> Post-Sprint) secara konsisten mulai M1.
- Setiap task penguraian di M1 harus berfokus pada *headless-first*, memastikan serialisasi dan validasi struktural dapat diuji otomatis secara cepat via `cargo test`.
