# Digital Logic Forge (DLF) — Sprint Backlog & Progress Tracker

> **Milestone Aktif:** **Milestone M1 (Headless Circuit Model)**  
> **Status Sprint:** **PRE-SPRINT COMPLETED → READY FOR DAILY LOOP**  
> **Terakhir Diperbarui:** 19 November 2026

---

## 1. Task yang Sedang Berjalan (IN PROGRESS)

> *Task M1.05 telah selesai. Siap melanjutkan ke Task M1.06 (CircuitDefinition & Project Hierarchy Container).*

---

## 2. Backlog Task Milestone M1 (Headless Circuit Model)

| ID | Tipe | Deskripsi Task & Definition of Done (DoD) | Dependensi | Status |
| :---: | :---: | :--- | :---: | :---: |
| **M1.01** | **SPIKE** | **Spesifikasi Skema JSON Dokumen & Versi Format**<br/>• *DoD:* Format JSON untuk entitas dokumen didefinisikan secara eksplisit, contoh fixture JSON valid tersedia, dan keputusan dicatat di `docs/decisions/DECISIONS.md`. | *None* | **DONE** ✅ |
| **M1.02** | **DELIVERY** | **Stable ID 64-bit Generator & Identifier Types**<br/>• *DoD:* Generator ID terpusat deterministik per Project, tipe kuat (`InstanceId`, `PinId`, `WireId`, `CircuitId`), unit test keunikan ID lulus. | M1.01 | **DONE** ✅ |
| **M1.03** | **DELIVERY** | **Pin Contract, Direction, & Bus Width Types**<br/>• *DoD:* Struct `PinDeclaration`, `PinInstance`, enum `PinDirection` (Input, Output, Inout), enum `PinRole`, tipe `SignalWidth` (1..64), unit test validasi lebar pin lulus. | M1.02 | **DONE** ✅ |
| **M1.04** | **DELIVERY** | **Component Definition & Instance Model**<br/>• *DoD:* Struct `ComponentDefinition`, `ComponentInstance`, registry pustaka primitif L0/L1 bawaan (AND, OR, NOT, XOR, DFF, RAM, dll.), unit test registry lookup lulus. | M1.03 | **DONE** ✅ |
| **M1.05** | **DELIVERY** | **Geometri WireSegment, Junction, & Tunnel Entities**<br/>• *DoD:* Struct `WireSegment` (geometri rute & dua endpoint), `Junction` (titik cabang topologis), `Tunnel` (label nama & scope), unit test pembentukan entitas lulus. | M1.02 | **DONE** ✅ |
| **M1.06** | **DELIVERY** | **CircuitDefinition & Project Hierarchy Container**<br/>• *DoD:* Struct `CircuitDefinition` (daftar instance, wire, junction, port) dan `Project` (akar multi-circuit), unit test pembuatan sirkuit bertingkat lulus. | M1.04, M1.05 | **READY** 🚀 |
| **M1.07** | **DELIVERY** | **Serde JSON Serialization Engine**<br/>• *DoD:* Implementasi `Serialize` untuk semua entitas dokumen, output JSON rapi (*pretty printed*), unit test serialisasi menghasilkan string JSON valid. | M1.06 | *BLOCKED* |
| **M1.08** | **DELIVERY** | **Serde JSON Deserialization & Schema Validation**<br/>• *DoD:* Implementasi `Deserialize` dengan pemeriksaan `schema_version`, penanganan skema lama/tidak cocok dengan pesan error deskriptif, unit test parsing lulus. | M1.07 | *BLOCKED* |
| **M1.09** | **DELIVERY** | **Round-Trip Fixture Test Suite**<br/>• *DoD:* Test suite otomatis yang memuat berkas JSON fixture, mem-parse ke memory, me-reserialize, dan memvalidasi kesamaan semantik secara bit-exact (*round-trip*). | M1.08 | *BLOCKED* |
| **M1.10** | **DELIVERY** | **Diagnostic Framework Model**<br/>• *DoD:* Struct `Diagnostic`, enum `Severity` (Error, Warning, Info), enum `DiagnosticCode` stabil (`CONN-*`, `HIER-*`), pemetaan referensi objek visual (`primary`, `related`). | M1.01 | **READY** 🚀 |
| **M1.11** | **DELIVERY** | **Structural Validator: Unconnected Mandatory Pins**<br/>• *DoD:* Rule validator untuk mendeteksi pin masukan wajib yang mengambang/tidak terhubung, menghasilkan error `CONN-PIN-UNCONNECTED` dengan ID target yang akurat. | M1.06, M1.10 | *BLOCKED* |
| **M1.12** | **DELIVERY** | **Structural Validator: Dangling Wires & Endpoints**<br/>• *DoD:* Rule validator mendeteksi segmen kawat yang menggantung bebas tanpa terhubung ke pin/junction lain, menghasilkan warning `CONN-WIRE-DANGLING`. | M1.06, M1.10 | *BLOCKED* |
| **M1.13** | **DELIVERY** | **Structural Validator: Bus Width Mismatch Check**<br/>• *DoD:* Rule validator memeriksa ketidakcocokan lebar bit antara pin yang tersambung, menghasilkan error `CONN-WIDTH-MISMATCH`, serta memverifikasi integritas referensi ID. | M1.06, M1.10 | *BLOCKED* |
| **M1.14** | **DELIVERY** | **End-to-End M1 Headless Validation Suite & Scripts**<br/>• *DoD:* Seluruh unit test dan integration test M1 lulus 100%, `cargo fmt --check` dan `cargo clippy -- -D warnings` bersih, skrip `scripts/test.ps1` lulus. | M1.09, M1.11, M1.12, M1.13 | *BLOCKED* |

