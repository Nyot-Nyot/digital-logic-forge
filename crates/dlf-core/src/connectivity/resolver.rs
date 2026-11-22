//! # Resolusi Percabangan Junction dan Penggabungan Segmen Kawat
//!
//! Modul ini mengimplementasikan aturan konektivitas visual Bab 3.4.2 proposal:
//! - Endpoint kawat yang terhubung ke pin/port/junction digabungkan ke partisi yang sama.
//! - Titik sambung Junction dengan dua cabang atau lebih menyatukan seluruh segmen kawat yang bertemu.
//! - Menggunakan `DisjointSet<ConnectionNode>` untuk menghasilkan representasi kanonis.

use super::dsu::DisjointSet;
use super::node::{ConnectionNode, extract_circuit_nodes, extract_wire_connections};
use super::tunnel::resolve_circuit_tunnels;
use crate::model::circuit::CircuitDefinition;
use crate::validation::ValidationReport;

/// Membangun partisi konektivitas topologis lembar sirkuit menggunakan algoritma DSU.
///
/// Seluruh node (Pin, Port, Junction, Tunnel) didaftarkan dan digabungkan berdasarkan
/// segmen-segmen kawat yang menghubungkannya serta pasangan tunnel bernama identik.
pub fn resolve_circuit_connectivity(circuit: &CircuitDefinition) -> DisjointSet<ConnectionNode> {
    let mut report = ValidationReport::new();
    resolve_circuit_connectivity_with_report(circuit, &mut report)
}

/// Membangun partisi konektivitas topologis lembar sirkuit dan mengumpulkan diagnostik konektivitas.
pub fn resolve_circuit_connectivity_with_report(
    circuit: &CircuitDefinition,
    report: &mut ValidationReport,
) -> DisjointSet<ConnectionNode> {
    let mut dsu = DisjointSet::new();

    // 1. Daftarkan seluruh simpul yang ada di sirkuit
    for node in extract_circuit_nodes(circuit) {
        dsu.insert(node);
    }

    // 2. Gabungkan simpul-simpul yang dihubungkan oleh segmen kawat
    for (node_a, node_b) in extract_wire_connections(circuit) {
        dsu.union(node_a, node_b);
    }

    // 3. Resolusi pasangan tunnel nirkabel
    resolve_circuit_tunnels(circuit, &mut dsu, report);

    dsu
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::PortDefinition;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, InstanceId, JunctionId, PinId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, Junction, WireSegment};

    #[test]
    fn test_junction_branching_fanout() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Fanout Test");
        let registry = ComponentRegistry::with_builtins();

        // 1 Port Input
        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "clk_in",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));

        // 2 Gerbang Buffer (Gate1: in=10, out=11; Gate2: in=20, out=21)
        let buf_def = registry.get("dlf.builtin.buf").unwrap();
        let g1 = buf_def
            .instantiate(
                InstanceId::new(100),
                &[PinId::new(10), PinId::new(11)],
                Point::ZERO,
            )
            .unwrap();
        let g2 = buf_def
            .instantiate(
                InstanceId::new(200),
                &[PinId::new(20), PinId::new(21)],
                Point::ZERO,
            )
            .unwrap();
        circuit.add_component(g1);
        circuit.add_component(g2);

        // 1 Junction
        circuit.add_junction(Junction::new(JunctionId::new(50), Point::ZERO));

        // Wire 1: Port(1) -> Junction(50)
        circuit.add_wire(WireSegment::new(
            WireId::new(1),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Junction {
                id: JunctionId::new(50),
            },
        ));

        // Wire 2: Junction(50) -> Pin(10) [Gate 1]
        circuit.add_wire(WireSegment::new(
            WireId::new(2),
            Endpoint::Junction {
                id: JunctionId::new(50),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(100),
                pin_id: PinId::new(10),
            },
        ));

        // Wire 3: Junction(50) -> Pin(20) [Gate 2]
        circuit.add_wire(WireSegment::new(
            WireId::new(3),
            Endpoint::Junction {
                id: JunctionId::new(50),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(200),
                pin_id: PinId::new(20),
            },
        ));

        let mut dsu = resolve_circuit_connectivity(&circuit);

        let port_node = ConnectionNode::Port { id: PinId::new(1) };
        let junc_node = ConnectionNode::Junction {
            id: JunctionId::new(50),
        };
        let pin10_node = ConnectionNode::Pin {
            instance_id: InstanceId::new(100),
            pin_id: PinId::new(10),
        };
        let pin20_node = ConnectionNode::Pin {
            instance_id: InstanceId::new(200),
            pin_id: PinId::new(20),
        };
        let pin11_node = ConnectionNode::Pin {
            instance_id: InstanceId::new(100),
            pin_id: PinId::new(11),
        };

        // Seluruh cabang fanout harus saling terhubung
        assert!(dsu.is_connected(port_node, junc_node));
        assert!(dsu.is_connected(port_node, pin10_node));
        assert!(dsu.is_connected(port_node, pin20_node));
        assert!(dsu.is_connected(pin10_node, pin20_node));

        // Pin output Gate 1 (11) tidak terhubung ke net clock
        assert!(!dsu.is_connected(port_node, pin11_node));

        // Canonical root dari net ini harus bernilai Port(1) karena ID-nya terkecil (1 < 10, 20, 50)
        assert_eq!(dsu.canonical(pin10_node), port_node);
        assert_eq!(dsu.canonical(pin20_node), port_node);
        assert_eq!(dsu.canonical(junc_node), port_node);
    }

    #[test]
    fn test_chained_junctions() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Chained Junctions");

        // Port(10) -> J(20) -> J(30) -> Port(40)
        circuit.add_port(PortDefinition::new(
            PinId::new(10),
            "p_in",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(40),
            "p_out",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));
        circuit.add_junction(Junction::new(JunctionId::new(20), Point::ZERO));
        circuit.add_junction(Junction::new(JunctionId::new(30), Point::ZERO));

        circuit.add_wire(WireSegment::new(
            WireId::new(1),
            Endpoint::Port { id: PinId::new(10) },
            Endpoint::Junction {
                id: JunctionId::new(20),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(2),
            Endpoint::Junction {
                id: JunctionId::new(20),
            },
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(3),
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
            Endpoint::Port { id: PinId::new(40) },
        ));

        let mut dsu = resolve_circuit_connectivity(&circuit);

        let p10 = ConnectionNode::Port { id: PinId::new(10) };
        let p40 = ConnectionNode::Port { id: PinId::new(40) };

        assert!(dsu.is_connected(p10, p40));
        assert_eq!(dsu.canonical(p40), p10);
    }
}
