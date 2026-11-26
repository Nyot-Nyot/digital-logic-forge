//! # Artefak Kompilasi IR dan Pemetaan Simbolik
//!
//! Modul ini mendefinisikan struktur `CompiledArtifact`, `SignalBinding`, dan `SourceMap`
//! sebagai representasi terkompilasi (Representasi #2 dalam arsitektur DLF) sesuai
//! Tabel 3.26 proposal serta `[SPIKE-004]`.

use super::instruction::{CompiledOp, Opcode};
use crate::connectivity::NetId;
use crate::diagnostic::SourceRef;
use crate::error::CoreError;
use crate::model::id::{InstanceId, PinId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;

/// Pemetaan antara titik I/O rangkaian (port masukan/keluaran atau probe) dan indeks slot runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBinding {
    /// Pengenal port atau pin I/O pada dokumen sumber.
    pub pin_id: PinId,
    /// Nama tampilan sinyal (misal `"A"`, `"SUM"`, `"CLK"`).
    pub name: String,
    /// Indeks slot sinyal pada array runtime `SignalValue`.
    pub slot: usize,
    /// Lebar bit sinyal (1 hingga 64).
    pub width: u8,
}

impl SignalBinding {
    /// Membuat entri pemetaan sinyal baru.
    pub fn new(pin_id: PinId, name: impl Into<String>, slot: usize, width: u8) -> Self {
        Self {
            pin_id,
            name: name.into(),
            slot,
            width,
        }
    }
}

/// Peta balik (*Source Map*) yang menghubungkan slot dan instruksi runtime ke objek editor.
///
/// Struktur ini hanya dibaca oleh diagnostik, *debugger*, dan *probe* UI, tidak pernah
/// disentuh oleh jalur eksekusi panas (*hot loop*) simulasi.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMap {
    /// Pemetaan dari indeks slot sinyal ke `NetId` kanonis.
    pub slot_to_net: BTreeMap<usize, NetId>,
    /// Pemetaan dari indeks slot sinyal ke referensi sumber visual primer.
    pub slot_to_source: BTreeMap<usize, SourceRef>,
    /// Pemetaan dari indeks operasi IR ke `InstanceId` komponen sumber.
    pub op_to_instance: BTreeMap<usize, InstanceId>,
}

impl SourceMap {
    /// Membuat `SourceMap` kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mendaftarkan hubungan antara slot sinyal, `NetId`, dan referensi sumber visual.
    pub fn insert_slot(&mut self, slot: usize, net_id: NetId, source_ref: SourceRef) {
        self.slot_to_net.insert(slot, net_id);
        self.slot_to_source.insert(slot, source_ref);
    }

    /// Mendaftarkan hubungan antara indeks operasi IR dan komponen asalnya.
    pub fn insert_op(&mut self, op_index: usize, instance_id: InstanceId) {
        self.op_to_instance.insert(op_index, instance_id);
    }
}

/// Artefak hasil kompilasi lengkap dari suatu rangkaian kombinasional.
///
/// Berisi seluruh array datar yang dibutuhkan untuk menginisialisasi `CompiledRuntime`
/// tanpa bergantung pada dokumen editor atau objek UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledArtifact {
    /// Daftar instruksi IR terurut menurut level ketergantungan (*levelized*).
    pub operations: Vec<CompiledOp>,
    /// Batas rentang indeks `[start..end)` pada `operations` untuk setiap level kedalaman.
    pub level_ranges: Vec<Range<usize>>,
    /// Kumpulan nilai konstanta 4-nilai `(val, unk)` yang dimuat oleh `Opcode::Const`.
    pub const_pool: Vec<(u64, u64)>,
    /// Jumlah total slot sinyal yang harus dialokasikan oleh runtime.
    pub slot_count: usize,
    /// Lebar bit untuk masing-masing slot sinyal (`slot_widths.len() == slot_count`).
    pub slot_widths: Vec<u8>,
    /// Daftar pemetaan port masukan eksternal ke slot runtime.
    pub input_bindings: Vec<SignalBinding>,
    /// Daftar pemetaan port keluaran eksternal ke slot runtime.
    pub output_bindings: Vec<SignalBinding>,
    /// Peta balik untuk diagnostik dan inspeksi *probe*.
    pub source_map: SourceMap,
}