---

## 3. Ringkasan Status Task M1
- **Total Task:** 14 Task (1 SPIKE, 13 DELIVERY)
- **Status:** 2 READY, 7 BLOCKED, 0 IN PROGRESS, 5 DONE
- **Progress:** `5 / 14` (35.7%)
| **M1.07** | **DELIVERY** | **Serde JSON Serialization Engine**<br/>• *DoD:* Implementasi `Serialize` untuk semua entitas dokumen, output JSON rapi (*pretty printed*), unit test serialisasi menghasilkan string JSON valid. | M1.06 | *BLOCKED* |
| **M1.08** | **DELIVERY** | **Serde JSON Deserialization & Schema Validation**<br/>• *DoD:* Implementasi `Deserialize` dengan pemeriksaan `schema_version`, penanganan skema lama/tidak cocok dengan pesan error deskriptif, unit test parsing lulus. | M1.07 | *BLOCKED* |
| **M1.09** | **DELIVERY** | **Round-Trip Fixture Test Suite**<br/>• *DoD:* Test suite otomatis yang memuat berkas JSON fixture, mem-parse ke memory, me-reserialize, dan memvalidasi kesamaan semantik secara bit-exact (*round-trip*). | M1.08 | *BLOCKED* |
| **M1.10** | **DELIVERY** | **Diagnostic Framework Model**<br/>• *DoD:* Struct `Diagnostic`, enum `Severity` (Error, Warning, Info), enum `DiagnosticCode` stabil (`CONN-*`, `HIER-*`), pemetaan referensi objek visual (`primary`, `related`). | M1.01 | **READY** 🚀 |
| **M1.11** | **DELIVERY** | **Structural Validator: Unconnected Mandatory Pins**<br/>• *DoD:* Rule validator untuk mendeteksi pin masukan wajib yang mengambang/tidak terhubung, menghasilkan error `CONN-PIN-UNCONNECTED` dengan ID target yang akurat. | M1.06, M1.10 | *BLOCKED* |
| **M1.12** | **DELIVERY** | **Structural Validator: Dangling Wires & Endpoints**<br/>• *DoD:* Rule validator mendeteksi segmen kawat yang menggantung bebas tanpa terhubung ke pin/junction lain, menghasilkan warning `CONN-WIRE-DANGLING`. | M1.06, M1.10 | *BLOCKED* |
| **M1.13** | **DELIVERY** | **Structural Validator: Bus Width Mismatch Check**<br/>• *DoD:* Rule validator memeriksa ketidakcocokan lebar bit antara pin yang tersambung, menghasilkan error `CONN-WIDTH-MISMATCH`, serta memverifikasi integritas referensi ID. | M1.06, M1.10 | *BLOCKED* |
| **M1.14** | **DELIVERY** | **End-to-End M1 Headless Validation Suite & Scripts**<br/>• *DoD:* Seluruh unit test dan integration test M1 lulus 100%, `cargo fmt --check` dan `cargo clippy -- -D warnings` bersih, skrip `scripts/test.ps1` lulus. | M1.09, M1.11, M1.12, M1.13 | *BLOCKED* |

