//! # Compiled Simulation Runtime Engine (Baseline B1)
//!
//! Modul ini mengimplementasikan `CompiledRuntime` (Baseline B1 pada Tabel 3.17 proposal)
//! yang mengeksekusi `CompiledArtifact` secara langsung di atas array memori kontigu
//! (`values: Vec<u64>`, `unknowns: Vec<u64>`) tanpa pencarian nama atau traversal objek.

use super::value::LogicValue;
use crate::connectivity::NetId;
use crate::error::CoreError;
use crate::ir::{CompiledArtifact, CompiledOp, Opcode, width_mask};
use crate::model::id::PinId;
use std::collections::BTreeMap;

/// Evaluator simulasi terkompilasi berkinerja tinggi berbasis array kontigu (B1).
#[derive(Debug, Clone)]
pub struct CompiledRuntime {
    artifact: CompiledArtifact,
    /// Array nilai biner kontigu untuk seluruh slot sinyal (`SignalValue`).
    values: Vec<u64>,
    /// Array mask ketidakpastian 4-nilai kontigu untuk seluruh slot sinyal.
    unknowns: Vec<u64>,
    /// Indeks cepat dari nama port masukan ke `(slot, width)`.
    input_by_name: BTreeMap<String, (usize, u8)>,
    /// Indeks cepat dari `PinId` port masukan ke `(slot, width)`.
    input_by_pin: BTreeMap<PinId, (usize, u8)>,
    /// Indeks cepat dari nama port keluaran ke `(slot, width)`.
    output_by_name: BTreeMap<String, (usize, u8)>,
    /// Indeks cepat dari `PinId` port keluaran ke `(slot, width)`.
    output_by_pin: BTreeMap<PinId, (usize, u8)>,
    /// Indeks balik dari `NetId` ke indeks slot untuk fitur *probe*.
    net_to_slot: BTreeMap<NetId, usize>,
}

impl CompiledRuntime {
    /// Menginisialisasi `CompiledRuntime` dari `CompiledArtifact`.
    ///
    /// Memvalidasi invarian artefak, mengalokasikan array datar `values` dan `unknowns`,
    /// serta membangun tabel indeks binding I/O. Seluruh slot diinisialisasi ke `X`
    /// sebelum input ditetapkan.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::ValidationError` apabila artefak tidak lolos validasi invarian.
    pub fn from_artifact(artifact: CompiledArtifact) -> Result<Self, CoreError> {
        artifact.validate()?;

        let slot_count = artifact.slot_count;
        let values = vec![0u64; slot_count];
        let mut unknowns = Vec::with_capacity(slot_count);
        for &w in &artifact.slot_widths {
            unknowns.push(width_mask(w));
        }

        let mut input_by_name = BTreeMap::new();
        let mut input_by_pin = BTreeMap::new();
        for b in &artifact.input_bindings {
            input_by_name.insert(b.name.clone(), (b.slot, b.width));
            input_by_pin.insert(b.pin_id, (b.slot, b.width));
        }

        let mut output_by_name = BTreeMap::new();
        let mut output_by_pin = BTreeMap::new();
        for b in &artifact.output_bindings {
            output_by_name.insert(b.name.clone(), (b.slot, b.width));
            output_by_pin.insert(b.pin_id, (b.slot, b.width));
        }

        let mut net_to_slot = BTreeMap::new();
        for (&slot, &net_id) in &artifact.source_map.slot_to_net {
            net_to_slot.insert(net_id, slot);
        }

        Ok(Self {
            artifact,
            values,
            unknowns,
            input_by_name,
            input_by_pin,
            output_by_name,
            output_by_pin,
            net_to_slot,
        })
    }

    /// Mengembalikan jumlah slot sinyal yang dialokasikan oleh runtime.
    #[inline]
    pub fn slot_count(&self) -> usize {
        self.values.len()
    }

