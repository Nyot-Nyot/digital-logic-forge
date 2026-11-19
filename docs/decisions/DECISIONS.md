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
