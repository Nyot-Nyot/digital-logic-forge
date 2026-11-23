# Digital Logic Forge (DLF) — Progress Dashboard

> **Status Saat Ini:** Milestone M2 (Connectivity & Semantic Frontend) — **ACCEPTED ✅**  
> **Terakhir Diperbarui:** 23 November 2026  
> **Single Source of Truth (SSoT):** [Proposal TA (Markdown)](../compiled%20circuit%20simulator.md), [Sprint Progress](PROGRESS.md), & Dokumen ini.

---

## 1. Ringkasan Status Milestone

| ID | Milestone | Prioritas | Status | Target Deliverable Utama |
| :---: | :--- | :---: | :---: | :--- |
| **M0** | **Implementation Baseline** | P0 | **ACCEPTED** ✅ | Kerangka workspace Cargo, binding GDExtension, smoke test, baseline prototype canvas |
| **M1** | **Headless Circuit Model** | P0 | **ACCEPTED** ✅ | Dokumen JSON, Stable ID 64-bit, structural validator |
| **M2** | **Connectivity & Semantic Frontend** | P0 | **ACCEPTED** ✅ | Resolusi net & junction, validasi elektris, SCC Tarjan, levelization, hierarki |
| **M3** | **First Compiled Simulation** | P0 | *Ready to Start* 🚀 | Snapshot -> Compile -> IR ringkas -> Runtime B1 vs B0 oracle |
| **M4** | **Stateful Simulation** | P0 | *Not Started* | Clock domain, register simultaneous commit, RAM/ROM |
| **M5** | **Editor Vertical Slice** | P0 | *Not Started* | Canvas editor Godot, placement, wiring, inspector UI |
| **M6** | **Interactive Runtime** | P0 | *Not Started* | Worker thread, non-blocking UI, command queue, probe viewer |
| **M7** | **Benchmark Baseline** | P0 | *Not Started* | Benchmark suite W01-W09 di Lenovo Y520, manifest & raw results |
| **M8** | **CPU Demonstration** | P1 | *Not Started* | CPU buatan pengguna, eksekusi program deterministik, FPS display |
| **M9** | **Stabilization & Delivery** | P0 | *Not Started* | Regression suite, installer Windows (.exe), final thesis report |

---

## 2. Checklist Detail Milestone yang Baru Selesai: M2 (Connectivity & Semantic Frontend)

- [x] SPIKE riset DSU deterministik kanonis, validasi elektris, dan pemutusan siklus kombinasional register (`[SPIKE-003]`)
- [x] Implementasi `DisjointSet<T>` dengan path compression, union-by-rank, dan `canonical_min` tracking
- [x] Ekstraksi simpul konektivitas `ConnectionNode` (Pin, Port, Junction, Tunnel) dan parsing kawat
- [x] Resolusi percabangan multi-kawat pada Junction dan segmen merging via DSU
- [x] Resolusi pasangan Tunnel nirkabel (Local & Global) dan diagnostik `CONN-TUNNEL-UNPAIRED`
- [x] Konstruksi Netlist Kanonis (`NetId`, `NetTerminal`, `Net`, `Netlist`) dengan pemetaan cepat $O(\log N)$
- [x] Validasi aturan elektris Multi-Driver check (`NET-MULTI-DRIVER`)
- [x] Validasi aturan elektris Undriven Net check (`NET-UNDRIVEN`)
- [x] Konstruksi Graf Ketergantungan Kombinasional (`DependencyGraph`) dengan pemisahan simpul DFF `SequentialSource` (level 0) vs `SequentialSink`
- [x] Algoritma Tarjan SCC untuk isolasi komponen umpan balik dan pelaporan `DEP-SCC-OSCILLATION`
- [x] Levelization kombinasional statis (`EvaluationSchedule`) dengan pengurutan linier deterministik
- [x] Elaborasi hierarki sub-sirkuit kustom dasar (`CircuitInstanceNode`), port-to-pin binding, dan deteksi rekursi tak berhingga `HIER-RECURSION`
- [x] Rangkaian integrasi end-to-end `crates/dlf-core/tests/connectivity_test.rs` (6 skenario lengkap)
- [x] Seluruh 65 automated test dan skrip PowerShell (`check.ps1`, `test.ps1`) lulus 100% tanpa warning

---

## 3. Matriks Keputusan & Penutupan TBD

| ID | Topik TBD | Batas Penutupan | Status | Catatan / Keputusan Terpilih |
| :--- | :--- | :---: | :---: | :--- |
| **TBD-1** | Tabel kebenaran primitif & MUX (X/Z) | M3 | Open | Usulan: y = X jika select bernilai X/Z kecuali semua data sama |
| **TBD-2** | Pengodean 4-nilai (Mask vs 2-bit pack) | M3 / M4 | Open | Eksperimen benchmarking pada M3 |
| **TBD-3** | Algoritma konvergensi SCC & batas iterasi | M3 | Open | Trigger: ketersediaan workload W07 |
| **TBD-4** | Kebijakan state awal sekuensial | M4 | Open | Mode ketat bernilai X, mode P0 bernilai 0 |
| **TBD-5** | Prioritas reset vs enable register | M4 | Open | Ditetapkan per kontrak komponen sekuensial |
| **TBD-6** | Mode baca bawaan RAM (read-first dll.) | M4 | Open | Usulan: read-first untuk synchronous single-port |
| **TBD-7** | Primitif pull-up dan pull-down | Ditunda | Open | Hanya jika ada workload tri-state terselesaikan |
| **TBD-8** | Kebijakan multi-clock domain | Ditunda (P2) | Open | Out of scope inti P0 |

---

## 4. Log Perubahan Terakhir (Session Log)
- **23 Nov 2026**: Milestone M2 (Connectivity & Semantic Frontend) diselesaikan 100% (13/13 tasks). DSU deterministik kanonis, netlist kanonis, validasi elektris multi-driver & undriven, dependency graph dengan pemutusan loop DFF sekuensial, algoritma Tarjan SCC (`DEP-SCC-OSCILLATION`), levelization topologis kombinasional (`EvaluationSchedule`), serta elaborasi hierarki dan deteksi rekursi (`HIER-RECURSION`) berhasil diimplementasikan dan diverifikasi dengan 65 automated tests.
- **19 Nov 2026**: Milestone M1 (Headless Circuit Model) diselesaikan 100% (14/14 tasks). Format dokumen JSON v1, Stable ID 64-bit deterministik, kontrak pin & bus width 1..64 bit, registry komponen L0/L1, geometri wiring, round-trip fixture test suite, serta validator struktural (`CONN-PIN-UNCONNECTED`, `CONN-WIRE-DANGLING`, `CONN-WIDTH-MISMATCH`) berhasil diimplementasikan dan diverifikasi dengan 31 automated tests.
- **18 Nov 2026**: Inisialisasi tata kelola proyek, `AGENTS.md`, master workflow framework, dan `DASHBOARD.md`. Penyelesaian Milestone M0.
