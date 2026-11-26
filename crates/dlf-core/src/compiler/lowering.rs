//! # IR Lowering Engine
//!
//! Modul ini menerjemahkan `CircuitDefinition`, `Netlist` kanonis, dan `EvaluationSchedule`
//! hasil levelisasi M2 menjadi `CompiledArtifact` berisi daftar operasi `CompiledOp`
//! yang tersusun rapi per tingkatan level ketergantungan (`LevelRange`).

use super::allocator::SignalSlotAllocator;
use crate::analysis::{EvaluationSchedule, GraphNode};
use crate::connectivity::{NetTerminal, Netlist};
use crate::error::CoreError;
use crate::ir::{CompiledArtifact, CompiledOp, Opcode};
use crate::model::circuit::CircuitDefinition;
use crate::model::component::{ComponentInstance, ParameterValue};
use crate::model::id::InstanceId;
use std::collections::BTreeMap;
use std::ops::Range;

/// Mengambil indeks slot runtime dan lebar bit dari suatu pin komponen pada `Netlist`.
fn resolve_pin_slot(
    comp: &ComponentInstance,
    pin_decl_id: &str,
    netlist: &Netlist,
    allocator: &SignalSlotAllocator,
) -> Result<(usize, u8), CoreError> {
    let pin = comp.get_pin_by_declaration(pin_decl_id).ok_or_else(|| {
        CoreError::ValidationError(format!(
            "Komponen {} ('{}') tidak memiliki pin '{}'",
            comp.id, comp.definition_id, pin_decl_id
        ))
    })?;

    let terminal = NetTerminal::Pin {
        instance_id: comp.id,
        pin_id: pin.id,
    };

    let slot = allocator
        .slot_of_terminal(netlist, &terminal)
        .ok_or_else(|| {
            CoreError::ValidationError(format!(
                "Pin '{}' pada komponen {} tidak terpetakan ke slot netlist manapun",
                pin_decl_id, comp.id
            ))
        })?;

    Ok((slot, pin.width.get()))
}

/// Mengambil parameter integer dari komponen dengan nilai bawaan jika tidak disetel.
fn get_int_param(comp: &ComponentInstance, name: &str, default: i64) -> i64 {
    match comp.parameters.get(name) {
        Some(ParameterValue::Integer(v)) => *v,
        _ => default,
    }
}

