# Digital Logic Forge (DLF) — Sprint Backlog & Progress Tracker

> **Milestone Aktif:** **Milestone M2 (Connectivity & Semantic Frontend)**  
> **Status Sprint:** **PRE-SPRINT COMPLETED → READY FOR DAILY LOOP**  
> **Terakhir Diperbarui:** 19 November 2026

---

## 1. Task yang Sedang Berjalan (IN PROGRESS)

- [ ] **M2.06 [DELIVERY]** — **Konstruksi Netlist Kanonis & Model Net**  
  *Deskripsi:* Buat modul `crates/dlf-core/src/connectivity/netlist.rs` yang mendefinisikan tipe `NetId`, struct `Net` (id kanonis, lebar bit, daftar pin driver, daftar pin load, dan referensi komponen), serta konstruktor `Netlist` yang mengubah hasil partisi DSU dan sirkuit menjadi netlist terstruktur kanonis.  
  *DoD:* Struct `Net` dan `Netlist`, builder dari partitioned DSU sets + `CircuitDefinition`, penentuan canonical root id, pengelompokan driver vs load pins, unit test konstruksi netlist lulus.

---

## 2. Backlog Task Milestone M2 (Connectivity & Semantic Frontend)

| ID | Tipe | Deskripsi Task & Definition of Done (DoD) | Dependensi | Status |
| :---: | :---: | :--- | :---: | :---: |
| **M2.01** | **SPIKE** | **Riset & Desain Topologi Net, DSU, dan Ketergantungan Graf**<br/>• *DoD:* Spesifikasi struktur `NetId`, data graph topologi endpoint, strategi DSU deterministik kanonis, penanganan SCC Tarjan, dan keputusan dicatat di `docs/decisions/DECISIONS.md`. | *None* | **DONE** ✅ |
| **M2.02** | **DELIVERY** | **Implementasi Disjoint Set Union (DSU) Deterministik**<br/>• *DoD:* Modul `crates/dlf-core/src/connectivity/dsu.rs` dengan struct `DisjointSet`, path compression & union-by-rank, penentuan root kanonis berbasis ID minimum untuk determinisme, unit test lulus. | M2.01 | **DONE** ✅ |
| **M2.03** | **DELIVERY** | **Ekstraksi Node Konektivitas & Normalisasi Endpoint**<br/>• *DoD:* Modul `crates/dlf-core/src/connectivity/node.rs` dengan `ConnectionNode` (Pin, Port, Junction, Tunnel), parser yang memetakan seluruh kawat (`WireSegment`) ke node-node konektivitas, unit test ekstraksi lulus. | M2.02 | **DONE** ✅ |
| **M2.04** | **DELIVERY** | **Resolusi Percabangan Junction & Segment Merging**<br/>• *DoD:* Implementasi aturan konektivitas visual Bab 3.4.2 (kawat menempel ke pin, port, junction), penggabungan segmen kawat yang bertemu pada junction via DSU, unit test topologi branching lulus. | M2.03 | **DONE** ✅ |
| **M2.05** | **DELIVERY** | **Resolusi Tunnel Nirkabel (Local & Global Scopes)**<br/>• *DoD:* Penyatuan node tunnel yang memiliki nama label sama dalam satu sirkuit/scope ke net yang sama via DSU, deteksi dan laporan `CONN-TUNNEL-UNPAIRED` bila ada tunnel tanpa pasangan, unit test lulus. | M2.03 | **DONE** ✅ |
| **M2.06** | **DELIVERY** | **Konstruksi Netlist Kanonis & Model Net**<br/>• *DoD:* Struct `Net` (NetId kanonis, daftar pin driver, daftar pin load, lebar bit) dan `Netlist`, transformasi hasil partisi DSU menjadi netlist terstruktur, unit test konstruksi netlist lulus. | M2.04, M2.05 | **IN PROGRESS** ⏳ |
| **M2.07** | **DELIVERY** | **Validasi Elektris Net: Multi-Driver Check (`NET-MULTI-DRIVER`)**<br/>• *DoD:* Aturan validasi net untuk mendeteksi lebih dari satu driver aktif pada net yang tidak diresolusi (dua output pin berkonflik), menghasilkan error `NET-MULTI-DRIVER` dengan daftar driver terkait, unit test lulus. | M2.06 | *BLOCKED* |
| **M2.08** | **DELIVERY** | **Validasi Elektris Net: Undriven Net Check (`NET-UNDRIVEN`)**<br/>• *DoD:* Aturan validasi net mendeteksi net yang memiliki beban (input pin) tetapi tidak memiliki penggerak (output pin/port), menghasilkan warning/error `NET-UNDRIVEN`, unit test lulus. | M2.06 | *BLOCKED* |
| **M2.09** | **DELIVERY** | **Konstruksi Graf Ketergantungan Kombinasional**<br/>• *DoD:* Modul `crates/dlf-core/src/analysis/graph.rs` dengan struct `DependencyGraph`, simpul operasi dan sisi aliran sinyal driver-ke-load, pemutusan siklus kombinasional pada register (DFF), unit test graf lulus. | M2.06 | *BLOCKED* |
| **M2.10** | **DELIVERY** | **Deteksi Siklus & Analisis SCC (Tarjan Algorithm)**<br/>• *DoD:* Modul `crates/dlf-core/src/analysis/scc.rs` dengan algoritma Tarjan untuk mendeteksi Strongly Connected Components, pemisahan wilayah asiklik (DAG) dari siklik (feedback), pelaporan diagnostik `DEP-SCC-OSCILLATION`, unit test lulus. | M2.09 | *BLOCKED* |
| **M2.11** | **DELIVERY** | **Levelization & Topological Sorting Kombinasional**<br/>• *DoD:* Modul `crates/dlf-core/src/analysis/levelize.rs` untuk menghitung kedalaman level ketergantungan (level = 1 + max(level sumber)), pengurutan deterministik berurutan level dan ID stabil, unit test levelization lulus. | M2.10 | *BLOCKED* |
| **M2.12** | **DELIVERY** | **Elaborasi Hierarki Sub-Sirkuit Dasar (Target P1)**<br/>• *DoD:* Modul `crates/dlf-core/src/analysis/hierarchy.rs` untuk ekspansi sub-sirkuit chip kustom, port-to-pin binding, deteksi siklus instansiasi rekursif `HIER-RECURSION`, unit test elaborasi lulus. | M2.06 | *BLOCKED* |
| **M2.13** | **DELIVERY** | **End-to-End M2 Connectivity & Semantic Test Suite**<br/>• *DoD:* Rangkaian integration test komprehensif pada `crates/dlf-core/tests/connectivity_test.rs` dengan aneka variasi sirkuit, seluruh automated tests (`scripts/check.ps1` dan `scripts/test.ps1`) lulus 100%. | M2.07, M2.08, M2.11, M2.12 | *BLOCKED* |

---

## 3. Ringkasan Status Task M2
- **Total Task:** 13 Task (1 SPIKE, 12 DELIVERY)
- **Status:** 0 READY, 7 BLOCKED, 1 IN PROGRESS, 5 DONE
- **Progress:** `5 / 13` (38.5%)
