# Digital Logic Forge (DLF)

> **Simulator Rangkaian Logika Digital Visual dengan Compiled Execution untuk Komputer Buatan Pengguna**  
> *Tugas Akhir Berbasis Project — Program Studi Informatika, Universitas Negeri Padang (2026)*  
> Penulis: **Dzaki Sultan Rabbani** (NIM: 23343035)

---

## 📌 Status & Posisi Proyek Saat Ini

| Parameter | Keterangan |
| :--- | :--- |
| **Milestone Aktif** | **M3: First Compiled Simulation** 🚀 *(M0, M1, & M2 Selesai & Terverifikasi)* |
| **Fokus Pekerjaan** | Snapshot -> Compile -> IR Lowering -> Runtime B1 vs B0 Differential Oracle |
| **Status Engine** | Netlist kanonis DSU, validasi elektris, dependency graph, Tarjan SCC, levelization, & hierarki lulus (65 tests) |
| **Single Source of Truth** | [Proposal TA (Markdown)](docs/compiled%20circuit%20simulator.md) & [Progress Dashboard](docs/tracking/DASHBOARD.md) |

---

## 🔄 Visualisasi Alur Kerja (Development Workflow)

Proyek ini dibangun secara inkremental dan digerakkan oleh risiko (*risk-driven capability-based milestones*) mengikuti 7 prinsip rekayasa di Bab III proposal.

### 1. Peta Jalan 10 Milestone (M0 - M9)

```mermaid
flowchart LR
    M0["M0: Baseline (Done)"] --> M1["M1: Headless Model (Done)"]
    M1 --> M2["M2: Connectivity (Done)"]
    M2 --> M3["M3: First Compiled"]
    M3 --> M4["M4: Stateful Sim"]
    M4 --> M5["M5: Editor Slice"]
    M5 --> M6["M6: Interactive Worker"]
    M6 --> M7["M7: Benchmark"]
    M7 --> M8["M8: CPU Demo (P1)"]
    M8 --> M9["M9: Delivery"]

    classDef done fill:#16a34a,stroke:#15803d,stroke-width:2px,color:#fff;
    classDef active fill:#2563eb,stroke:#1d4ed8,stroke-width:2px,color:#fff;
    classDef pending fill:#374151,stroke:#4b5563,stroke-width:1px,color:#9ca3af;
    class M0,M1,M2 done;
    class M3 active;
    class M4,M5,M6,M7,M8,M9 pending;
```

### 2. Siklus Eksekusi per Milestone (The 3-Phase Milestone Loop)

Setiap tahapan milestone dieksekusi secara disiplin melalui siklus 3 fase berikut:

```mermaid
flowchart TD
    subgraph PreSprint ["1. PRE-SPRINT (30 Menit)"]
        A1["Baca Proposal Milestone Aktif"] --> A2["Glimpse Milestone Berikutnya"]
        A2 --> A3["Dekomposisi 8-15 Task Terukur"]
        A3 --> A4["Klasifikasi: SPIKE vs DELIVERY"]
        A4 --> A5["Petakan Dependensi (READY vs BLOCKED)"]
        A5 --> A6["Simpan di docs/tracking/PROGRESS.md"]
    end

    subgraph DailyLoop ["2. DAILY LOOP (Sepanjang Milestone)"]
        B1["Pilih 1 Task READY -> IN PROGRESS"] --> B2{"Tipe Task?"}
        B2 -->|SPIKE| B3["Eksperimen & Catat di DECISIONS.md"]
        B2 -->|DELIVERY| B4["Tulis Kode + Test (fmt & clippy bersih)"]
        B3 --> B5["Tandai DONE di PROGRESS.md"]
        B4 --> B5
        B5 --> B6["Atomic Git Commit Semantik"]
    end

    subgraph PostSprint ["3. POST-SPRINT (30 Menit)"]
        C1["Verifikasi Semua Task DONE (DoD Lulus)"] --> C2["Tulis Evaluasi di RETROSPECTIVE.md"]
        C2 --> C3["Update DASHBOARD.md & README.md"]
        C3 --> C4["Patch Proposal Markdown (jika ada sync)"]
        C4 --> C5["Lanjut ke Pre-Sprint Milestone Berikutnya"]
    end

    PreSprint --> DailyLoop
    DailyLoop --> PostSprint
```

---

## 🏛️ Arsitektur Tiga Representasi Data

DLF memisahkan secara ketat tiga representasi data agar eksekusi pada *hot loop* tidak terbebani struktur data editor visual:

```mermaid
flowchart TD
    subgraph EditMode ["Mode Edit (Godot UI)"]
        AD["Authoring Document<br/>(Hierarki, instance, geometri wire, mutable)"]
    end

    subgraph CompilePhase ["Tahap Kompilasi (Rust Core)"]
        CS["Circuit Snapshot<br/>(Immutable copy)"] --> CP["Compiler Pipeline<br/>(Validation, Net resolution, Levelization, IR lowering)"]
        CP --> CA["Compiled Artifact<br/>(Compact bytecode IR, contiguous memory layout, immutable)"]
    end

    subgraph SimMode ["Mode Simulasi (Background Thread)"]
        SW["Simulation Worker<br/>(Thread terpisah)"]
        RS["Runtime State<br/>(Signal slots, register state, RAM/ROM, mutable)"]
        CA --> SW
        SW <--> RS
    end

    AD -->|Tekan Simulate| CS
    SW -.->|Snapshot tidak berubah / Coalesced| EditMode
```

---

## 📂 Struktur Repositori

```
digital-logic-forge/
├── AGENTS.md                  # Working contract & pedoman AI
├── README.md                  # Dokumentasi utama & visualisasi workflow
├── docs/
│   ├── compiled circuit simulator.md  # Naskah proposal (Markdown SSoT)
│   ├── compiled circuit simulator.pdf # Dokumen acuan asli
│   ├── tracking/              # Dashboard progres, status risk, & workload
│   ├── decisions/             # Architecture Decision Records (ADR)
│   └── specs/                 # Spesifikasi formal format file & IR
├── crates/
│   ├── dlf-core/              # Logika simulasi & kompilasi murni (Rust)
│   └── dlf-godot/             # GDExtension adapter tipis (Rust -> Godot)
├── app/
│   └── godot/                 # Proyek UI Godot 4.7.2
├── tests/
│   └── fixtures/              # Rangkaian uji & golden outputs
└── scripts/                   # Skrip otomasi PowerShell
```

---

## 🛠️ Standar & Disiplin Pengembangan

- **No Clever Code**: Kode harus jelas, ekspresif, dan mudah dipahami.
- **Wajib Code Docs**: Setiap fungsi publik, modul, dan tipe data wajib memiliki dokumentasi (`///` di Rust).
- **Correctness First**: Optimasi performa hanya dipertimbangkan setelah lulus uji kebenaran (*correctness suite*).
- **Elephant-Goldfish Model (EGM)**: Memori jangka panjang dipegang oleh Proposal, Dashboard, dan ADR (Elephant); eksekusi tugas mikro dan audit kualitas dijalankan oleh sub-task / subagent spesifik (Goldfish).
