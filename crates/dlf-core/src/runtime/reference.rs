//! # Reference Evaluator Kombinasional (B0 Semantic Oracle)
//!
//! Modul ini mengimplementasikan `ReferenceEvaluator` (Baseline B0 pada Tabel 3.17 proposal)
//! sebagai evaluator semantik independen tanpa IR datar maupun alokasi slot kontigu.
//! B0 mengevaluasi rangkaian bit-demi-bit menggunakan tabel kebenaran eksplisit `BitState`
//! sesuai Tabel 3.24 dan resolusi TBD-1/TBD-2 sebagai *golden oracle* untuk *differential testing*.

use super::value::LogicValue;
use crate::analysis::{EvaluationSchedule, GraphNode};
use crate::connectivity::{NetId, NetTerminal, Netlist};
use crate::error::CoreError;
use crate::model::circuit::CircuitDefinition;
use crate::model::component::{ComponentInstance, ParameterValue};
use crate::model::id::InstanceId;
use crate::model::pin::PinDirection;
use std::collections::BTreeMap;

/// Keadaan satu bit logika empat-nilai untuk evaluasi tabel kebenaran B0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BitState {
    Zero,
    One,
    X,
    Z,
}

impl BitState {
    /// Mengonversi bit ke-`i` dari `LogicValue` menjadi `BitState`.
    fn from_logic_value(lv: LogicValue, bit: u8) -> Self {
        let v = (lv.val >> bit) & 1;
        let u = (lv.unk >> bit) & 1;
        match (u, v) {
            (0, 0) => Self::Zero,
            (0, 1) => Self::One,
            (1, 0) => Self::X,
            (1, 1) => Self::Z,
            _ => unreachable!(),
        }
    }

    /// Menulis `BitState` ke posisi bit ke-`i` pada pasangan `(val, unk)`.
    fn write_to_parts(self, bit: u8, val: &mut u64, unk: &mut u64) {
        let bit_mask = 1u64 << bit;
        match self {
            Self::Zero => {
                *val &= !bit_mask;
                *unk &= !bit_mask;
            }
            Self::One => {
                *val |= bit_mask;
                *unk &= !bit_mask;
            }
            Self::X => {
                *val &= !bit_mask;
                *unk |= bit_mask;
            }
            Self::Z => {
                *val |= bit_mask;
                *unk |= bit_mask;
            }
        }
    }

    /// Normalisasi Z pada masukan gerbang biasa menjadi X sesuai Tabel 3.24.
    fn normalize_gate_input(self) -> Self {
        match self {
            Self::Z => Self::X,
            other => other,
        }
    }

    /// Tabel kebenaran NOT (Tabel 3.24).
    fn not(self) -> Self {
        match self.normalize_gate_input() {
            Self::Zero => Self::One,
            Self::One => Self::Zero,
            Self::X | Self::Z => Self::X,
        }
    }

    /// Tabel kebenaran AND konservatif (Tabel 3.24).
    fn and(self, rhs: Self) -> Self {
        match (self.normalize_gate_input(), rhs.normalize_gate_input()) {
            (Self::Zero, _) | (_, Self::Zero) => Self::Zero,
            (Self::One, Self::One) => Self::One,
            _ => Self::X,
        }
    }

    /// Tabel kebenaran OR konservatif (Tabel 3.24).
    fn or(self, rhs: Self) -> Self {
        match (self.normalize_gate_input(), rhs.normalize_gate_input()) {
            (Self::One, _) | (_, Self::One) => Self::One,
            (Self::Zero, Self::Zero) => Self::Zero,
            _ => Self::X,
        }
    }

    /// Tabel kebenaran XOR konservatif (Tabel 3.24).
    fn xor(self, rhs: Self) -> Self {
        match (self.normalize_gate_input(), rhs.normalize_gate_input()) {
            (Self::Zero, Self::Zero) | (Self::One, Self::One) => Self::Zero,
            (Self::Zero, Self::One) | (Self::One, Self::Zero) => Self::One,
            _ => Self::X,
        }
    }

    /// Tabel kebenaran MUX konservatif (TBD-1).
    fn mux(d0: Self, d1: Self, sel: Self) -> Self {
        match sel.normalize_gate_input() {
            Self::Zero => d0,
            Self::One => d1,
            Self::X | Self::Z => match (d0, d1) {
                (Self::Zero, Self::Zero) => Self::Zero,
                (Self::One, Self::One) => Self::One,
                _ => Self::X,
            },
        }
    }
}

