//! # Model Dokumen Rangkaian (Circuit Document Model)
//!
//! Modul ini mengimplementasikan representasi dokumen penyuntingan (*Authoring Document*)
//! untuk simulator logika digital DLF sesuai Bab 3.3.2 dan Bab 3.10 proposal.
//!
//! Model ini murni bersifat struktural dan visual; netlist, graf dependensi,
//! dan representasi eksekusi runtime tidak disimpan di sini, melainkan dihasilkan
//! pada fase kompilasi frontend.

pub mod id;