---

## 3. Ringkasan Status Task M1
- **Total Task:** 14 Task (1 SPIKE, 13 DELIVERY)
- **Status:** 2 READY, 8 BLOCKED, 0 IN PROGRESS, 4 DONE
- **Progress:** `4 / 14` (28.6%)
| **M1.05** | **DELIVERY** | **Geometri WireSegment, Junction, & Tunnel Entities**<br/>• *DoD:* Struct `WireSegment` (geometri rute & dua endpoint), `Junction` (titik cabang topologis), `Tunnel` (label nama & scope), unit test pembentukan entitas lulus. | M1.02 | **READY** 🚀 |
| **M1.06** | **DELIVERY** | **CircuitDefinition & Project Hierarchy Container**<br/>• *DoD:* Struct `CircuitDefinition` (daftar instance, wire, junction, port) dan `Project` (akar multi-circuit), unit test pembuatan sirkuit bertingkat lulus. | M1.04, M1.05 | *BLOCKED* |
| **M1.07** | **DELIVERY** | **Serde JSON Serialization Engine**<br/>• *DoD:* Implementasi `Serialize` untuk semua entitas dokumen, output JSON rapi (*pretty printed*), unit test serialisasi menghasilkan string JSON valid. | M1.06 | *BLOCKED* |
| **M1.08** | **DELIVERY** | **Serde JSON Deserialization & Schema Validation**<br/>• *DoD:* Implementasi `Deserialize` dengan pemeriksaan `schema_version`, penanganan skema lama/tidak cocok dengan pesan error deskriptif, unit test parsing lulus. | M1.07 | *BLOCKED* |
| **M1.09** | **DELIVERY** | **Round-Trip Fixture Test Suite**<br/>• *DoD:* Test suite otomatis yang memuat berkas JSON fixture, mem-parse ke memory, me-reserialize, dan memvalidasi kesamaan semantik secara bit-exact (*round-trip*). | M1.08 | *BLOCKED* |
| **M1.10** | **DELIVERY** | **Diagnostic Framework Model**<br/>• *DoD:* Struct `Diagnostic`, enum `Severity` (Error, Warning, Info), enum `DiagnosticCode` stabil (`CONN-*`, `HIER-*`), pemetaan referensi objek visual (`primary`, `related`). | M1.01 | **READY** 🚀 |
| **M1.11** | **DELIVERY** | **Structural Validator: Unconnected Mandatory Pins**<br/>• *DoD:* Rule validator untuk mendeteksi pin masukan wajib yang mengambang/tidak terhubung, menghasilkan error `CONN-PIN-UNCONNECTED` dengan ID target yang akurat. | M1.06, M1.10 | *BLOCKED* |
| **M1.12** | **DELIVERY** | **Structural Validator: Dangling Wires & Endpoints**<br/>• *DoD:* Rule validator mendeteksi segmen kawat yang menggantung bebas tanpa terhubung ke pin/junction lain, menghasilkan warning `CONN-WIRE-DANGLING`. | M1.06, M1.10 | *BLOCKED* |
| **M1.13** | **DELIVERY** | **Structural Validator: Bus Width Mismatch Check**<br/>• *DoD:* Rule validator memeriksa ketidakcocokan lebar bit antara pin yang tersambung, menghasilkan error `CONN-WIDTH-MISMATCH`, serta memverifikasi integritas referensi ID. | M1.06, M1.10 | *BLOCKED* |
| **M1.14** | **DELIVERY** | **End-to-End M1 Headless Validation Suite & Scripts**<br/>• *DoD:* Seluruh unit test dan integration test M1 lulus 100%, `cargo fmt --check` dan `cargo clippy -- -D warnings` bersih, skrip `scripts/test.ps1` lulus. | M1.09, M1.11, M1.12, M1.13 | *BLOCKED* |

---

