# Digital Logic Forge (DLF) — Decision Log & SPIKE Records

Dokumen ini mencatat seluruh keputusan teknis arsitektur, hasil eksperimen (*SPIKE*), dan resolusi *trade-off* yang diambil sepanjang pengembangan proyek.

---

## Format Entri Keputusan / SPIKE

```markdown
### [SPIKE/ADR-xxx] Judul Keputusan / Eksperimen
- **Tanggal:** YYYY-MM-DD
- **Milestone Terkait:** M0 / M1 / dst.
- **Konteks & Masalah:** Mengapa keputusan ini perlu diambil?
- **Opsi yang Dieksplorasi:**
  1. Opsi A (Kelebihan / Kekurangan)
  2. Opsi B (Kelebihan / Kekurangan)
- **Keputusan Terpilih:** Opsi yang ditetapkan dan alasannya.
- **Konsekuensi & Dampak:** Dampak pada performa, codebase, atau semantik.
```

---

## Log Keputusan & SPIKE

### [ADR-000] Baseline Stack Teknologi & Pemisahan Bahasa
- **Tanggal:** 2026-11-18
- **Milestone:** M0
- **Konteks:** Menentukan platform target, bahasa inti, framework UI, dan binding GDExtension sesuai Tabel 3.8 proposal.
- **Keputusan:** Menggunakan Rust stable (edisi 2024) untuk `dlf-core` tanpa dependensi Godot; Godot 4.7.2 stable untuk layer UI; `godot-rust 0.5.5` dengan feature `api-4-7` untuk adapter GDExtension `dlf-godot`.
- **Dampak:** Eksekusi kompilasi dan testing logika murni dapat berjalan super cepat secara headless tanpa memuat engine Godot.

### [SPIKE-001] Optimasi Viewport Culling, LOD, dan Wire Batching pada Canvas Prototype
- **Tanggal:** 2026-11-18
- **Milestone:** M0
- **Konteks:** Canvas prototype mengalami drop FPS saat zoom out dan panning pada beban 1.200 komponen sintetis.
- **Akar Masalah:**
  1. `draw_string` memicu *text shaping* CPU untuk 1.200 komponen per frame saat zoom out.
  2. 2.400 lingkaran pin (`draw_circle`) melakukan tessellation poligon terus-menerus.
  3. 1.200 pemanggilan `draw_line` terpisah tanpa batching GPU.
  4. Komponen offscreen tetap diproses tanpa *viewport culling*.
- **Keputusan:**
  1. Implementasi Frustum/Viewport Culling via `get_visible_world_rect()`. Komponen di luar layar di-*skip*.
  2. Level of Detail (LOD): Nonaktifkan teks saat `zoom < 0.65x`, nonaktifkan lingkaran pin saat `zoom < 0.35x`.
  3. Wire Batching: Kumpulkan seluruh segmen kawat ke `PackedVector2Array` dan gambar dalam 1 pemanggilan `draw_multiline()`.
- **Dampak:** Rendering stabil 60 FPS saat pan dan zoom out, latensi hit-testing tetap konsisten di ~2 µs.

### [SPIKE-002] Spesifikasi Skema Dokumen JSON & Kebijakan Stable ID 64-bit
- **Tanggal:** 2026-11-19
- **Milestone:** M1 (Headless Circuit Model)
- **Konteks:** Menentukan format serialisasi dokumen rangkaian (*Authoring Document*), versi skema, dan mekanisme pembuatan identitas objek stabil (*Stable ID*) sesuai proposal Bab 3.10.1 & Tabel 3.21.
- **Opsi yang Dieksplorasi:**
  1. *UUID string 128-bit*: Sangat unik global, tetapi overhead memori tinggi, parsing lambat, dan menyulitkan pengurutan deterministik kanonis.
  2. *Coordinate-based ID (x, y)*: Bergantung pada posisi visual; langsung rusak jika komponen digeser atau dipindahkan.
  3. *Sequential 64-bit unsigned integer (`u64`)*: Dikelola oleh penghitung terpusat di tingkat `Project` (`next_id`), efisien dalam memori, deterministik saat diurutkan, dan stabil melintasi pemindahan objek serta *save/load*.
- **Keputusan Terpilih:**
  1. **Format File:** Berkas JSON berbasis teks dengan field akar `schema_version: u32` (versi awal = 1) untuk deteksi ketidakcocokan versi dan dukungan migrasi di masa depan.
  2. **Stable ID:** Bilangan bulat tak bertanda 64-bit (`u64`) yang dihasilkan secara sekuensial melalui generator deterministik `Project::allocate_id()`. ID tidak dipakai ulang dalam satu proyek.
  3. **Pemisahan Geometri vs Netlist:** Entitas dokumen (`Project`, `CircuitDefinition`, `ComponentInstance`, `WireSegment`, `Junction`, `Tunnel`, `PortDefinition`) hanya menyimpan representasi pengeditan visual. Konektivitas listrik (*net*) tidak disimpan di dokumen JSON, melainkan diturunkan (*derived*) saat fase kompilasi frontend (M2).
  4. **Endpoint Kawat Bersifat Eksplisit:** Setiap `WireSegment` memiliki `start` dan `end` bertipe `Endpoint` (`Pin`, `Junction`, `Port`, atau `Free`), dilengkapi titik bantu lintasan (`waypoints`) untuk rute ortogonal.
  5. **Fixture Baseline:** Dibuat fixture awal `tests/fixtures/simple_circuit.json` yang memodelkan *Half Adder* (XOR + AND + Junctions + Ports).
