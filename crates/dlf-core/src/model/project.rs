//! # Proyek Rangkaian Multi-Sirkuit (Circuit Project Hierarchy)
//!
//! Modul ini mendefinisikan wadah akar dokumen (*Project*) yang menyimpan seluruh definisi
//! sirkuit, generator Stable ID deterministik, serta nomor versi skema dokumen.

use super::circuit::CircuitDefinition;
use super::id::{CircuitId, IdGenerator};
use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Versi skema dokumen proyek aktif saat ini.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Akar dokumen proyek rangkaian digital (*Authoring Project*).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Versi skema format berkas dokumen.
    pub schema_version: u32,
    /// Nama proyek sirkuit.
    pub name: String,
    /// Nilai ID berikutnya yang akan dialokasikan oleh generator.
    pub next_id: u64,
    /// ID sirkuit utama (*root / top-level circuit*).
    pub root_circuit_id: CircuitId,
    /// Seluruh lembar definisi sirkuit yang ada di dalam proyek.
    pub circuits: Vec<CircuitDefinition>,

    /// Generator ID internal (diabaikan dari serialisasi karena dikendalikan via `next_id`).
    #[serde(skip)]
    id_generator: IdGenerator,
}

impl Project {
    /// Membuat proyek baru dengan nama tertentu dan satu sirkuit utama default (`"main"`).
    pub fn new(name: impl Into<String>) -> Self {
        let mut id_generator = IdGenerator::new(IdGenerator::DEFAULT_START_ID);
        let root_circuit_id = id_generator.allocate_circuit_id();

        let root_circuit = CircuitDefinition::new(root_circuit_id, "main");

        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: name.into(),
            next_id: id_generator.peek_next(),
            root_circuit_id,
            circuits: vec![root_circuit],
            id_generator,
        }
    }

    /// Mengalokasikan ID mentah `u64` baru secara deterministik dari generator proyek.
    pub fn allocate_id(&mut self) -> u64 {
        // Sinkronkan generator dengan next_id jika baru dimuat dari deserialisasi
        self.id_generator.bump_to_at_least(self.next_id);
        let id = self.id_generator.allocate_raw();
        self.next_id = self.id_generator.peek_next();
        id
    }

    /// Mengambil referensi generator ID proyek.
    pub fn id_generator_mut(&mut self) -> &mut IdGenerator {
        self.id_generator.bump_to_at_least(self.next_id);
        &mut self.id_generator
    }

    /// Menyinkronkan status generator setelah deserialisasi atau modifikasi manual `next_id`.
    pub fn sync_id_generator(&mut self) {
        self.id_generator = IdGenerator::new(self.next_id);
    }

    /// Membuat dan menambahkan lembar definisi sirkuit baru ke dalam proyek.
    pub fn create_circuit(&mut self, name: impl Into<String>) -> CircuitId {
        let circuit_id = CircuitId::new(self.allocate_id());
        self.circuits.push(CircuitDefinition::new(circuit_id, name));
        circuit_id
    }

    /// Mengambil referensi sirkuit utama (*root circuit*).
    pub fn get_root_circuit(&self) -> Result<&CircuitDefinition, CoreError> {
        self.get_circuit(self.root_circuit_id).ok_or_else(|| {
            CoreError::ValidationError(format!(
                "Root circuit dengan ID {} tidak ditemukan di dalam proyek",
                self.root_circuit_id
            ))
        })
    }

    /// Mengambil referensi mutable sirkuit utama (*root circuit*).
    pub fn get_root_circuit_mut(&mut self) -> Result<&mut CircuitDefinition, CoreError> {
        let root_id = self.root_circuit_id;
        self.get_circuit_mut(root_id).ok_or_else(|| {
            CoreError::ValidationError(format!(
                "Root circuit dengan ID {} tidak ditemukan di dalam proyek",
                root_id
            ))
        })
    }

    /// Mencari definisi sirkuit berdasarkan `CircuitId`.
    pub fn get_circuit(&self, circuit_id: CircuitId) -> Option<&CircuitDefinition> {
        self.circuits.iter().find(|c| c.id == circuit_id)
    }

    /// Mencari definisi sirkuit mutable berdasarkan `CircuitId`.
    pub fn get_circuit_mut(&mut self, circuit_id: CircuitId) -> Option<&mut CircuitDefinition> {
        self.circuits.iter_mut().find(|c| c.id == circuit_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_initialization_and_circuit_creation() {
        let mut project = Project::new("Test CPU Project");

        assert_eq!(project.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(project.name, "Test CPU Project");
        assert_eq!(project.circuits.len(), 1);

        let root = project.get_root_circuit().expect("Root harus ada");
        assert_eq!(root.name, "main");
        assert_eq!(root.id, project.root_circuit_id);

        let alu_id = project.create_circuit("ALU");
        assert_eq!(project.circuits.len(), 2);
        assert_ne!(alu_id, project.root_circuit_id);

        let alu = project.get_circuit(alu_id).expect("ALU harus ada");
        assert_eq!(alu.name, "ALU");
    }

    #[test]
    fn test_project_id_allocation() {
        let mut project = Project::new("Alloc Test");
        let next_start = project.next_id;

        let id1 = project.allocate_id();
        let id2 = project.allocate_id();

        assert_eq!(id1, next_start);
        assert_eq!(id2, next_start + 1);
        assert_eq!(project.next_id, next_start + 2);
    }
}
