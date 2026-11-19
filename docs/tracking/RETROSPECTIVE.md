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

---

## Milestone M1: Headless Circuit Model

- **Tanggal Selesai:** 19 November 2026
- **Status:** **ACCEPTED** (Seluruh 14 Task & Exit Criteria Lulus 100%)
- **Git Checkpoint:** [`afdc71a`](https://github.com/Nyot-Nyot/digital-logic-forge/commit/afdc71a)

### 1. Apa yang Berjalan Baik (What Went Well)?
- Model sirkuit *headless* berhasil dibangun murni di `dlf-core` tanpa dependensi Godot.
- Sistem **Stable ID 64-bit** (`u64` dengan newtype `InstanceId`, `PinId`, `WireId`, `CircuitId`, `JunctionId`, `TunnelId`) terbukti deterministik, tidak bergantung koordinat grafis atau urutan array, dan mempertahankan identitas saat penyimpanan/pemuatan.
- Kontrak pin dan lebar bus aman (`SignalWidth` 1..=64 bit) memvalidasi batas secara otomatis menggunakan `#[serde(try_from = "u8")]`.
- Format JSON skema v1 berhasil diuji round-trip bit-exact melalui berkas golden fixture `tests/fixtures/simple_circuit.json`.
- Seluruh validator struktural (pin masukan wajib yang mengambang `CONN-PIN-UNCONNECTED`, kawat menggantung `CONN-WIRE-DANGLING`, dan ketidakcocokan lebar bus `CONN-WIDTH-MISMATCH`) bekerja dengan akurat dan memetakan referensi visual (`SourceRef`) untuk penyorotan canvas visual.
- Sebanyak 31 unit & integration test lulus 100% dengan waktu eksekusi super cepat (< 0.3 detik).

### 2. Apa Masalah / Tantangan yang Dihadapi (What Went Wrong)?
- Rust edisi 2024 memperkenalkan `gen` sebagai kata kunci yang dicadangkan (*reserved keyword*), sehingga nama variabel pengujian `let mut gen` menghasilkan compiler error dan langsung diperbaiki menjadi nama deskriptif `generator`.
- Clippy memicu peringatan `too_many_arguments` pada `ComponentDefinition::new`, yang diselesaikan secara elegan dengan menerapkan **Builder Pattern** (`ComponentDefinitionBuilder`).
- Serde secara bawaan tidak mengizinkan *internally tagged enum* (`#[serde(tag = "...")]`) membungkus newtype variant primitif tanpa nama field. Hal ini diselesaikan dengan memberi named field struct `{ id: WireId }` pada `SourceRef`.

### 3. Temuan Kunci & Pembelajaran (Key Learnings / SPIKE Results)
- Pemisahan ketat antara representasi pengeditan (*Authoring Document*) dan netlist/kompilasi terbukti sangat menyederhanakan arsitektur dokumen. Dokumen JSON hanya peduli pada komponen, endpoint kawat, junction, dan port.
- Keputusan serialisasi `Rotation` mendukung fleksibilitas integer derajat (0, 90, 180, 270) maupun string kuadran. Hasil dicatat di [SPIKE-002].

### 4. Tindakan Perbaikan untuk Milestone Berikutnya (Action Items M2)
- Di Milestone M2 (*Connectivity & Semantic Frontend*), fokus berpindah ke pembentukan graf net listrik menggunakan algoritma **Disjoint Set Union (DSU)** dan deteksi siklus umpan balik via algoritma **Tarjan SCC**.
- Persiapkan fixture sirkuit dengan berbagai topologi kawat rumit (percabangan jamak, junction berantai, splitter/merger bus, dan tunnel nirkabel).
