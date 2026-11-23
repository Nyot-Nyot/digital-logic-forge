//! # Digital Logic Forge (DLF) Core Engine
//!
//! `dlf-core` adalah pustaka inti yang bertanggung jawab atas model rangkaian digital,
//! validasi struktural, resolusi konektivitas, kompilasi execution plan (IR),
//! serta eksekusi simulasi berbasis siklus (*cycle-based simulation runtime*).
//!
//! Crate ini bersifat *headless* murni (independen dari Godot Engine) agar dapat
//! diuji secara cepat, deterministik, dan dapat direproduksi pada lingkungan CI/CLI.

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Versi semantik dari mesin inti DLF.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Modul model dokumen rangkaian dan entitas penyuntingan visual.
pub mod model;

/// Modul resolusi konektivitas dan pembentukan netlist kanonis.
pub mod connectivity;

/// Modul kerangka kerja diagnostik dan pelaporan error terstruktur.
pub mod diagnostic;

/// Modul validasi aturan struktural dokumen rangkaian.
pub mod validation;

/// Modul analisis semantik, graf ketergantungan, dan penjadwalan eksekusi.
pub mod analysis;

/// Modul worker simulasi latar belakang dan protokol snapshot.
pub mod worker;

/// Modul error untuk penanganan kesalahan eksplisit di seluruh domain DLF.
pub mod error {
    use thiserror::Error;

    /// Enumerasi kesalahan umum yang dapat terjadi pada mesin inti.
    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum CoreError {
        /// Terjadi kesalahan validasi dokumen rangkaian.
        #[error("Kesalahan validasi struktural: {0}")]
        ValidationError(String),

        /// Terjadi kesalahan pada alur eksekusi runtime.
        #[error("Kesalahan runtime eksekusi: {0}")]
        RuntimeError(String),
    }
}

/// Fungsi pembantu untuk verifikasi kesiapan pustaka inti.
///
/// Mengembalikan string status dan versi pustaka inti untuk memastikan
/// binding dan workspace terhubung dengan baik.
pub fn engine_status() -> String {
    format!("Digital Logic Forge Core v{} [OK]", CORE_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_engine_status() {
        let status = engine_status();
        assert!(status.contains("Digital Logic Forge Core"));
        assert!(status.contains("[OK]"));
    }
}