/// Menerjemahkan satu `ComponentInstance` kombinasional menjadi satu atau lebih `CompiledOp`.
fn lower_component(
    comp: &ComponentInstance,
    netlist: &Netlist,
    allocator: &mut SignalSlotAllocator,
) -> Result<CompiledOp, CoreError> {
    let def_id = comp.definition_id.as_str();

    match def_id {
        "dlf.builtin.buf" => {
            let (src_a, _) = resolve_pin_slot(comp, "a", netlist, allocator)?;
            let (dst, width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            Ok(CompiledOp::unary(Opcode::Buf, dst, src_a, width))
        }
        "dlf.builtin.not" => {
            let (src_a, _) = resolve_pin_slot(comp, "a", netlist, allocator)?;
            let (dst, width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            Ok(CompiledOp::unary(Opcode::Not, dst, src_a, width))
        }
        "dlf.builtin.and" | "dlf.builtin.or" | "dlf.builtin.xor" | "dlf.builtin.nand"
        | "dlf.builtin.nor" | "dlf.builtin.xnor" => {
            let opcode = match def_id {
                "dlf.builtin.and" => Opcode::And,
                "dlf.builtin.or" => Opcode::Or,
                "dlf.builtin.xor" => Opcode::Xor,
                "dlf.builtin.nand" => Opcode::Nand,
                "dlf.builtin.nor" => Opcode::Nor,
                "dlf.builtin.xnor" => Opcode::Xnor,
                _ => unreachable!(),
            };
            let (src_a, _) = resolve_pin_slot(comp, "a", netlist, allocator)?;
            let (src_b, _) = resolve_pin_slot(comp, "b", netlist, allocator)?;
            let (dst, width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            Ok(CompiledOp::binary(opcode, dst, src_a, src_b, width))
        }
        "dlf.builtin.mux" => {
            let (d0, _) = resolve_pin_slot(comp, "d0", netlist, allocator)?;
            let (d1, _) = resolve_pin_slot(comp, "d1", netlist, allocator)?;
            let (sel, _) = resolve_pin_slot(comp, "sel", netlist, allocator)?;
            let (dst, width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            Ok(CompiledOp::mux(dst, d0, d1, sel, width))
        }
        "dlf.builtin.const" => {
            let (dst, width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            let raw_val = get_int_param(comp, "value", 0) as u64;
            let raw_unk = get_int_param(comp, "unknown", 0) as u64;
            let const_idx = allocator.intern_const(raw_val, raw_unk);
            Ok(CompiledOp::constant(dst, const_idx, width))
        }
        "dlf.builtin.slice" => {
            let (src_a, _) = resolve_pin_slot(comp, "a", netlist, allocator)?;
            let (dst, width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            let offset = get_int_param(comp, "offset", 0);
            if !(0..64).contains(&offset) {
                return Err(CoreError::ValidationError(format!(
                    "Komponen slice {} memiliki offset {} di luar rentang 0..64",
                    comp.id, offset
                )));
            }
            Ok(CompiledOp::slice(dst, src_a, offset as usize, width))
        }
        "dlf.builtin.concat" => {
            let (low, low_width) = resolve_pin_slot(comp, "low", netlist, allocator)?;
            let (high, _) = resolve_pin_slot(comp, "high", netlist, allocator)?;
            let (dst, total_width) = resolve_pin_slot(comp, "y", netlist, allocator)?;
            Ok(CompiledOp::concat(
                dst,
                low,
                high,
                low_width as usize,
                total_width,
            ))
        }
        _ => Err(CoreError::ValidationError(format!(
            "Komponen '{}' pada instance {} belum didukung oleh IR lowering M3",
            def_id, comp.id
        ))),
    }
}

/// Mengonversi `CircuitDefinition`, `Netlist`, dan `EvaluationSchedule` menjadi
/// `CompiledArtifact` siap eksekusi.
///
/// # Errors
/// Mengembalikan `CoreError::ValidationError` apabila terdapat pin yang belum terpetakan,
/// komponen yang belum didukung, atau pelanggaran invarian IR pasca-kompilasi.
pub fn lower_to_ir(
    circuit: &CircuitDefinition,
    netlist: &Netlist,
    schedule: &EvaluationSchedule,
) -> Result<CompiledArtifact, CoreError> {
    let mut allocator = SignalSlotAllocator::allocate_from_netlist(circuit, netlist);

    let comp_map: BTreeMap<InstanceId, &ComponentInstance> =
        circuit.components.iter().map(|c| (c.id, c)).collect();

    let mut operations = Vec::new();
    let mut level_ranges: Vec<Range<usize>> = Vec::with_capacity(schedule.level_count());

    for level_nodes in &schedule.levels {
        let range_start = operations.len();

        for &node in level_nodes {
            if let GraphNode::Combinational(inst_id) = node {
                let comp = comp_map.get(&inst_id).ok_or_else(|| {
                    CoreError::ValidationError(format!(
                        "Instance komponen {} pada EvaluationSchedule tidak ditemukan di sirkuit",
                        inst_id
                    ))
                })?;

                let op = lower_component(comp, netlist, &mut allocator)?;
                let op_idx = operations.len();
                operations.push(op);
                allocator.source_map_mut().insert_op(op_idx, comp.id);
            }
        }

        let range_end = operations.len();
        level_ranges.push(range_start..range_end);
    }

    let layout = allocator.into_layout();
    let artifact = CompiledArtifact {
        operations,
        level_ranges,
        const_pool: layout.const_pool,
        slot_count: layout.slot_count,
        slot_widths: layout.slot_widths,
        input_bindings: layout.input_bindings,
        output_bindings: layout.output_bindings,
        source_map: layout.source_map,
    };

    artifact.validate()?;
    Ok(artifact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{DependencyGraph, levelize_dependency_graph};
    use crate::model::circuit::PortDefinition;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, JunctionId, PinId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, Junction, WireSegment};

    #[test]
    fn test_lower_half_adder_to_ir() {
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

        // A -> J30 -> XOR.a, AND.a
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

        // B -> J31 -> XOR.b, AND.b
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

        // Outputs
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

        let netlist = Netlist::from_circuit(&circuit);
        let graph = DependencyGraph::from_circuit_and_netlist(&circuit, &netlist, &registry);
        let schedule = levelize_dependency_graph(&graph).unwrap();

        let artifact = lower_to_ir(&circuit, &netlist, &schedule).expect("Lowering harus berhasil");

        assert_eq!(artifact.operations.len(), 2);
        assert_eq!(artifact.slot_count, 4);
        assert_eq!(artifact.input_bindings.len(), 2);
        assert_eq!(artifact.output_bindings.len(), 2);
        assert_eq!(artifact.source_map.op_to_instance.len(), 2);
    }
}
