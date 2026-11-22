//! # Validasi Model dan Pemeriksaan Aturan Struktural (Model Validation)
//!
//! Modul ini mengimplementasikan aturan validasi integritas dokumen rangkaian (*structural validation*)
//! sebelum sirkuit dikompilasi atau disimulasikan sesuai Bab 3.4.2 dan Bab 3.10.8 proposal.

pub mod electrical;
pub mod structural;

pub use electrical::{validate_multi_drivers, validate_netlist_electrical, validate_undriven_nets};
pub use structural::{validate_circuit, validate_project};

use crate::diagnostic::{Diagnostic, Severity};

/// Laporan hasil validasi dokumen sirkuit yang merangkum seluruh temuan diagnostik.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    /// Daftar seluruh diagnostik yang terdeteksi.
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    /// Membuat laporan validasi baru yang kosong.
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Menambahkan diagnostik ke dalam laporan.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Menambahkan seluruh diagnostik dari laporan lain.
    pub fn merge(&mut self, other: ValidationReport) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Memeriksa apakah terdapat sekurangnya satu kesalahan fatal (*Error*).
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Jumlah kesalahan fatal (*Error*).
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Jumlah peringatan (*Warning*).
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Memeriksa apakah laporan bersih tanpa kesalahan maupun peringatan.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