/// Evaluator referensi semantik independen (Baseline B0).
#[derive(Debug, Clone)]
pub struct ReferenceEvaluator {
    circuit: CircuitDefinition,
    netlist: Netlist,
    schedule: EvaluationSchedule,
    net_values: BTreeMap<NetId, LogicValue>,
    comp_map: BTreeMap<InstanceId, ComponentInstance>,
    input_port_nets: BTreeMap<String, (NetId, u8)>,
    output_port_nets: BTreeMap<String, (NetId, u8)>,
}

impl ReferenceEvaluator {
    /// Membangun `ReferenceEvaluator` (B0) dari `CircuitDefinition`, `Netlist`, dan `EvaluationSchedule`.
    pub fn new(circuit: CircuitDefinition, netlist: Netlist, schedule: EvaluationSchedule) -> Self {
        let mut net_values = BTreeMap::new();
        for (&net_id, net) in &netlist.nets {
            net_values.insert(net_id, LogicValue::all_x(net.width.get()));
        }

        let comp_map = circuit
            .components
            .iter()
            .map(|c| (c.id, c.clone()))
            .collect();

        let mut input_port_nets = BTreeMap::new();
        let mut output_port_nets = BTreeMap::new();
        for port in &circuit.ports {
            let terminal = NetTerminal::Port { id: port.id };
            if let Some(&net_id) = netlist.terminal_to_net.get(&terminal) {
                let entry = (net_id, port.width.get());
                match port.direction {
                    PinDirection::Input => {
                        input_port_nets.insert(port.name.clone(), entry);
                    }
                    PinDirection::Output => {
                        output_port_nets.insert(port.name.clone(), entry);
                    }
                    PinDirection::Inout => {
                        input_port_nets.insert(port.name.clone(), entry);
                        output_port_nets.insert(port.name.clone(), entry);
                    }
                }
            }
        }

        Self {
            circuit,
            netlist,
            schedule,
            net_values,
            comp_map,
            input_port_nets,
            output_port_nets,
        }
    }

