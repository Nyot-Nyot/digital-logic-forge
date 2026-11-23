//! # Resolusi Koneksi Nirkabel Tunnel (Wireless Tunnel Resolution)
//!
//! Modul ini mengelompokkan dan menyatukan entitas `Tunnel` nirkabel sesuai Bab 3.4.2 Aturan 7 proposal.
//! Tunnel dengan nama label yang identik dalam satu sirkuit/scope digabungkan ke dalam net yang sama.
//! Tunnel yang berdiri sendiri tanpa pasangan dideteksi dan dilaporkan sebagai diagnostik `CONN-TUNNEL-UNPAIRED`.

use super::dsu::DisjointSet;
use super::node::ConnectionNode;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticStage, SourceRef};
use crate::model::circuit::CircuitDefinition;
use crate::model::wiring::TunnelScope;
use crate::validation::ValidationReport;
use std::collections::BTreeMap;

/// Menyatukan seluruh entitas tunnel yang memiliki nama dan cakupan (*scope*) yang sama via DSU,
/// serta melaporkan diagnostik `CONN-TUNNEL-UNPAIRED` jika ditemukan tunnel tanpa pasangan.
pub fn resolve_circuit_tunnels(
    circuit: &CircuitDefinition,
    dsu: &mut DisjointSet<ConnectionNode>,
    report: &mut ValidationReport,
) {
    // Kelompokkan tunnel berdasarkan (nama, scope)
    let mut groups: BTreeMap<(String, TunnelScope), Vec<crate::model::id::TunnelId>> =
        BTreeMap::new();

    for tunnel in &circuit.tunnels {
        let tun_node = ConnectionNode::Tunnel { id: tunnel.id };
        let pin_node = ConnectionNode::Port { id: tunnel.pin_id };
        dsu.insert(pin_node);
        dsu.union(tun_node, pin_node);

        groups
            .entry((tunnel.name.clone(), tunnel.scope))
            .or_default()
            .push(tunnel.id);
    }

    for ((name, scope), tunnel_ids) in groups {
        if tunnel_ids.len() == 1 {
            // Tunnel berdiri sendiri tanpa pasangan
            report.add(Diagnostic::warning(
                DiagnosticCode::ConnTunnelUnpaired,
                format!(
                    "Label tunnel '{}' (scope: {}) tidak memiliki pasangan di dalam sirkuit",
                    name, scope
                ),
                DiagnosticStage::Connectivity,
                SourceRef::Tunnel { id: tunnel_ids[0] },
            ));
        } else if tunnel_ids.len() > 1 {
            // Gabungkan seluruh tunnel bernama sama ke partisi yang sama
            let first_node = ConnectionNode::Tunnel { id: tunnel_ids[0] };
            for &tid in &tunnel_ids[1..] {
                let other_node = ConnectionNode::Tunnel { id: tid };
                dsu.union(first_node, other_node);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::PortDefinition;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, PinId, TunnelId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, Tunnel, WireSegment};

    #[test]
    fn test_paired_tunnels_resolution() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Tunnel Test");

        // Port In (10) -> Tunnel Tx (1) "DATA_BUS"
        // Tunnel Rx (2) "DATA_BUS" -> Port Out (20)
        circuit.add_port(PortDefinition::new(
            PinId::new(10),
            "in",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(20),
            "out",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));

        let t1 = Tunnel::new(
            TunnelId::new(1),
            "DATA_BUS",
            TunnelScope::Local,
            Point::ZERO,
            PinId::new(101),
        );
        let t2 = Tunnel::new(
            TunnelId::new(2),
            "DATA_BUS",
            TunnelScope::Local,
            Point::ZERO,
            PinId::new(102),
        );
        circuit.add_tunnel(t1);
        circuit.add_tunnel(t2);

        // Kawat 1: Port(10) -> Tunnel(1)
        circuit.add_wire(WireSegment::new(
            WireId::new(1),
            Endpoint::Port { id: PinId::new(10) },
            Endpoint::Junction {
                id: crate::model::id::JunctionId::new(1),
            }, // Representasi endpoint tunnel melalui junction/port
        ));

        let mut dsu = DisjointSet::new();
        let node_t1 = ConnectionNode::Tunnel {
            id: TunnelId::new(1),
        };
        let node_t2 = ConnectionNode::Tunnel {
            id: TunnelId::new(2),
        };
        let port_in = ConnectionNode::Port { id: PinId::new(10) };
        let port_out = ConnectionNode::Port { id: PinId::new(20) };

        dsu.insert(node_t1);
        dsu.insert(node_t2);
        dsu.insert(port_in);
        dsu.insert(port_out);

        // Kawat menghubungkan port_in ke node_t1 dan node_t2 ke port_out
        dsu.union(port_in, node_t1);
        dsu.union(node_t2, port_out);

        assert!(!dsu.is_connected(port_in, port_out));

        // Jalankan resolusi tunnel
        let mut report = ValidationReport::new();
        resolve_circuit_tunnels(&circuit, &mut dsu, &mut report);

        // Setelah resolusi, port_in dan port_out harus terhubung via tunnel "DATA_BUS"
        assert!(dsu.is_connected(port_in, port_out));
        assert!(dsu.is_connected(node_t1, node_t2));
        assert!(report.is_clean());
    }

    #[test]
    fn test_unpaired_tunnel_detected() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Unpaired Test");

        // Tunnel berdiri sendiri
        circuit.add_tunnel(Tunnel::new(
            TunnelId::new(99),
            "LONELY_SIGNAL",
            TunnelScope::Local,
            Point::ZERO,
            PinId::new(100),
        ));

        let mut dsu = DisjointSet::new();
        let mut report = ValidationReport::new();
        resolve_circuit_tunnels(&circuit, &mut dsu, &mut report);

        assert_eq!(report.warning_count(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::ConnTunnelUnpaired
        );
    }
}