    /// Menetapkan nilai sinyal masukan berdasarkan nama port (*SetInput by name*).
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika nama port masukan tidak terdaftar.
    pub fn set_input(&mut self, name: &str, value: LogicValue) -> Result<(), CoreError> {
        let &(slot, width) = self.input_by_name.get(name).ok_or_else(|| {
            CoreError::RuntimeError(format!("Input binding '{}' tidak ditemukan", name))
        })?;
        self.write_slot(slot, width, value);
        Ok(())
    }

    /// Menetapkan nilai sinyal masukan berdasarkan `PinId` port (*SetInput by PinId*).
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika `PinId` masukan tidak terdaftar.
    pub fn set_input_by_pin(&mut self, pin_id: PinId, value: LogicValue) -> Result<(), CoreError> {
        let &(slot, width) = self.input_by_pin.get(&pin_id).ok_or_else(|| {
            CoreError::RuntimeError(format!("Input binding untuk {} tidak ditemukan", pin_id))
        })?;
        self.write_slot(slot, width, value);
        Ok(())
    }

    /// Membaca nilai sinyal keluaran berdasarkan nama port (*GetOutput by name*).
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika nama port keluaran tidak terdaftar.
    pub fn get_output(&self, name: &str) -> Result<LogicValue, CoreError> {
        let &(slot, width) = self.output_by_name.get(name).ok_or_else(|| {
            CoreError::RuntimeError(format!("Output binding '{}' tidak ditemukan", name))
        })?;
        Ok(self.read_slot(slot, width))
    }

    /// Membaca nilai sinyal keluaran berdasarkan `PinId` port (*GetOutput by PinId*).
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika `PinId` keluaran tidak terdaftar.
    pub fn get_output_by_pin(&self, pin_id: PinId) -> Result<LogicValue, CoreError> {
        let &(slot, width) = self.output_by_pin.get(&pin_id).ok_or_else(|| {
            CoreError::RuntimeError(format!("Output binding untuk {} tidak ditemukan", pin_id))
        })?;
        Ok(self.read_slot(slot, width))
    }

    /// Menginspeksi nilai sinyal internal berdasarkan indeks slot (*Probe by slot*).
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika indeks slot di luar batas.
    pub fn probe_slot(&self, slot: usize) -> Result<LogicValue, CoreError> {
        let width = self
            .artifact
            .slot_widths
            .get(slot)
            .copied()
            .ok_or_else(|| {
                CoreError::RuntimeError(format!("Slot probe {} di luar batas runtime", slot))
            })?;
        Ok(self.read_slot(slot, width))
    }

    /// Menginspeksi nilai sinyal internal berdasarkan `NetId` kanonis (*Probe by NetId*).
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika `NetId` tidak terpetakan di `SourceMap`.
    pub fn probe_net(&self, net_id: NetId) -> Result<LogicValue, CoreError> {
        let &slot = self.net_to_slot.get(&net_id).ok_or_else(|| {
            CoreError::RuntimeError(format!("{} tidak ditemukan pada SourceMap", net_id))
        })?;
        self.probe_slot(slot)
    }

    /// Mengeksekusi satu siklus penuh evaluasi kombinasional berurutan level.
    ///
    /// Setiap level pada `level_ranges` dieksekusi secara berurutan pada array kontigu
    /// sehingga seluruh dependensi masukan dijamin telah selesai dihitung.
    pub fn step_cycle(&mut self) {
        for level_idx in 0..self.artifact.level_ranges.len() {
            let range = self.artifact.level_ranges[level_idx].clone();
            for op_idx in range {
                let op = self.artifact.operations[op_idx];
                self.execute_op(&op);
            }
        }
    }

    /// Menulis nilai sinyal ke slot tertentu dengan menerapkan *width masking*.
    #[inline]
    fn write_slot(&mut self, slot: usize, width: u8, value: LogicValue) {
        let mask = width_mask(width);
        self.values[slot] = value.val & mask;
        self.unknowns[slot] = value.unk & mask;
    }

    /// Membaca nilai sinyal dari slot tertentu dengan menerapkan *width masking*.
    #[inline]
    fn read_slot(&self, slot: usize, width: u8) -> LogicValue {
        let mask = width_mask(width);
        LogicValue {
            val: self.values[slot] & mask,
            unk: self.unknowns[slot] & mask,
        }
    }