    /// Menetapkan nilai sinyal masukan pada port bernama `name`.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika nama port masukan tidak ditemukan.
    pub fn set_input(&mut self, name: &str, value: LogicValue) -> Result<(), CoreError> {
        let &(net_id, width) = self.input_port_nets.get(name).ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Port masukan '{}' tidak ditemukan pada rangkaian '{}'",
                name, self.circuit.name
            ))
        })?;

        let masked = LogicValue::from_parts(value.val, value.unk, width);
        self.net_values.insert(net_id, masked);
        Ok(())
    }

    /// Membaca nilai sinyal keluaran pada port bernama `name`.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika nama port keluaran tidak ditemukan.
    pub fn get_output(&self, name: &str) -> Result<LogicValue, CoreError> {
        let &(net_id, width) = self.output_port_nets.get(name).ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Port keluaran '{}' tidak ditemukan pada rangkaian '{}'",
                name, self.circuit.name
            ))
        })?;

        let lv = self.net_values.get(&net_id).copied().unwrap_or_default();
        Ok(LogicValue::from_parts(lv.val, lv.unk, width))
    }

    /// Mengevaluasi seluruh komponen kombinasional secara topologis satu siklus penuh.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::RuntimeError` jika ditemukan komponen atau pin yang tidak valid.
    pub fn step_cycle(&mut self) -> Result<(), CoreError> {
        for idx in 0..self.schedule.ordered_nodes.len() {
            let node = self.schedule.ordered_nodes[idx];
            if let GraphNode::Combinational(inst_id) = node {
                let comp = self
                    .comp_map
                    .get(&inst_id)
                    .ok_or_else(|| {
                        CoreError::RuntimeError(format!(
                            "Komponen {} tidak ditemukan di ReferenceEvaluator",
                            inst_id
                        ))
                    })?
                    .clone();

                self.eval_component(comp)?;
            }
        }
        Ok(())
    }

    /// Membaca `NetId` dan `LogicValue` untuk pin komponen tertentu.
    fn read_pin(
        &self,
        comp: &ComponentInstance,
        pin_decl_id: &str,
    ) -> Result<LogicValue, CoreError> {
        let pin = comp.get_pin_by_declaration(pin_decl_id).ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Pin '{}' tidak ditemukan pada komponen {}",
                pin_decl_id, comp.id
            ))
        })?;
        let terminal = NetTerminal::Pin {
            instance_id: comp.id,
            pin_id: pin.id,
        };
        let net_id = self.netlist.terminal_to_net.get(&terminal).ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Pin '{}' pada komponen {} tidak terhubung ke net",
                pin_decl_id, comp.id
            ))
        })?;
        Ok(self.net_values.get(net_id).copied().unwrap_or_default())
    }

    /// Menulis `LogicValue` ke net yang terhubung dengan pin keluaran komponen.
    fn write_pin(
        &mut self,
        comp: &ComponentInstance,
        pin_decl_id: &str,
        value: LogicValue,
    ) -> Result<(), CoreError> {
        let pin = comp.get_pin_by_declaration(pin_decl_id).ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Pin '{}' tidak ditemukan pada komponen {}",
                pin_decl_id, comp.id
            ))
        })?;
        let terminal = NetTerminal::Pin {
            instance_id: comp.id,
            pin_id: pin.id,
        };
        let net_id = *self.netlist.terminal_to_net.get(&terminal).ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Pin '{}' pada komponen {} tidak terhubung ke net",
                pin_decl_id, comp.id
            ))
        })?;
        let masked = LogicValue::from_parts(value.val, value.unk, pin.width.get());
        self.net_values.insert(net_id, masked);
        Ok(())
    }

    /// Mengevaluasi satu komponen kombinasional secara eksplisit bit-demi-bit.
    fn eval_component(&mut self, comp: ComponentInstance) -> Result<(), CoreError> {
        let def_id = comp.definition_id.as_str();
        let out_pin = comp.get_pin_by_declaration("y").ok_or_else(|| {
            CoreError::RuntimeError(format!(
                "Komponen {} tidak memiliki pin keluaran 'y'",
                comp.id
            ))
        })?;
        let width = out_pin.width.get();

        let result = match def_id {
            "dlf.builtin.buf" => {
                let a = self.read_pin(&comp, "a")?;
                LogicValue::from_parts(a.val, a.unk, width)
            }
            "dlf.builtin.not" => {
                let a = self.read_pin(&comp, "a")?;
                eval_bitwise_unary(a, width, BitState::not)
            }
            "dlf.builtin.and" => {
                let a = self.read_pin(&comp, "a")?;
                let b = self.read_pin(&comp, "b")?;
                eval_bitwise_binary(a, b, width, BitState::and)
            }
            "dlf.builtin.or" => {
                let a = self.read_pin(&comp, "a")?;
                let b = self.read_pin(&comp, "b")?;
                eval_bitwise_binary(a, b, width, BitState::or)
            }
            "dlf.builtin.xor" => {
                let a = self.read_pin(&comp, "a")?;
                let b = self.read_pin(&comp, "b")?;
                eval_bitwise_binary(a, b, width, BitState::xor)
            }
            "dlf.builtin.nand" => {
                let a = self.read_pin(&comp, "a")?;
                let b = self.read_pin(&comp, "b")?;
                eval_bitwise_binary(a, b, width, |x, y| x.and(y).not())
            }
            "dlf.builtin.nor" => {
                let a = self.read_pin(&comp, "a")?;
                let b = self.read_pin(&comp, "b")?;
                eval_bitwise_binary(a, b, width, |x, y| x.or(y).not())
            }
            "dlf.builtin.xnor" => {
                let a = self.read_pin(&comp, "a")?;
                let b = self.read_pin(&comp, "b")?;
                eval_bitwise_binary(a, b, width, |x, y| x.xor(y).not())
            }
            "dlf.builtin.mux" => {
                let d0 = self.read_pin(&comp, "d0")?;
                let d1 = self.read_pin(&comp, "d1")?;
                let sel = self.read_pin(&comp, "sel")?;
                let sel_bit = BitState::from_logic_value(sel, 0);

                let mut val = 0u64;
                let mut unk = 0u64;
                for bit in 0..width {
                    let b0 = BitState::from_logic_value(d0, bit);
                    let b1 = BitState::from_logic_value(d1, bit);
                    BitState::mux(b0, b1, sel_bit).write_to_parts(bit, &mut val, &mut unk);
                }
                LogicValue::from_parts(val, unk, width)
            }
            "dlf.builtin.const" => {
                let val = match comp.parameters.get("value") {
                    Some(ParameterValue::Integer(v)) => *v as u64,
                    _ => 0,
                };
                let unk = match comp.parameters.get("unknown") {
                    Some(ParameterValue::Integer(v)) => *v as u64,
                    _ => 0,
                };
                LogicValue::from_parts(val, unk, width)
            }
            "dlf.builtin.slice" => {
                let a = self.read_pin(&comp, "a")?;
                let offset = match comp.parameters.get("offset") {
                    Some(ParameterValue::Integer(v)) => (*v).clamp(0, 63) as u8,
                    _ => 0,
                };
                let mut val = 0u64;
                let mut unk = 0u64;
                for bit in 0..width {
                    let src_bit = offset.saturating_add(bit);
                    let state = if src_bit < 64 {
                        BitState::from_logic_value(a, src_bit)
                    } else {
                        BitState::Zero
                    };
                    state.write_to_parts(bit, &mut val, &mut unk);
                }
                LogicValue::from_parts(val, unk, width)
            }
            "dlf.builtin.concat" => {
                let low = self.read_pin(&comp, "low")?;
                let high = self.read_pin(&comp, "high")?;
                let low_width = comp
                    .get_pin_by_declaration("low")
                    .map(|p| p.width.get())
                    .unwrap_or(1);

                let mut val = 0u64;
                let mut unk = 0u64;
                for bit in 0..width {
                    let state = if bit < low_width {
                        BitState::from_logic_value(low, bit)
                    } else {
                        let high_bit = bit - low_width;
                        if high_bit < 64 {
                            BitState::from_logic_value(high, high_bit)
                        } else {
                            BitState::Zero
                        }
                    };
                    state.write_to_parts(bit, &mut val, &mut unk);
                }
                LogicValue::from_parts(val, unk, width)
            }
            other => {
                return Err(CoreError::RuntimeError(format!(
                    "Komponen '{}' belum didukung pada ReferenceEvaluator B0",
                    other
                )));
            }
        };

        self.write_pin(&comp, "y", result)
    }
}

