//! # Node Konektivitas dan Ekstraksi Endpoint (Connection Nodes)
//!
//! Modul ini memodelkan simpul-simpul sambungan listrik (`ConnectionNode`) yang diekstraksi
//! dari entitas dokumen rangkaian (`CircuitDefinition`), seperti Pin komponen, Port sirkuit,
//! titik cabang Junction, dan label Tunnel nirkabel sesuai Bab 3.4.2 proposal.

use crate::model::circuit::CircuitDefinition;
use crate::model::id::{InstanceId, JunctionId, PinId, TunnelId};
use crate::model::wiring::Endpoint;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Simpul konektivitas listrik terpadu pada sirkuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionNode {
    /// Terminal pin konkret pada instance komponen.
    Pin {
        /// ID instance komponen pemilik pin.
        instance_id: InstanceId,
        /// ID pin konkret.
        pin_id: PinId,
    },
    /// Port batas antarmuka sirkuit.
    Port {
        /// ID port pin antarmuka.
        id: PinId,
    },
    /// Titik cabang kawat (*junction*).
    Junction {
        /// ID junction.
        id: JunctionId,
    },
    /// Label terminal koneksi nirkabel (*tunnel*).
    Tunnel {
        /// ID entitas tunnel.
        id: TunnelId,
    },
}

impl ConnectionNode {
    /// Mengambil representasi nilai ID bilangan bulat 64-bit mentah dari node ini.
    #[inline]
    pub const fn id_u64(&self) -> u64 {
        match self {
            Self::Pin { pin_id, .. } => pin_id.get(),
            Self::Port { id } => id.get(),
            Self::Junction { id } => id.get(),
            Self::Tunnel { id } => id.get(),
        }
    }

    /// Membuat `ConnectionNode` dari `Endpoint` kawat jika endpoint tersebut terhubung.
    pub fn from_endpoint(endpoint: &Endpoint) -> Option<Self> {
        match endpoint {
            Endpoint::Pin {
                instance_id,
                pin_id,
            } => Some(Self::Pin {
                instance_id: *instance_id,
                pin_id: *pin_id,
            }),
            Endpoint::Port { id } => Some(Self::Port { id: *id }),
            Endpoint::Junction { id } => Some(Self::Junction { id: *id }),
            Endpoint::Free { .. } => None,
        }
    }
}

impl PartialOrd for ConnectionNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConnectionNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Urutkan berdasarkan ID numerik mentah terlebih dahulu untuk menjamin determinisme kanonis
        self.id_u64().cmp(&other.id_u64())
    }
}

impl Display for ConnectionNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pin {
                instance_id,
                pin_id,
            } => write!(f, "Pin({}:{})", instance_id, pin_id),
            Self::Port { id } => write!(f, "Port({})", id),
            Self::Junction { id } => write!(f, "Junction({})", id),
            Self::Tunnel { id } => write!(f, "Tunnel({})", id),
        }
    }
}

/// Mengekstraksi seluruh simpul sambungan (`ConnectionNode`) yang ada di dalam lembar sirkuit.
pub fn extract_circuit_nodes(circuit: &CircuitDefinition) -> Vec<ConnectionNode> {
    let mut nodes = Vec::new();

    // 1. Ekstraksi Port antarmuka
    for port in &circuit.ports {
        nodes.push(ConnectionNode::Port { id: port.id });
    }

    // 2. Ekstraksi Pin dari seluruh instance komponen
    for comp in &circuit.components {
        for pin in &comp.pins {
            nodes.push(ConnectionNode::Pin {
                instance_id: comp.id,
                pin_id: pin.id,
            });
        }
    }

    // 3. Ekstraksi Junction
    for junc in &circuit.junctions {
        nodes.push(ConnectionNode::Junction { id: junc.id });
    }

    // 4. Ekstraksi Tunnel
    for tun in &circuit.tunnels {
        nodes.push(ConnectionNode::Tunnel { id: tun.id });
    }

    // Urutkan secara stabil
    nodes.sort_unstable();
    nodes
}

/// Mengekstraksi seluruh koneksi dua titik dari segmen kawat yang sah di dalam lembar sirkuit.
///
/// Segmen kawat yang memiliki ujung bebas (`Endpoint::Free`) diabaikan dari daftar sambungan aktif.
pub fn extract_wire_connections(
    circuit: &CircuitDefinition,
) -> Vec<(ConnectionNode, ConnectionNode)> {
    let mut connections = Vec::new();

    for wire in &circuit.wires {
        if let (Some(node_a), Some(node_b)) = (
            ConnectionNode::from_endpoint(&wire.start),
            ConnectionNode::from_endpoint(&wire.end),
        ) {
            connections.push((node_a, node_b));
        }
    }

    connections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::PortDefinition;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Junction, WireSegment};

    #[test]
    fn test_extract_circuit_nodes_and_wires() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Extraction Test");
        let registry = ComponentRegistry::with_builtins();

        // 1 Port
        circuit.add_port(PortDefinition::new(
            PinId::new(10),
            "in_a",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));

        // 1 Komponen NOT (2 pin: a=20, y=21)
        let not_def = registry.get("dlf.builtin.not").unwrap();
        let not_inst = not_def
            .instantiate(
                InstanceId::new(5),
                &[PinId::new(20), PinId::new(21)],
                Point::ZERO,
            )
            .unwrap();
        circuit.add_component(not_inst);

        // 1 Junction
        circuit.add_junction(Junction::new(JunctionId::new(30), Point::ZERO));

        // 1 Wire: Port(10) -> Junction(30)
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Port { id: PinId::new(10) },
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
        ));

        // 1 Dangling wire: Junction(30) -> Free
        circuit.add_wire(WireSegment::new(
            WireId::new(101),
            Endpoint::Junction {
                id: JunctionId::new(30),
            },
            Endpoint::Free {
                position: Point::ZERO,
            },
        ));

        let nodes = extract_circuit_nodes(&circuit);
        assert_eq!(nodes.len(), 4); // Port(10), Pin(20), Pin(21), Junction(30)

        let conns = extract_wire_connections(&circuit);
        assert_eq!(conns.len(), 1); // Hanya wire 100 yang terhubung penuh
        assert_eq!(conns[0].0, ConnectionNode::Port { id: PinId::new(10) });
        assert_eq!(
            conns[0].1,
            ConnectionNode::Junction {
                id: JunctionId::new(30)
            }
        );
    }
}