    /// Mengeksekusi satu instruksi `CompiledOp` di atas array `values` dan `unknowns`.
    #[inline]
    fn execute_op(&mut self, op: &CompiledOp) {
        let mask = width_mask(op.width);
        let (out_val, out_unk) = match op.opcode {
            Opcode::Buf => (self.values[op.src_a] & mask, self.unknowns[op.src_a] & mask),
            Opcode::Not => eval_not(self.values[op.src_a], self.unknowns[op.src_a], mask),
            Opcode::And => eval_and(
                self.values[op.src_a],
                self.unknowns[op.src_a],
                self.values[op.src_b],
                self.unknowns[op.src_b],
                mask,
            ),
            Opcode::Or => eval_or(
                self.values[op.src_a],
                self.unknowns[op.src_a],
                self.values[op.src_b],
                self.unknowns[op.src_b],
                mask,
            ),
            Opcode::Xor => eval_xor(
                self.values[op.src_a],
                self.unknowns[op.src_a],
                self.values[op.src_b],
                self.unknowns[op.src_b],
                mask,
            ),
            Opcode::Nand => {
                let (v, u) = eval_and(
                    self.values[op.src_a],
                    self.unknowns[op.src_a],
                    self.values[op.src_b],
                    self.unknowns[op.src_b],
                    mask,
                );
                eval_not(v, u, mask)
            }
            Opcode::Nor => {
                let (v, u) = eval_or(
                    self.values[op.src_a],
                    self.unknowns[op.src_a],
                    self.values[op.src_b],
                    self.unknowns[op.src_b],
                    mask,
                );
                eval_not(v, u, mask)
            }
            Opcode::Xnor => {
                let (v, u) = eval_xor(
                    self.values[op.src_a],
                    self.unknowns[op.src_a],
                    self.values[op.src_b],
                    self.unknowns[op.src_b],
                    mask,
                );
                eval_not(v, u, mask)
            }
            Opcode::Const => {
                let (c_val, c_unk) = self.artifact.const_pool[op.src_a];
                (c_val & mask, c_unk & mask)
            }
            Opcode::Mux => eval_mux(
                self.values[op.src_a],
                self.unknowns[op.src_a],
                self.values[op.src_b],
                self.unknowns[op.src_b],
                self.values[op.src_c],
                self.unknowns[op.src_c],
                mask,
            ),
            Opcode::Slice => {
                let offset = op.src_b as u32;
                (
                    (self.values[op.src_a] >> offset) & mask,
                    (self.unknowns[op.src_a] >> offset) & mask,
                )
            }
            Opcode::Concat => {
                let low_width = op.src_c as u8;
                let low_mask = width_mask(low_width);
                let low_v = self.values[op.src_a] & low_mask;
                let low_u = self.unknowns[op.src_a] & low_mask;
                let (high_v, high_u) = if low_width >= 64 {
                    (0, 0)
                } else {
                    (
                        self.values[op.src_b] << low_width,
                        self.unknowns[op.src_b] << low_width,
                    )
                };
                ((low_v | high_v) & mask, (low_u | high_u) & mask)
            }
        };

        self.values[op.dst] = out_val;
        self.unknowns[op.dst] = out_unk;
    }
}

/// Evaluasi NOT bitwise 4-nilai.
#[inline]
const fn eval_not(a_val: u64, a_unk: u64, mask: u64) -> (u64, u64) {
    ((!a_val & !a_unk) & mask, a_unk & mask)
}

/// Evaluasi AND bitwise 4-nilai konservatif.
#[inline]
const fn eval_and(a_val: u64, a_unk: u64, b_val: u64, b_unk: u64, mask: u64) -> (u64, u64) {
    let known_zero = (!a_val & !a_unk) | (!b_val & !b_unk);
    let unk = (a_unk | b_unk) & !known_zero & mask;
    let val = (a_val & !a_unk) & (b_val & !b_unk) & mask;
    (val, unk)
}

