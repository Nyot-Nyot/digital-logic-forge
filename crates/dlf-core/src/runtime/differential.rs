//! # Differential Testing & Fuzzing Harness (B1 vs B0 Bit-Exact Verification)
//!
//! Modul ini mengimplementasikan `DifferentialHarness` untuk membandingkan keluaran
//! `CompiledRuntime` (B1) terhadap `ReferenceEvaluator` (B0) secara *bit-exact*
//! menggunakan kombinasi uji tabel kebenaran penuh maupun *seeded PRNG fuzzing*.

use super::compiled::CompiledRuntime;
use super::reference::ReferenceEvaluator;
use super::value::LogicValue;
use crate::compiler::CircuitCompiler;
use crate::error::CoreError;
use crate::ir::width_mask;
use crate::model::circuit::CircuitDefinition;
use crate::model::component::ComponentRegistry;

/// Catatan ketidakcocokan apabila keluaran B1 berbeda dengan B0 pada suatu vektor uji.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialMismatch {
    /// Nomor urut vektor masukan saat ketidaksesuaian terjadi.
    pub vector_index: usize,
    /// Vektor masukan yang memicu ketidaksesuaian.
    pub inputs: Vec<(String, LogicValue)>,
    /// Nama port keluaran yang berbeda.
    pub output_name: String,
    /// Nilai keluaran harapan dari *oracle* B0.
    pub b0_expected: LogicValue,
    /// Nilai keluaran aktual dari *runtime* B1.
    pub b1_actual: LogicValue,
}

/// Laporan hasil eksekusi *differential testing* antara B0 dan B1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialReport {
    /// Jumlah total vektor masukan yang telah diuji.
    pub vectors_tested: usize,
    /// Daftar ketidaksesuaian yang ditemukan (kosong berarti lulus 100%).
    pub mismatches: Vec<DifferentialMismatch>,
}

impl DifferentialReport {
    /// Mengembalikan `true` jika seluruh vektor menghasilkan keluaran *bit-exact* identik.
    #[inline]
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty()
    }
}

/// Generator angka acak deterministik SplitMix64 agar *fuzzing* dapat direproduksi ulang.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Penyelaras pengujian diferensial antara *Reference Evaluator* (B0) dan *Compiled Runtime* (B1).
pub struct DifferentialHarness {
    b0: ReferenceEvaluator,
    b1: CompiledRuntime,
    inputs: Vec<(String, u8)>,
    outputs: Vec<(String, u8)>,
}

impl DifferentialHarness {
    /// Mengompilasi `circuit` dan menginisialisasi pasangan evaluator B0 & B1.
    ///
    /// # Errors
    /// Mengembalikan `CoreError` jika kompilasi atau inisialisasi runtime gagal.
    pub fn new(
        circuit: &CircuitDefinition,
        registry: &ComponentRegistry,
    ) -> Result<Self, CoreError> {
        let compiled = CircuitCompiler::compile(circuit, registry)?;
        let inputs = compiled
            .artifact
            .input_bindings
            .iter()
            .map(|b| (b.name.clone(), b.width))
            .collect();
        let outputs = compiled
            .artifact
            .output_bindings
            .iter()
            .map(|b| (b.name.clone(), b.width))
            .collect();

        let b0 = ReferenceEvaluator::new(
            compiled.snapshot.circuit,
            compiled.netlist,
            compiled.schedule,
        );
        let b1 = CompiledRuntime::from_artifact(compiled.artifact)?;

        Ok(Self {
            b0,
            b1,
            inputs,
            outputs,
        })
    }

    /// Menjalankan satu vektor masukan pada B0 dan B1 serta membandingkan seluruh keluaran.
    ///
    /// # Errors
    /// Mengembalikan `CoreError` apabila terjadi kesalahan saat penetapan input atau eksekusi siklus.
    pub fn check_vector(
        &mut self,
        vector_index: usize,
        input_values: &[(String, LogicValue)],
    ) -> Result<Option<DifferentialMismatch>, CoreError> {
        for (name, val) in input_values {
            self.b0.set_input(name, *val)?;
            self.b1.set_input(name, *val)?;
        }

        self.b0.step_cycle()?;
        self.b1.step_cycle();

        for (out_name, _width) in &self.outputs {
            let expected = self.b0.get_output(out_name)?;
            let actual = self.b1.get_output(out_name)?;
            if expected != actual {
                return Ok(Some(DifferentialMismatch {
                    vector_index,
                    inputs: input_values.to_vec(),
                    output_name: out_name.clone(),
                    b0_expected: expected,
                    b1_actual: actual,
                }));
            }
        }

        Ok(None)
    }