impl CompiledArtifact {
    /// Memverifikasi seluruh invarian struktural `CompiledArtifact` sebelum dijalankan runtime.
    ///
    /// Memastikan tidak ada indeks slot atau indeks `const_pool` yang *out-of-bounds*,
    /// `level_ranges` mencakup seluruh `operations` secara berurutan tanpa celah, dan
    /// seluruh lebar bit berada dalam rentang valid `1..=64`.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::ValidationError` apabila ditemukan pelanggaran invarian IR.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.slot_widths.len() != self.slot_count {
            return Err(CoreError::ValidationError(format!(
                "Panjang slot_widths ({}) tidak sama dengan slot_count ({})",
                self.slot_widths.len(),
                self.slot_count
            )));
        }

        for (slot_idx, &w) in self.slot_widths.iter().enumerate() {
            if !(1..=64).contains(&w) {
                return Err(CoreError::ValidationError(format!(
                    "Lebar slot {} tidak valid: {} (harus 1..=64)",
                    slot_idx, w
                )));
            }
        }

        // Validasi level_ranges berurutan dan menutupi 0..operations.len()
        let mut expected_start = 0;
        for (lvl, range) in self.level_ranges.iter().enumerate() {
            if range.start != expected_start || range.end < range.start {
                return Err(CoreError::ValidationError(format!(
                    "LevelRange[{}] ({}..{}) tidak kontinu dari {}",
                    lvl, range.start, range.end, expected_start
                )));
            }
            expected_start = range.end;
        }
        if expected_start != self.operations.len() {
            return Err(CoreError::ValidationError(format!(
                "Total operasi pada level_ranges ({}) tidak cocok dengan panjang operations ({})",
                expected_start,
                self.operations.len()
            )));
        }

        // Validasi setiap instruksi IR
        for (op_idx, op) in self.operations.iter().enumerate() {
            self.validate_op(op_idx, op)?;
        }

        // Validasi binding masukan & keluaran
        for binding in self
            .input_bindings
            .iter()
            .chain(self.output_bindings.iter())
        {
            if binding.slot >= self.slot_count {
                return Err(CoreError::ValidationError(format!(
                    "Binding '{}' merujuk slot {} di luar batas slot_count ({})",
                    binding.name, binding.slot, self.slot_count
                )));
            }
            if !(1..=64).contains(&binding.width) {
                return Err(CoreError::ValidationError(format!(
                    "Lebar binding '{}' tidak valid: {} (harus 1..=64)",
                    binding.name, binding.width
                )));
            }
        }

        Ok(())
    }

    /// Memvalidasi satu instruksi `CompiledOp` terhadap batas slot dan lebar bit.
    fn validate_op(&self, op_idx: usize, op: &CompiledOp) -> Result<(), CoreError> {
        if !(1..=64).contains(&op.width) {
            return Err(CoreError::ValidationError(format!(
                "Operasi #{} ({}) memiliki lebar tidak valid: {}",
                op_idx, op.opcode, op.width
            )));
        }

        if op.dst >= self.slot_count {
            return Err(CoreError::ValidationError(format!(
                "Operasi #{} ({}) memiliki dst={} di luar slot_count={}",
                op_idx, op.opcode, op.dst, self.slot_count
            )));
        }

        let check_slot = |slot: usize, label: &str| -> Result<(), CoreError> {
            if slot >= self.slot_count {
                Err(CoreError::ValidationError(format!(
                    "Operasi #{} ({}) memiliki {}={} di luar slot_count={}",
                    op_idx, op.opcode, label, slot, self.slot_count
                )))
            } else {
                Ok(())
            }
        };

        match op.opcode {
            Opcode::Buf | Opcode::Not => {
                check_slot(op.src_a, "src_a")?;
            }
            Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Nand | Opcode::Nor | Opcode::Xnor => {
                check_slot(op.src_a, "src_a")?;
                check_slot(op.src_b, "src_b")?;
            }
            Opcode::Const => {
                if op.src_a >= self.const_pool.len() {
                    return Err(CoreError::ValidationError(format!(
                        "Operasi #{} (CONST) merujuk const_pool[{}] di luar panjang={}",
                        op_idx,
                        op.src_a,
                        self.const_pool.len()
                    )));
                }
            }
            Opcode::Mux => {
                check_slot(op.src_a, "src_a")?;
                check_slot(op.src_b, "src_b")?;
                check_slot(op.src_c, "src_c")?;
            }
            Opcode::Slice => {
                check_slot(op.src_a, "src_a")?;
                if op.src_b >= 64 {
                    return Err(CoreError::ValidationError(format!(
                        "Operasi #{} (SLICE) memiliki offset src_b={} >= 64",
                        op_idx, op.src_b
                    )));
                }
            }
            Opcode::Concat => {
                check_slot(op.src_a, "src_a")?;
                check_slot(op.src_b, "src_b")?;
                if op.src_c > 64 {
                    return Err(CoreError::ValidationError(format!(
                        "Operasi #{} (CONCAT) memiliki low_width src_c={} > 64",
                        op_idx, op.src_c
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_valid_artifact() -> CompiledArtifact {
        CompiledArtifact {
            operations: vec![
                CompiledOp::binary(Opcode::And, 2, 0, 1, 1),
                CompiledOp::unary(Opcode::Not, 3, 2, 1),
            ],
            level_ranges: vec![0..1, 1..2],
            const_pool: vec![],
            slot_count: 4,
            slot_widths: vec![1, 1, 1, 1],
            input_bindings: vec![
                SignalBinding::new(PinId::new(10), "A", 0, 1),
                SignalBinding::new(PinId::new(11), "B", 1, 1),
            ],
            output_bindings: vec![SignalBinding::new(PinId::new(12), "Y", 3, 1)],
            source_map: SourceMap::new(),
        }
    }

    #[test]
    fn test_artifact_validate_ok() {
        let artifact = sample_valid_artifact();
        assert!(artifact.validate().is_ok());
    }

    #[test]
    fn test_artifact_validate_detects_oob_slot() {
        let mut artifact = sample_valid_artifact();
        artifact.operations[0].dst = 99;
        let err = artifact.validate().unwrap_err();
        assert!(matches!(err, CoreError::ValidationError(_)));
    }

    #[test]
    fn test_artifact_validate_detects_non_contiguous_levels() {
        let mut artifact = sample_valid_artifact();
        artifact.level_ranges.pop(); // Kurang level kedua
        let err = artifact.validate().unwrap_err();
        assert!(matches!(err, CoreError::ValidationError(_)));
    }

    #[test]
    fn test_artifact_validate_detects_invalid_width() {
        let mut artifact = sample_valid_artifact();
        artifact.operations[0].width = 65;
        assert!(artifact.validate().is_err());
    }
}
