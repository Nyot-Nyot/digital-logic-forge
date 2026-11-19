# Digital Logic Forge (DLF) — Progress Dashboard

> **Status Saat Ini:** Milestone M1 (Headless Circuit Model) — **ACCEPTED** ✅ → Siap Masuk Pre-Sprint M2  
> **Terakhir Diperbarui:** 19 November 2026  
> **Single Source of Truth (SSoT):** [Proposal TA (Markdown)](../compiled%20circuit%20simulator.md), [Sprint Progress](PROGRESS.md), & Dokumen ini.

---

## 1. Ringkasan Status Milestone

| ID | Milestone | Prioritas | Status | Target Deliverable Utama |
| :---: | :--- | :---: | :---: | :--- |
| **M0** | **Implementation Baseline** | P0 | **ACCEPTED** ✅ | Kerangka workspace Cargo, binding GDExtension, smoke test, baseline prototype canvas |
| **M1** | **Headless Circuit Model** | P0 | **ACCEPTED** ✅ | Dokumen JSON, Stable ID 64-bit, structural validator |
| **M2** | **Connectivity & Semantic Frontend** | P0 | *Not Started* | Resolusi net & junction, validasi lebar bus, SCC detection |
| **M3** | **First Compiled Simulation** | P0 | *Not Started* | Snapshot -> Compile -> IR ringkas -> Runtime B1 vs B0 oracle |
| **M4** | **Stateful Simulation** | P0 | *Not Started* | Clock domain, register simultaneous commit, RAM/ROM |
| **M5** | **Editor Vertical Slice** | P0 | *Not Started* | Canvas editor Godot, placement, wiring, inspector UI |
| **M6** | **Interactive Runtime** | P0 | *Not Started* | Worker thread, non-blocking UI, command queue, probe viewer |
| **M7** | **Benchmark Baseline** | P0 | *Not Started* | Benchmark suite W01-W09 di Lenovo Y520, manifest & raw results |
| **M8** | **CPU Demonstration** | P1 | *Not Started* | CPU buatan pengguna, eksekusi program deterministik, FPS display |
| **M9** | **Stabilization & Delivery** | P0 | *Not Started* | Regression suite, installer Windows (.exe), final thesis report |

---

## 2. Checklist Detail Milestone Aktif: M0 (Implementation Baseline)

- [x] Konfirmasi pedoman kode dan working contract di `AGENTS.md`
- [x] Inisialisasi Git repository & remote GitHub (`Nyot-Nyot/digital-logic-forge`)
- [x] Setup root `README.md` dengan visualisasi workflow dan status aktif
- [x] Inisialisasi Cargo Workspace (`dlf-core`, `dlf-godot`) dengan `rust-toolchain.toml` edisi 2024
- [x] Setup proyek dasar Godot 4.7.2 di `app/godot/` & binding `dlf.gdextension`
- [x] Implementasi GDExtension binding tipis & smoke test pemanggilan dari GDScript
- [x] Verifikasi worker Rust thread independen & snapshot non-blocking polling
- [x] Prototype canvas terpusat (pan/zoom, selection, hit-testing O(1) 2.2 µs pada 2500 elemen)
- [x] Verifikasi profil release `dlf_godot.dll` dengan LTO
- [x] Skrip otomatisasi PowerShell di `scripts/` (`check.ps1`, `test.ps1`, `build_extension.ps1`)
- [x] Audit 8 kriteria keluar M0 & penutupan checklist M0 (Semua 100% Lulus)

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
- **19 Nov 2026**: Milestone M1 (Headless Circuit Model) diselesaikan 100% (14/14 tasks). Format dokumen JSON v1, Stable ID 64-bit deterministik, kontrak pin & bus width 1..64 bit, registry komponen L0/L1, geometri wiring, round-trip fixture test suite, serta validator struktural (`CONN-PIN-UNCONNECTED`, `CONN-WIRE-DANGLING`, `CONN-WIDTH-MISMATCH`) berhasil diimplementasikan dan diverifikasi dengan 31 automated tests.
- **18 Nov 2026**: Inisialisasi tata kelola proyek, `AGENTS.md`, master workflow framework, dan `DASHBOARD.md`. Penyelesaian Milestone M0.
