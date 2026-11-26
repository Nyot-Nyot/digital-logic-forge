# Digital Logic Forge (DLF) — Sprint Backlog & Progress Tracker

> **Milestone Aktif:** **Milestone M3 (First Compiled Simulation)**  
> **Status Sprint:** **PRE-SPRINT COMPLETED → READY FOR DAILY LOOP**  
> **Terakhir Diperbarui:** 23 November 2026

---

## 1. Task yang Sedang Berjalan (IN PROGRESS)

- **M3.05 [DELIVERY]**: Pipeline Snapshot & Kompilasi Sirkuit (`CircuitSnapshot` & `CircuitCompiler`)

---

## 2. Backlog Task Milestone M3 (First Compiled Simulation)

| ID | Tipe | Deskripsi Task & Definition of Done (DoD) | Dependensi | Status |
| :---: | :---: | :--- | :---: | :---: |
| **M3.01** | **SPIKE** | **Riset & Desain IR, Memory Layout, Semantik 4-Nilai, & Arsitektur Reference Evaluator (B0)**<br/>• *DoD:* Dokumen keputusan teknis di `docs/decisions/DECISIONS.md` (`[SPIKE-004]`) yang menetapkan arsitektur opcode IR ringkas (Tabel 3.25), layout array kontigu (Tabel 3.26), resolusi TBD-1 (tabel kebenaran MUX & konservatisme IEEE), analisis TBD-2 (pengodean 4-nilai mask vs 2-bit packed), serta strategi differential testing B0 vs B1. | *None* | **DONE** ✅ |
| **M3.02** | **DELIVERY** | **Definisi Model IR & Instruksi Kompilasi (`Opcode`, `CompiledOp`, `CompiledArtifact`)**<br/>• *DoD:* Modul `crates/dlf-core/src/ir/` dengan enum `Opcode` (BUF, NOT, AND, OR, XOR, NAND, NOR, XNOR, CONST, MUX, SLICE, CONCAT), struct `CompiledOp` (`opcode, dst, src_a, src_b, src_c, width, flags`), dan container `CompiledArtifact`. Unit test validasi invariant struct IR lulus. | M3.01 | **DONE** ✅ |
| **M3.03** | **DELIVERY** | **Alokasi Slot Memori Kontigu & Pemetaan Simbolik (`SignalSlotAllocator`, `SourceMap`, `Bindings`)**<br/>• *DoD:* Struct `SignalSlotAllocator` untuk memetakan `NetId` dan konstanta ke indeks slot memori (`usize`), serta pembuatan `InputBinding`, `OutputBinding`, dan `SourceMap` (Tabel 3.26) yang memetakan slot runtime kembali ke ID komponen/pin editor untuk diagnostik. Unit test alokasi lulus. | M3.02 | **DONE** ✅ |
| **M3.04** | **DELIVERY** | **IR Lowering Engine (Translasi Netlist & EvaluationSchedule ke IR Terkompilasi)**<br/>• *DoD:* Modul `crates/dlf-core/src/compiler/lowering.rs` yang mengonversi `Netlist`, komponen primitif kombinasional, dan `EvaluationSchedule` hasil levelisasi M2 menjadi daftar `CompiledOp` berurutan per level (`LevelRange`). Unit test lowering sirkuit dasar lulus. | M3.03 | **DONE** ✅ |
| **M3.05** | **DELIVERY** | **Pipeline Snapshot & Kompilasi Sirkuit (`CircuitSnapshot` & `CircuitCompiler`)**<br/>• *DoD:* Modul `crates/dlf-core/src/compiler/pipeline.rs` yang membekukan `CircuitDocument` menjadi snapshot immutable, menjalankan validasi frontend M2 (netlist, SCC, electrical rules), dan memanggil lowering engine untuk menghasilkan `CompiledArtifact`. Error diagnostik lengkap jika kompilasi gagal. Unit test lulus. | M3.04 | **IN PROGRESS** 🔨 |
| **M3.06** | **DELIVERY** | **Reference Evaluator Kombinasional (B0 Oracle Interpreter)**<br/>• *DoD:* Modul `crates/dlf-core/src/runtime/reference.rs` sebagai evaluator semantik independen (B0) yang mengevaluasi graf rangkaian secara langsung (topological graph walk tanpa IR flat) untuk menghasilkan golden reference output. Unit test evaluator B0 lulus. | M3.01, M3.04 | **BLOCKED** |
| **M3.07** | **DELIVERY** | **Compiled Simulation Runtime Engine (B1 Evaluator: Storage & Cycle Execution)**<br/>• *DoD:* Modul `crates/dlf-core/src/runtime/compiled.rs` dengan `CompiledRuntime` yang mengalokasikan flat array `SignalValue: Vec<u64>` (dan mask bila 4-nilai), eksekusi per-level dari `CompiledOp`, serta metode `step_cycle()` kombinasional. Unit test alokasi runtime lulus. | M3.02, M3.05 | **BLOCKED** |
| **M3.08** | **DELIVERY** | **Eksekusi Primitif Logika & Bus Masking pada Engine Terkompilasi**<br/>• *DoD:* Implementasi eksekusi bitwise untuk seluruh opcode logika (BUF, NOT, AND, OR, XOR, NAND, NOR, XNOR, CONST) pada 64-bit word dengan penerapan width mask eksplisit (`(1 << width) - 1`) agar bit di luar lebar bus tidak bocor. Unit test multi-bit logic masking lulus. | M3.07 | **BLOCKED** |
| **M3.09** | **DELIVERY** | **Eksekusi Operasi Multiplexer (MUX) & Manipulasi Bus (SLICE, CONCAT)**<br/>• *DoD:* Implementasi eksekusi MUX multi-bit (dengan semantik penanganan konservatif sesuai TBD-1), operasi SLICE (ekstraksi bitfield bus), dan CONCAT (penggabungan beberapa bus menjadi satu). Unit test operasi manipulasi data lulus. | M3.08 | **BLOCKED** |
| **M3.10** | **DELIVERY** | **Antarmuka Interaksi Runtime (`SetInput`, `GetOutput`, & `Probe` State Inspection)**<br/>• *DoD:* API publik pada `CompiledRuntime` untuk `set_input(name/id, value)`, `get_output(name/id)`, serta pembacaan nilai slot via `SourceMap` untuk probe diagnostik. Unit test I/O interaksi lulus. | M3.09 | **BLOCKED** |
| **M3.11** | **DELIVERY** | **Differential Testing & Fuzzing Harness (B1 vs B0 Bit-Exact Verification)**<br/>• *DoD:* Test harness `crates/dlf-core/src/runtime/differential.rs` yang mengeksekusi input acak (fuzzing) dan tabel kebenaran lengkap pada B1 dan B0, memverifikasi kesetaraan keluaran bit-exact 100%. Lulus pada berbagai variasi rangkaian kombinasional. | M3.06, M3.10 | **BLOCKED** |
| **M3.12** | **DELIVERY** | **End-to-End M3 Compiled Simulation Integration Suite (`workload W01-W04`)**<br/>• *DoD:* Test integration `crates/dlf-core/tests/compiled_simulation_test.rs` menguji skenario kombinasional kompleks non-trivial (4-bit ripple-carry adder, 8-bit ALU dengan opcode select, 8-bit barrel shifter, priority encoder), validasi passing `scripts/check.ps1` dan `scripts/test.ps1`. | M3.11 | **BLOCKED** |

---

## 3. Ringkasan Status Task M3
- **Total Task:** 12 Task (1 SPIKE, 11 DELIVERY)
- **Status:** 1 READY 🚀, 11 BLOCKED ⏳, 0 IN PROGRESS, 0 DONE
- **Progress:** `0 / 12` (0.0%)