## 3. Ringkasan Status Task M1
- **Total Task:** 14 Task (1 SPIKE, 13 DELIVERY)
- **Status:** 3 READY, 8 BLOCKED, 0 IN PROGRESS, 3 DONE
- **Progress:** `3 / 14` (21.4%)
| **M1.04** | **DELIVERY** | **Component Definition & Instance Model**<br/>• *DoD:* Struct `ComponentDefinition`, `ComponentInstance`, registry pustaka primitif L0/L1 bawaan (AND, OR, NOT, XOR, DFF, RAM, dll.), unit test registry lookup lulus. | M1.03 | *BLOCKED* |
| **M1.05** | **DELIVERY** | **Geometri WireSegment, Junction, & Tunnel Entities**<br/>• *DoD:* Struct `WireSegment` (geometri rute & dua endpoint), `Junction` (titik cabang topologis), `Tunnel` (label nama & scope), unit test pembentukan entitas lulus. | M1.02 | **READY** 🚀 |
| **M1.06** | **DELIVERY** | **CircuitDefinition & Project Hierarchy Container**<br/>• *DoD:* Struct `CircuitDefinition` (daftar instance, wire, junction, port) dan `Project` (akar multi-circuit), unit test pembuatan sirkuit bertingkat lulus. | M1.04, M1.05 | *BLOCKED* |
| **M1.07** | **DELIVERY** | **Serde JSON Serialization Engine**<br/>• *DoD:* Implementasi `Serialize` untuk semua entitas dokumen, output JSON rapi (*pretty printed*), unit test serialisasi menghasilkan string JSON valid. | M1.06 | *BLOCKED* |
| **M1.08** | **DELIVERY** | **Serde JSON Deserialization & Schema Validation**<br/>• *DoD:* Implementasi `Deserialize` dengan pemeriksaan `schema_version`, penanganan skema lama/tidak cocok dengan pesan error deskriptif, unit test parsing lulus. | M1.07 | *BLOCKED* |
| **M1.09** | **DELIVERY** | **Round-Trip Fixture Test Suite**<br/>• *DoD:* Test suite otomatis yang memuat berkas JSON fixture, mem-parse ke memory, me-reserialize, dan memvalidasi kesamaan semantik secara bit-exact (*round-trip*). | M1.08 | *BLOCKED* |
| **M1.10** | **DELIVERY** | **Diagnostic Framework Model**<br/>• *DoD:* Struct `Diagnostic`, enum `Severity` (Error, Warning, Info), enum `DiagnosticCode` stabil (`CONN-*`, `HIER-*`), pemetaan referensi objek visual (`primary`, `related`). | M1.01 | **READY** 🚀 |
| **M1.11** | **DELIVERY** | **Structural Validator: Unconnected Mandatory Pins**<br/>• *DoD:* Rule validator untuk mendeteksi pin masukan wajib yang mengambang/tidak terhubung, menghasilkan error `CONN-PIN-UNCONNECTED` dengan ID target yang akurat. | M1.06, M1.10 | *BLOCKED* |
| **M1.12** | **DELIVERY** | **Structural Validator: Dangling Wires & Endpoints**<br/>• *DoD:* Rule validator mendeteksi segmen kawat yang menggantung bebas tanpa terhubung ke pin/junction lain, menghasilkan warning `CONN-WIRE-DANGLING`. | M1.06, M1.10 | *BLOCKED* |
| **M1.13** | **DELIVERY** | **Structural Validator: Bus Width Mismatch Check**<br/>• *DoD:* Rule validator memeriksa ketidakcocokan lebar bit antara pin yang tersambung, menghasilkan error `CONN-WIDTH-MISMATCH`, serta memverifikasi integritas referensi ID. | M1.06, M1.10 | *BLOCKED* |
| **M1.14** | **DELIVERY** | **End-to-End M1 Headless Validation Suite & Scripts**<br/>• *DoD:* Seluruh unit test dan integration test M1 lulus 100%, `cargo fmt --check` dan `cargo clippy -- -D warnings` bersih, skrip `scripts/test.ps1` lulus. | M1.09, M1.11, M1.12, M1.13 | *BLOCKED* |

---