    /// Menjalankan *fuzzing* acak deterministik sebanyak `iterations` siklus dengan `seed` tertentu.
    ///
    /// Jika `include_unknowns` bernilai `true`, harness juga menyuntikkan bit `X` dan `Z`
    /// untuk memverifikasi konsistensi logika empat-nilai antara B0 dan B1.
    ///
    /// # Errors
    /// Mengembalikan `CoreError` apabila eksekusi siklus gagal.
    pub fn fuzz_random(
        &mut self,
        iterations: usize,
        seed: u64,
        include_unknowns: bool,
    ) -> Result<DifferentialReport, CoreError> {
        let mut rng = SplitMix64::new(seed);
        let mut mismatches = Vec::new();

        for iter_idx in 0..iterations {
            let mut vec_inputs = Vec::with_capacity(self.inputs.len());
            for (name, width) in &self.inputs {
                let raw_val = rng.next_u64();
                let raw_unk = if include_unknowns && (rng.next_u64() & 0b11) == 0 {
                    // Suntikkan mask X/Z secara sporadis pada ~25% vektor
                    rng.next_u64() & rng.next_u64()
                } else {
                    0
                };
                vec_inputs.push((
                    name.clone(),
                    LogicValue::from_parts(raw_val, raw_unk, *width),
                ));
            }

            if let Some(mismatch) = self.check_vector(iter_idx, &vec_inputs)? {
                mismatches.push(mismatch);
                break;
            }
        }

        Ok(DifferentialReport {
            vectors_tested: iterations,
            mismatches,
        })
    }

    /// Menjalankan verifikasi tabel kebenaran 2-nilai lengkap (*exhaustive*) untuk rangkaian kecil.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::ValidationError` apabila total bit masukan melebihi 12 bit
    /// (4.096 kombinasi) untuk mencegah ledakan kombinatorial saat unit test.
    pub fn check_exhaustive_2val(&mut self) -> Result<DifferentialReport, CoreError> {
        let total_bits: u32 = self.inputs.iter().map(|(_, w)| u32::from(*w)).sum();
        if total_bits > 12 {
            return Err(CoreError::ValidationError(format!(
                "Total lebar masukan ({} bit) melebihi batas uji exhaustive (12 bit)",
                total_bits
            )));
        }

        let total_vectors = 1usize << total_bits;
        let mut mismatches = Vec::new();

        for vec_idx in 0..total_vectors {
            let mut shift = 0u32;
            let mut vec_inputs = Vec::with_capacity(self.inputs.len());
            for (name, width) in &self.inputs {
                let mask = width_mask(*width);
                let val = ((vec_idx as u64) >> shift) & mask;
                vec_inputs.push((name.clone(), LogicValue::from_u64(val, *width)));
                shift += u32::from(*width);
            }

            if let Some(mismatch) = self.check_vector(vec_idx, &vec_inputs)? {
                mismatches.push(mismatch);
                break;
            }
        }

        Ok(DifferentialReport {
            vectors_tested: total_vectors,
            mismatches,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::PortDefinition;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, InstanceId, JunctionId, PinId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, Junction, WireSegment};

    #[test]
    fn test_differential_half_adder_exhaustive_and_fuzz() {
        let registry = ComponentRegistry::with_builtins();
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "HalfAdder");

        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "A",
            PinDirection::Input,
            SignalWidth::new(4).unwrap(),
            Point::ZERO,
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(2),
            "B",
            PinDirection::Input,
            SignalWidth::new(4).unwrap(),
            Point::new(0.0, 40.0),
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(3),
            "SUM",
            PinDirection::Output,
            SignalWidth::new(4).unwrap(),
            Point::new(200.0, 0.0),
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(4),
            "CARRY",
            PinDirection::Output,
            SignalWidth::new(4).unwrap(),
            Point::new(200.0, 40.0),
        ));

        let mut xor_inst = registry
            .get("dlf.builtin.xor")
            .unwrap()
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12), PinId::new(13)],
                Point::new(100.0, 0.0),
            )
            .unwrap();
        for pin in &mut xor_inst.pins {
            pin.width = SignalWidth::new(4).unwrap();
        }
        let mut and_inst = registry
            .get("dlf.builtin.and")
            .unwrap()
            .instantiate(
                InstanceId::new(20),
                &[PinId::new(21), PinId::new(22), PinId::new(23)],
                Point::new(100.0, 40.0),
            )
            .unwrap();
        for pin in &mut and_inst.pins {
            pin.width = SignalWidth::new(4).unwrap();
        }
        circuit.add_component(xor_inst);
        circuit.add_component(and_inst);

        circuit.add_junction(Junction::new(JunctionId::new(30), Point::new(50.0, 0.0)));
        circuit.add_junction(Junction::new(JunctionId::new(31), Point::new(50.0, 40.0)));

        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(101),
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(102),
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(21),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(103),
            Endpoint::Port { id: PinId::new(2) },
            Endpoint::Junction {
                id: JunctionId::new(31),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(104),
            Endpoint::Junction {
                id: JunctionId::new(31),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(12),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(105),
            Endpoint::Junction {
                id: JunctionId::new(31),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(22),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(106),
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(13),
            },
            Endpoint::Port { id: PinId::new(3) },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(107),
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(23),
            },
            Endpoint::Port { id: PinId::new(4) },
        ));

        let mut harness = DifferentialHarness::new(&circuit, &registry).unwrap();

        let ex_report = harness.check_exhaustive_2val().unwrap();
        assert!(ex_report.is_clean());
        assert_eq!(ex_report.vectors_tested, 256);

        let fuzz_report = harness.fuzz_random(500, 0xDEAD_BEEF, true).unwrap();
        assert!(fuzz_report.is_clean());
        assert_eq!(fuzz_report.vectors_tested, 500);
    }
}