/// Evaluasi OR bitwise 4-nilai konservatif.
#[inline]
const fn eval_or(a_val: u64, a_unk: u64, b_val: u64, b_unk: u64, mask: u64) -> (u64, u64) {
    let known_one = (a_val & !a_unk) | (b_val & !b_unk);
    let unk = (a_unk | b_unk) & !known_one & mask;
    let val = known_one & mask;
    (val, unk)
}

/// Evaluasi XOR bitwise 4-nilai konservatif.
#[inline]
const fn eval_xor(a_val: u64, a_unk: u64, b_val: u64, b_unk: u64, mask: u64) -> (u64, u64) {
    let unk = (a_unk | b_unk) & mask;
    let val = (a_val ^ b_val) & !unk & mask;
    (val, unk)
}

/// Evaluasi MUX 2-ke-1 dengan penanganan selektor X/Z konservatif (TBD-1).
#[inline]
const fn eval_mux(
    d0_val: u64,
    d0_unk: u64,
    d1_val: u64,
    d1_unk: u64,
    sel_val: u64,
    sel_unk: u64,
    mask: u64,
) -> (u64, u64) {
    if (sel_unk & 1) == 0 {
        if (sel_val & 1) == 0 {
            (d0_val & mask, d0_unk & mask)
        } else {
            (d1_val & mask, d1_unk & mask)
        }
    } else {
        let unk = (d0_unk | d1_unk | (d0_val ^ d1_val)) & mask;
        let val = (d0_val & d1_val) & !unk & mask;
        (val, unk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{SignalBinding, SourceMap};

    #[test]
    fn test_compiled_runtime_allocation_and_io() {
        let artifact = CompiledArtifact {
            operations: vec![CompiledOp::binary(Opcode::And, 2, 0, 1, 4)],
            level_ranges: std::iter::once(0..1).collect(),
            const_pool: vec![],
            slot_count: 3,
            slot_widths: vec![4, 4, 4],
            input_bindings: vec![
                SignalBinding::new(PinId::new(1), "A", 0, 4),
                SignalBinding::new(PinId::new(2), "B", 1, 4),
            ],
            output_bindings: vec![SignalBinding::new(PinId::new(3), "Y", 2, 4)],
            source_map: SourceMap::new(),
        };

        let mut rt = CompiledRuntime::from_artifact(artifact).unwrap();
        assert_eq!(rt.slot_count(), 3);

        // Uji masking saat input melebihi lebar 4-bit
        rt.set_input("A", LogicValue::from_u64(0xFF, 8)).unwrap();
        rt.set_input_by_pin(PinId::new(2), LogicValue::from_u64(0xAA, 8))
            .unwrap();
        rt.step_cycle();

        let out = rt.get_output("Y").unwrap();
        assert_eq!(out, LogicValue::from_u64(0x0A, 4));
        assert_eq!(rt.probe_slot(2).unwrap(), out);
    }

    #[test]
    fn test_compiled_runtime_all_logic_gates_and_masking() {
        // Menguji NOT, AND, OR, XOR, NAND, NOR, XNOR, BUF, CONST pada lebar 6-bit
        let artifact = CompiledArtifact {
            operations: vec![
                CompiledOp::unary(Opcode::Not, 2, 0, 6),      // slot 2 = !A
                CompiledOp::binary(Opcode::Nand, 3, 0, 1, 6), // slot 3 = !(A & B)
                CompiledOp::binary(Opcode::Nor, 4, 0, 1, 6),  // slot 4 = !(A | B)
                CompiledOp::binary(Opcode::Xnor, 5, 0, 1, 6), // slot 5 = !(A ^ B)
                CompiledOp::constant(6, 0, 6),                // slot 6 = 0b10_1010
                CompiledOp::unary(Opcode::Buf, 7, 6, 6),      // slot 7 = slot 6
            ],
            level_ranges: std::iter::once(0..6).collect(),
            const_pool: vec![(0b1111_101010, 0)], // bit di atas 6-bit harus terpotong mask
            slot_count: 8,
            slot_widths: vec![6; 8],
            input_bindings: vec![
                SignalBinding::new(PinId::new(1), "A", 0, 6),
                SignalBinding::new(PinId::new(2), "B", 1, 6),
            ],
            output_bindings: vec![
                SignalBinding::new(PinId::new(10), "NOT_A", 2, 6),
                SignalBinding::new(PinId::new(11), "NAND_AB", 3, 6),
                SignalBinding::new(PinId::new(12), "NOR_AB", 4, 6),
                SignalBinding::new(PinId::new(13), "XNOR_AB", 5, 6),
                SignalBinding::new(PinId::new(14), "CONST_BUF", 7, 6),
            ],
            source_map: SourceMap::new(),
        };

        let mut rt = CompiledRuntime::from_artifact(artifact).unwrap();
        rt.set_input("A", LogicValue::from_u64(0b110010, 6))
            .unwrap();
        rt.set_input("B", LogicValue::from_u64(0b101010, 6))
            .unwrap();
        rt.step_cycle();

        assert_eq!(rt.get_output("NOT_A").unwrap().val, 0b001101);
        assert_eq!(rt.get_output("NAND_AB").unwrap().val, 0b011101);
        assert_eq!(rt.get_output("NOR_AB").unwrap().val, 0b000101);
        assert_eq!(rt.get_output("XNOR_AB").unwrap().val, 0b100111);
        assert_eq!(rt.get_output("CONST_BUF").unwrap().val, 0b101010);
    }

    #[test]
    fn test_compiled_runtime_mux_slice_concat_and_unknowns() {
        let artifact = CompiledArtifact {
            operations: vec![
                CompiledOp::mux(3, 0, 1, 2, 4),
                CompiledOp::slice(4, 3, 2, 2),
                CompiledOp::concat(5, 4, 0, 2, 6),
            ],
            level_ranges: Vec::from([0..1, 1..2, 2..3]),
            const_pool: vec![],
            slot_count: 6,
            slot_widths: vec![4, 4, 1, 4, 2, 6],
            input_bindings: vec![
                SignalBinding::new(PinId::new(1), "D0", 0, 4),
                SignalBinding::new(PinId::new(2), "D1", 1, 4),
                SignalBinding::new(PinId::new(3), "SEL", 2, 1),
            ],
            output_bindings: vec![
                SignalBinding::new(PinId::new(10), "MUX_Y", 3, 4),
                SignalBinding::new(PinId::new(11), "SLICE_Y", 4, 2),
                SignalBinding::new(PinId::new(12), "CONCAT_Y", 5, 6),
            ],
            source_map: SourceMap::new(),
        };

        let mut rt = CompiledRuntime::from_artifact(artifact).unwrap();

        // Kasus 1: sel = 1 -> pilih D1 = 0b1101, slice[3:2] = 0b11, concat(D0=0b0101, 0b11) = 0b0101_11
        rt.set_input("D0", LogicValue::from_u64(0b0101, 4)).unwrap();
        rt.set_input("D1", LogicValue::from_u64(0b1101, 4)).unwrap();
        rt.set_input("SEL", LogicValue::from_u64(1, 1)).unwrap();
        rt.step_cycle();

        assert_eq!(rt.get_output("MUX_Y").unwrap().val, 0b1101);
        assert_eq!(rt.get_output("SLICE_Y").unwrap().val, 0b11);
        assert_eq!(rt.get_output("CONCAT_Y").unwrap().val, 0b01_0111);

        // Kasus 2: sel = X, D0 = 0b0101, D1 = 0b1101 -> bit yang sama (0,1,2) terdefinisi, bit 3 = X
        rt.set_input("SEL", LogicValue::all_x(1)).unwrap();
        rt.step_cycle();

        let mux_x = rt.get_output("MUX_Y").unwrap();
        assert_eq!(mux_x.val, 0b0101);
        assert_eq!(mux_x.unk, 0b1000);
    }
}
