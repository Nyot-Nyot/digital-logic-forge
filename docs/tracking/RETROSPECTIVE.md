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

---

## Milestone M2: Connectivity & Semantic Frontend

- **Tanggal Selesai:** 23 November 2026
- **Status:** **ACCEPTED** (Seluruh 13 Task & Exit Criteria Lulus 100%)
- **Git Checkpoint:** [`be7175b`](https://github.com/Nyot-Nyot/digital-logic-forge/commit/be7175b)

### 1. Apa yang Berjalan Baik (What Went Well)?
- Algoritma **Disjoint Set Union (DSU) Deterministik** dengan pelacakan representatif kanonis minimum (`canonical_min`) berhasil menjamin penomoran `NetId` yang 100% konsisten dan independen dari urutan penggambaran kawat di canvas.
- Resolusi junction multi-cabang dan pasangan tunnel nirkabel (scope `Local` dan `Global`) bekerja secara mulus menyatukan terminal-terminal terhubung ke dalam netlist kanonis.
- Aturan validasi elektris (`NET-MULTI-DRIVER` dan `NET-UNDRIVEN`) berhasil mendeteksi konflik penggerak dan terminal menggantung dengan referensi visual lengkap (`primary` dan `related` `SourceRef`) untuk penyorotan di canvas UI.
- Konstruksi **Graf Ketergantungan Kombinasional (`DependencyGraph`)** berhasil membedakan elemen kombinasional murni dari elemen sekuensial (DFF): pin keluaran `q` bertindak sebagai sumber bebas (level 0) pada awal siklus $t$, sedangkan pin masukan `d` bertindak sebagai beban akhir (level $N$) pada batas clock $t+1$, sehingga memutus siklus kombinasional loop sekuensial (counter/register).
- Algoritma **Tarjan SCC** berhasil mengisolasi komponen terhubung kuat dan mendeteksi osilasi umpan balik kombinasional ilegal (`DEP-SCC-OSCILLATION`), membedakan wilayah DAG murni dari wilayah siklik.
- Algoritma **Levelization Kombinasional** menghitung kedalaman level ketergantungan dan menghasilkan urutan evaluasi linier statis (`EvaluationSchedule`) yang deterministik dan siap dikonsumsi langsung oleh compiler IR pada Milestone M3.
- Modul **Elaborasi Hierarki (`hierarchy.rs`)** berhasil memetakan port-to-pin binding sub-sirkuit dan mendeteksi siklus rekursi instansiasi tak berhingga (`HIER-RECURSION`) via 3-color DFS.
- Sebanyak 65 automated tests (58 unit tests + 7 integration tests) dan seluruh skrip verifikasi (`check.ps1` dan `test.ps1`) lulus 100% tanpa warning.

### 2. Apa Masalah / Tantangan yang Dihadapi (What Went Wrong)?
- Pada implementasi awal resolusi tunnel nirkabel, node tunnel diekstraksi ke DSU namun pin terminal kawat pada tunnel belum otomatis dipersatukan dengan partisi tunnel-nya. Masalah ini diselesaikan dengan menyatukan simpul pin tunnel ke simpul tunnel secara eksplisit saat fase resolusi.
- Clippy edisi terbaru memicu peringatan `collapsible_if` pada rantai pencocokan prefix `definition_id`, yang diselesaikan secara fungsional menggunakan combinator `Option::filter` dan `and_then`.

### 3. Temuan Kunci & Pembelajaran (Key Learnings / SPIKE Results)
- Penentuan root kanonis DSU berbasis ID objek terkecil di dalam equivalence class terbukti sangat ampuh menghasilkan identitas netlist yang sepenuhnya deterministik (KNF-01).
- Pemodelan elemen sekuensial sebagai simpul ganda (`SequentialSource` di awal siklus dan `SequentialSink` di akhir siklus) dalam graf ketergantungan kombinasional adalah solusi standar industri yang terbukti elegan dan akurat untuk memutus siklus sekuensial tanpa kehilangan kemampuan mendeteksi loop osilasi kombinasional murni. Hasil dicatat di [SPIKE-003].

### 4. Tindakan Perbaikan untuk Milestone Berikutnya (Action Items M3)
- Pada Milestone M3 (*First Compiled Simulation*), hasil `Netlist` dan `EvaluationSchedule` dari M2 akan dikonsumsi oleh pipeline kompilasi untuk alokasi memori runtime kontigu (slot state) dan penerjemahan ke instruksi IR ringkas.
- Membangun *Reference Evaluator* (interpreter baseline B0) sebagai *oracle* pembanding terhadap *Compiled Simulator* (B1) untuk pengujian diferensial bit-exact pada rangkaian kombinasional.

---

## Milestone M3: First Compiled Simulation

- **Tanggal Selesai:** 26 November 2026
- **Status:** **ACCEPTED** (Seluruh 12 Task & Exit Criteria Lulus 100%)
- **Git Checkpoint:** `HEAD` (Milestone M3 Completed)

### 1. Apa yang Berjalan Baik (What Went Well)?
- Arsitektur 3-representasi (*Authoring Document* -> *Compiled Artifact* -> *Runtime State*) berjalan bersih dari ujung ke ujung melalui `CircuitSnapshot` -> `CircuitCompiler::compile` -> `CompiledRuntime::from_artifact`.
- Resolusi **TBD-1** dan **TBD-2** di `[SPIKE-004]` menghasilkan desain dual-word (`values: Vec<u64>`, `unknowns: Vec<u64>`) yang mampu mengeksekusi operasi 64-bit dan logika 4-nilai konservatif secara paralel tanpa *bit-unpacking*.
- Pemisahan total antara **B0 (`ReferenceEvaluator`)** yang mengevaluasi graf komponen bit-demi-bit menggunakan tabel kebenaran eksplisit `BitState` dan **B1 (`CompiledRuntime`)** yang mengeksekusi IR datar berbasis array kontigu menghasilkan verifikasi diferensial independen yang sangat kuat.
- Seluruh workload uji non-trivial (4-bit Ripple-Carry Adder exhaustive 512 vektor, 8-bit ALU dengan 1.000 vektor fuzz 4-nilai, 8-bit Barrel Shifter dengan inspeksi probe `SourceMap`, dan 4-to-2 Priority Encoder) lulus 100% *bit-exact* antara B0 dan B1.
- Total 85 automated tests (74 unit tests + 11 integration tests) serta seluruh Godot GDExtension headless smoke test lulus 100% tanpa peringatan `clippy` maupun `rustfmt`.

### 2. Apa Masalah / Tantangan yang Dihadapi (What Went Wrong)?
- Tipe `Endpoint` pada model dokumen menggunakan koordinat `Point` (`f64`), sehingga tidak mengimplementasikan `Eq`/`Hash`/`Ord` untuk dipakai langsung sebagai key `HashMap`/`BTreeMap` pada helper `CircuitBuilder` di integration test; diselesaikan dengan `Vec<(Endpoint, JunctionId)>`.
- Lint Clippy `single_range_in_vec_init` mendeteksi `vec![0..1]` dan `Vec::from([0..1])`; diselesaikan dengan `std::iter::once(0..1).collect()` atau `.pop()`.

### 3. Temuan Kunci & Pembelajaran (Key Learnings / SPIKE Results)
- Formula *branchless* untuk MUX konservatif saat selektor bernilai `X`/`Z` (`unk = (d0_unk | d1_unk | (d0_val ^ d1_val)) & mask`, `val = (d0_val & d1_val) & !unk & mask`) terbukti 100% ekuivalen dengan tabel kebenaran `BitState::mux` pada seluruh uji diferensial.
- Penambahan primitif `dlf.builtin.const`, `dlf.builtin.slice`, dan `dlf.builtin.concat` pada `ComponentRegistry` sangat mempermudah konstruksi *datapath* multi-bit yang realistis.

### 4. Tindakan Perbaikan untuk Milestone Berikutnya (Action Items M4)
- Pada **M4 (Stateful Simulation)**, tutup **TBD-4** (kebijakan *state* awal), **TBD-5** (prioritas *reset* dan *enable* register), **TBD-6** (mode baca bawaan RAM), dan format *image* ROM pada fase Pre-Sprint / SPIKE.
- Ekspansi `CompiledOp` dengan `DffNext`, `MemRead`, dan `MemWrite`, serta tambahkan `StateSlot` (current/next state untuk *simultaneous commit*) dan `MemoryRegion` pada `CompiledArtifact` dan `CompiledRuntime`.