- **Konsekuensi & Dampak:**
  - Menjamin kompatibilitas serialisasi *round-trip* deterministik (KF-01).
  - Skema dokumen siap dipakai oleh modul Rust `model` (`Project`, `CircuitDefinition`, dll.) pada task M1.02–M1.08.

### [SPIKE-003] Arsitektur Resolusi Netlist Kanonis (DSU), Validasi Multi-Driver, dan Analisis Ketergantungan (Tarjan SCC & Levelization)
- **Tanggal:** 2026-11-22
- **Milestone:** M2 (Connectivity & Semantic Frontend)
- **Konteks:** Menentukan strategi perancangan frontend kompilasi untuk mentransformasikan dokumen pengeditan grafis menjadi netlist elektris yang deterministik, mendeteksi pelanggaran elektris, serta membentuk graf dependensi dan urutan eksekusi kombinasional (*levelization*) sesuai Bab 3.4.1, 3.4.2, dan 3.10.11 proposal.
- **Opsi yang Dieksplorasi:**
  1. *Adjacency List BFS/DFS Traversal langsung pada Canvas*: Bergantung pada urutan array kawat di memori; urutan traversal mudah berubah jika kawat dihapus/ditambah ulang, menghasilkan Net ID yang tidak deterministik.
  2. *Disjoint Set Union (DSU) dengan Root Minimum ID Kanonis*: Mengelompokkan seluruh endpoint terhubung ke dalam himpunan saling lepas (*equivalence classes*). Root kanonis selalu memilih ID objek terkecil di dalam himpunan. Kompleksitas amortisasi $O(\alpha(N))$ mendekati linier, deterministik 100%.
  3. *Deteksi Siklus via DFS Sederhana vs Tarjan SCC*: DFS biasa hanya mendeteksi "ada siklus atau tidak", tetapi tidak dapat mengisolasi komponen terhubung kuat (*Strongly Connected Components*) untuk dievaluasi secara iteratif atau dikondensasi menjadi DAG. Algoritma Tarjan SCC menemukan seluruh komponen siklus tertutup dalam satu lintasan linier $O(V + E)$.
- **Keputusan Terpilih:**
  1. **Identitas Net Kanonis (`NetId`):** `NetId(pub u64)` berbasis newtype, bernilai ID minimum dari anggota node di dalam himpunan partisi DSU (misal ID Pin terkecil atau Port terkecil). Menjamin dokumen identik selalu menghasilkan `NetId` yang persis sama (KNF-01).
  2. **Model Node Konektivitas (`ConnectionNode`):** Abstraksi terpadu yang memetakan terminal `Pin { instance_id, pin_id }`, `Port(PinId)`, `Junction(JunctionId)`, dan `Tunnel(TunnelId)`.
  3. **DSU Deterministik:** Struktur data `DisjointSet` mengimplementasikan *path compression* dan *canonical tracking*: operasi `union` selalu menetapkan elemen bernilai ID terkecil sebagai representatif kanonis himpunan.
  4. **Resolusi Tunnel:** Seluruh node tunnel dalam sirkuit yang sama dengan nama label yang identik disatukan ke dalam partisi net yang sama. Tunnel tunggal tanpa pasangan dalam cakupan sirkuit dilaporkan dengan `CONN-TUNNEL-UNPAIRED`.
  5. **Netlist & Validasi Elektris:**
     - Struktur `Net` memisahkan secara tegas antara `drivers` (pin/port keluaran) dan `loads` (pin/port masukan) serta mencatat `width: SignalWidth`.
     - Validasi Multi-Driver: Jika sebuah net memiliki `drivers.len() > 1` (dua atau lebih output berkonflik tanpa tri-state/resolusi), validator menerbitkan error `NET-MULTI-DRIVER`.
     - Validasi Undriven: Jika sebuah net memiliki `drivers.is_empty()` namun memiliki `loads.len() > 0`, validator menerbitkan error `NET-UNDRIVEN`.
  6. **Graf Ketergantungan Kombinasional (`DependencyGraph`):**
     - Simpul berupa komponen sirkuit (`InstanceId`).
     - Sisi terarah mengalir dari komponen driver net ke komponen load net.
     - Komponen sekuensial (DFF): pin input `d` adalah load pada siklus $t$, sedangkan pin output `q` adalah driver pada siklus $t+1$ (level 0), sehingga memutus siklus kombinasional antar-siklus.
  7. **Tarjan SCC & Levelization:**
     - Menggunakan algoritma Tarjan untuk mendeteksi siklus ketergantungan.
     - Setiap SCC dengan jumlah simpul > 1 (atau self-loop) ditandai sebagai wilayah umpan balik (*feedback loop*). Rangkaian tanpa umpan balik kombinasional diverifikasi memiliki seluruh SCC berukuran 1 (DAG murni).
     - Untuk rangkaian asiklik (DAG), dihitung kedalaman level topological: $\text{level}(u) = 1 + \max_{(v \to u)} \text{level}(v)$. Komponen diurutkan per level, dan di dalam level yang sama diurutkan secara deterministik berdasarkan `InstanceId`.
- **Konsekuensi & Dampak:**
  - Netlist kanonis dan urutan levelization siap dikonsumsi langsung oleh Milestone M3 (*First Compiled Simulation*) untuk IR Lowering dan alokasi layout memori simulasi.
  - Memenuhi jaminan kebenaran determinisme (KNF-01) dan keterlacakan diagnostik visual (KNF-03).