/// Evaluasi bit-demi-bit untuk operasi uner.
fn eval_bitwise_unary<F>(a: LogicValue, width: u8, op: F) -> LogicValue
where
    F: Fn(BitState) -> BitState,
{
    let mut val = 0u64;
    let mut unk = 0u64;
    for bit in 0..width {
        let out = op(BitState::from_logic_value(a, bit));
        out.write_to_parts(bit, &mut val, &mut unk);
    }
    LogicValue::from_parts(val, unk, width)
}

/// Evaluasi bit-demi-bit untuk operasi biner.
fn eval_bitwise_binary<F>(a: LogicValue, b: LogicValue, width: u8, op: F) -> LogicValue
where
    F: Fn(BitState, BitState) -> BitState,
{
    let mut val = 0u64;
    let mut unk = 0u64;
    for bit in 0..width {
        let out = op(
            BitState::from_logic_value(a, bit),
            BitState::from_logic_value(b, bit),
        );
        out.write_to_parts(bit, &mut val, &mut unk);
    }
    LogicValue::from_parts(val, unk, width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::CircuitCompiler;
    use crate::model::circuit::PortDefinition;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, JunctionId, PinId, WireId};
    use crate::model::pin::SignalWidth;
    use crate::model::wiring::{Endpoint, Junction, WireSegment};

    #[test]
    fn test_reference_evaluator_half_adder_truth_table() {
        let registry = ComponentRegistry::with_builtins();
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "HalfAdder");

        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "A",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(2),
            "B",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(0.0, 40.0),
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(3),
            "SUM",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::new(200.0, 0.0),
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(4),
            "CARRY",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::new(200.0, 40.0),
        ));

        let xor_inst = registry
            .get("dlf.builtin.xor")
            .unwrap()
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12), PinId::new(13)],
                Point::new(100.0, 0.0),
            )
            .unwrap();
        let and_inst = registry
            .get("dlf.builtin.and")
            .unwrap()
            .instantiate(
                InstanceId::new(20),
                &[PinId::new(21), PinId::new(22), PinId::new(23)],
                Point::new(100.0, 40.0),
            )
            .unwrap();
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

        let compiled = CircuitCompiler::compile(&circuit, &registry).unwrap();
        let mut b0 = ReferenceEvaluator::new(
            compiled.snapshot.circuit,
            compiled.netlist,
            compiled.schedule,
        );

        let truth_table = [(0, 0, 0, 0), (0, 1, 1, 0), (1, 0, 1, 0), (1, 1, 0, 1)];
        for (a, b, exp_sum, exp_carry) in truth_table {
            b0.set_input("A", LogicValue::from_u64(a, 1)).unwrap();
            b0.set_input("B", LogicValue::from_u64(b, 1)).unwrap();
            b0.step_cycle().unwrap();

            assert_eq!(b0.get_output("SUM").unwrap().val, exp_sum);
            assert_eq!(b0.get_output("CARRY").unwrap().val, exp_carry);
        }

        // Uji nilai X konservatif: 0 AND X = 0, 0 XOR X = X
        b0.set_input("A", LogicValue::from_u64(0, 1)).unwrap();
        b0.set_input("B", LogicValue::all_x(1)).unwrap();
        b0.step_cycle().unwrap();
        assert_eq!(b0.get_output("SUM").unwrap(), LogicValue::all_x(1));
        assert_eq!(b0.get_output("CARRY").unwrap(), LogicValue::from_u64(0, 1));
    }
}
