# Digital Logic Forge (DLF) — Agent Guidelines & Working Contract

Dokumen ini berisi aturan wajib yang mengikat agen AI (Antigravity) dalam mengembangkan proyek **Digital Logic Forge**.

---

## 1. Prinsip Kode & Standar Rekayasa

1. **Hindari "Clever Code"**:
   - Tulis kode yang lugas, eksplisit, terbaca, dan mudah dipahami orang lain.
   - Jangan korbankan keterbacaan demi trik sintaksis satu baris yang rumit (*no unreadable one-liners*).
   - Penamaan variabel, fungsi, struct, dan enum harus deskriptif dan mencerminkan domain logika digital.
2. **Dokumentasi Kode (*Code Docs*) Adalah Wajib**:
   - Di Rust: setiap `pub struct`, `pub enum`, `pub fn`, dan `module` wajib memiliki rustdoc comment (`///` atau `//!`) yang menjelaskan tujuan, argumen, dan ekspektasi perilakunya.
   - Di Godot (GDScript): setiap skrip dan fungsi publik wajib memiliki komentar penjelasan dan *static type hints* lengkap.
3. **Standar Pengembang Profesional**:
   - Gunakan penanganan error yang eksplisit (`Result<T, E>`, `Option<T>`). Dilarang memakai `unwrap()` atau `expect()` secara serampangan pada kode produksi (kecuali test fixture atau invariant yang telah terbukti).
   - Kode harus lolos `cargo clippy --workspace --all-targets` dan `cargo fmt --check` tanpa peringatan (*zero warnings*).

---

## 2. Single Source of Truth (SSoT)

1. **Sumber Kebenaran Tunggal**:
   - Proposal Tugas Akhir: `docs/compiled circuit simulator.md`
   - Progress & Status Dashboard: `docs/tracking/DASHBOARD.md`
2. **Prinsip Riset Mandiri Sebelum Bertanya**:
   - Sebelum memulai task apa pun, **baca dan periksa sumber terlebih dahulu** (proposal, rancangan arsitektur, atau codebase yang ada).
   - Tanyakan kepada pengguna hanya jika ada ambiguitas spesifikasi, kontradiksi logis, atau keputusan strategis/TBD yang memang memerlukan persetujuan pemilik proyek.
3. **Sinkronisasi Proposal Markdown**:
   - Jika ada keputusan teknis atau penyesuaian arsitektur yang disepakati bersama pengguna, lakukan pembaruan (*update*) pada `docs/compiled circuit simulator.md` agar dokumen proposal selalu selaras dengan kondisi nyata di codebase (*bidirectional sync*).

---

## 3. Paradigma Elephant-Goldfish Model (EGM)

Proyek ini menerapkan prinsip **Elephant-Goldfish Model**:
- **The Elephant (Gajah - Long-term Context & Memory)**:
  - Diwakili oleh Dokumen Proposal, `DASHBOARD.md`, ADR (`docs/decisions/`), dan Sesi Utama.
  - Memastikan *bird-eye view*, semantik, arsitektur 3 representasi (Authoring, Compiled Artifact, Runtime State), dan 18 Keputusan Terkunci (D01–D18) tidak pernah terdistorsi atau dilupakan.
- **The Goldfish (Ikan Mas - Short-term Task Execution & Audit)**:
  - Diwakili oleh sub-task terisolasi, runner test, atau subagent audit (`egm-*`).
  - Bekerja secara tajam dan fokus pada satu irisan vertikal tanpa terdistraksi atau mengalami context rot.
  - Hasil kerja Goldfish segera disintesis dan dicatat kembali ke memori Elephant (`DASHBOARD.md`).

---

## 4. Pelacakan & Disiplin Sesi

1. Setiap sesi baru wajib diawali dengan memeriksa `docs/tracking/DASHBOARD.md` untuk mengidentifikasi milestone aktif dan sub-task berikutnya.
2. Setiap kali menyelesaikan sub-task atau milestone, perbarui checklist pada `docs/tracking/DASHBOARD.md` dan `README.md`.
3. Gunakan git commit atomik dengan pesan semantik yang menyertakan nomor milestone, contoh:
   `feat(core): implement levelized topo-sort for combinational netlist [M2/KF-04]`