## 3. Ringkasan Status Task M1
- **Total Task:** 14 Task (1 SPIKE, 13 DELIVERY)
- **Status:** 3 READY, 9 BLOCKED, 0 IN PROGRESS, 2 DONE
- **Progress:** `2 / 14` (14.3%)
| **M1.03** | **DELIVERY** | **Pin Contract, Direction, & Bus Width Types**<br/>• *DoD:* Struct `PinDeclaration`, `PinInstance`, enum `PinDirection` (Input, Output, Inout), enum `PinRole`, tipe `SignalWidth` (1..64), unit test validasi lebar pin lulus. | M1.02 | *BLOCKED* |
| **M1.04** | **DELIVERY** | **Component Definition & Instance Model**<br/>• *DoD:* Struct `ComponentDefinition`, `ComponentInstance`, registry pustaka primitif L0/L1 bawaan (AND, OR, NOT, XOR, DFF, RAM, dll.), unit test registry lookup lulus. | M1.03 | *BLOCKED* |
| **M1.05** | **DELIVERY** | **Geometri WireSegment, Junction, & Tunnel Entities**<br/>• *DoD:* Struct `WireSegment` (geometri rute & dua endpoint), `Junction` (titik cabang topologis), `Tunnel` (label nama & scope), unit test pembentukan entitas lulus. | M1.02 | *BLOCKED* |
| **M1.06** | **DELIVERY** | **CircuitDefinition & Project Hierarchy Container**<br/>• *DoD:* Struct `CircuitDefinition` (daftar instance, wire, junction, port) dan `Project` (akar multi-circuit), unit test pembuatan sirkuit bertingkat lulus. | M1.04, M1.05 | *BLOCKED* |
| **M1.07** | **DELIVERY** | **Serde JSON Serialization Engine**<br/>• *DoD:* Implementasi `Serialize` untuk semua entitas dokumen, output JSON rapi (*pretty printed*), unit test serialisasi menghasilkan string JSON valid. | M1.06 | *BLOCKED* |
| **M1.08** | **DELIVERY** | **Serde JSON Deserialization & Schema Validation**<br/>• *DoD:* Implementasi `Deserialize` dengan pemeriksaan `schema_version`, penanganan skema lama/tidak cocok dengan pesan error deskriptif, unit test parsing lulus. | M1.07 | *BLOCKED* |
| **M1.09** | **DELIVERY** | **Round-Trip Fixture Test Suite**<br/>• *DoD:* Test suite otomatis yang memuat berkas JSON fixture, mem-parse ke memory, me-reserialize, dan memvalidasi kesamaan semantik secara bit-exact (*round-trip*). | M1.08 | *BLOCKED* |
| **M1.10** | **DELIVERY** | **Diagnostic Framework Model**<br/>• *DoD:* Struct `Diagnostic`, enum `Severity` (Error, Warning, Info), enum `DiagnosticCode` stabil (`CONN-*`, `HIER-*`), pemetaan referensi objek visual (`primary`, `related`). | M1.01 | **READY** 🚀 |
| **M1.11** | **DELIVERY** | **Structural Validator: Unconnected Mandatory Pins**<br/>• *DoD:* Rule validator untuk mendeteksi pin masukan wajib yang mengambang/tidak terhubung, menghasilkan error `CONN-PIN-UNCONNECTED` dengan ID target yang akurat. | M1.06, M1.10 | *BLOCKED* |
| **M1.12** | **DELIVERY** | **Structural Validator: Dangling Wires & Endpoints**<br/>• *DoD:* Rule validator mendeteksi segmen kawat yang menggantung bebas tanpa terhubung ke pin/junction lain, menghasilkan warning `CONN-WIRE-DANGLING`. | M1.06, M1.10 | *BLOCKED* |
| **M1.13** | **DELIVERY** | **Structural Validator: Bus Width Mismatch Check**<br/>• *DoD:* Rule validator memeriksa ketidakcocokan lebar bit antara pin yang tersambung, menghasilkan error `CONN-WIDTH-MISMATCH`, serta memverifikasi integritas referensi ID. | M1.06, M1.10 | *BLOCKED* |
| **M1.14** | **DELIVERY** | **End-to-End M1 Headless Validation Suite & Scripts**<br/>• *DoD:* Seluruh unit test dan integration test M1 lulus 100%, `cargo fmt --check` dan `cargo clippy -- -D warnings` bersih, skrip `scripts/test.ps1` lulus. | M1.09, M1.11, M1.12, M1.13 | *BLOCKED* |

---

## 3. Ringkasan Status Task M1
- **Total Task:** 14 Task (1 SPIKE, 13 DELIVERY)
- **Status:** 2 READY, 11 BLOCKED, 0 IN PROGRESS, 1 DONE
- **Progress:** `1 / 14` (7.1%)
