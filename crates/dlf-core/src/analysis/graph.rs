//! # Graf Ketergantungan Kombinasional (Combinational Dependency Graph)
//!
//! Modul ini mengonstruksi representasi graf ketergantungan terarah (*Dependency Graph*)
//! antara simpul-simpul operasi logika dalam lembar sirkuit sesuai Bab 3.4.1 dan Bab 3.10.11 proposal.
//!
//! Sisi terarah $u \to v$ merepresentasikan aliran sinyal kombinasional di mana simpul $v$
//! bergantung pada nilai keluaran simpul $u$. Komponen sekuensial (seperti register DFF)
//! dipisahkan menjadi `SequentialSource` (state read) dan `SequentialSink` (state write)
//! sehingga memutus siklus kombinasional antar-siklus clock.

use crate::connectivity::{NetTerminal, Netlist};
use crate::model::circuit::CircuitDefinition;
use crate::model::component::{BehaviorKind, ComponentRegistry};
use crate::model::id::{InstanceId, PinId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};

/// Simpul operasi fungsional pada graf ketergantungan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GraphNode {
    /// Komponen logika kombinasional murni (output seketika bergantung pada input).
    Combinational(InstanceId),
    /// Sisi keluaran elemen sekuensial (sumber nilai awal siklus, batas level 0).
    SequentialSource(InstanceId),
    /// Sisi masukan elemen sekuensial (beban nilai yang disampel pada batas clock).
    SequentialSink(InstanceId),
    /// Port masukan batas antarmuka lembar sirkuit (sumber sinyal luar, level 0).
    InputPort(PinId),
    /// Port keluaran batas antarmuka lembar sirkuit (beban akhir nilai sirkuit).
    OutputPort(PinId),
}

impl GraphNode {
    /// Mengambil `InstanceId` jika simpul ini terkait dengan komponen sirkuit.
    #[inline]
    pub const fn instance_id(&self) -> Option<InstanceId> {
        match *self {
            Self::Combinational(id) | Self::SequentialSource(id) | Self::SequentialSink(id) => {
                Some(id)
            }
            Self::InputPort(_) | Self::OutputPort(_) => None,
        }
    }

    /// Mengambil `PinId` jika simpul ini merupakan port antarmuka sirkuit.
    #[inline]
    pub const fn port_id(&self) -> Option<PinId> {
        match *self {
            Self::InputPort(id) | Self::OutputPort(id) => Some(id),
            Self::Combinational(_) | Self::SequentialSource(_) | Self::SequentialSink(_) => None,
        }
    }

    /// Memeriksa apakah simpul ini merupakan bagian dari elemen sekuensial.
    #[inline]
    pub const fn is_sequential(&self) -> bool {
        matches!(self, Self::SequentialSource(_) | Self::SequentialSink(_))
    }
}

impl Display for GraphNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Combinational(id) => write!(f, "Combinational({})", id),
            Self::SequentialSource(id) => write!(f, "SequentialSource({})", id),
            Self::SequentialSink(id) => write!(f, "SequentialSink({})", id),
            Self::InputPort(id) => write!(f, "InputPort({})", id),
            Self::OutputPort(id) => write!(f, "OutputPort({})", id),
        }
    }
}

/// Graf ketergantungan terarah (*Directed Dependency Graph*) untuk analisis jadwal eksekusi.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Seluruh simpul yang terdaftar pada graf.
    pub nodes: BTreeSet<GraphNode>,
    /// Sisi terarah keluar (forward adjacency): $u \to \{v\}$ (simpul $v$ membutuhkan nilai simpul $u$).
    pub adj: BTreeMap<GraphNode, BTreeSet<GraphNode>>,
    /// Sisi terarah masuk (reverse adjacency): $v \to \{u\}$ (simpul-simpul $u$ yang dibutuhkan oleh $v$).
    pub rev_adj: BTreeMap<GraphNode, BTreeSet<GraphNode>>,
}

impl DependencyGraph {
    /// Membuat graf ketergantungan baru yang kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Membangun graf ketergantungan kombinasional dari lembar sirkuit, netlist kanonis, dan pustaka komponen.
    pub fn from_circuit_and_netlist(
        circuit: &CircuitDefinition,
        netlist: &Netlist,
        registry: &ComponentRegistry,
    ) -> Self {
        let mut graph = Self::new();

        // 1. Petakan karakteristik perilaku seluruh instance komponen
        let mut behavior_map = BTreeMap::new();
        for comp in &circuit.components {
            let kind = registry
                .get(&comp.definition_id)
                .map(|def| def.behavior_kind)
                .unwrap_or(BehaviorKind::Combinational);
            behavior_map.insert(comp.id, kind);

            match kind {
                BehaviorKind::Sequential => {
                    graph.add_node(GraphNode::SequentialSource(comp.id));
                    graph.add_node(GraphNode::SequentialSink(comp.id));
                }
                _ => {
                    graph.add_node(GraphNode::Combinational(comp.id));
                }
            }
        }

        // 2. Petakan port antarmuka sirkuit
        for port in &circuit.ports {
            if port.direction.is_input() {
                graph.add_node(GraphNode::InputPort(port.id));
            } else {
                graph.add_node(GraphNode::OutputPort(port.id));
            }
        }

        // 3. Bentuk sisi terarah dari driver net ke beban net
        for net in netlist.nets.values() {
            for driver in &net.drivers {
                let driver_node = match *driver {
                    NetTerminal::Port { id } => GraphNode::InputPort(id),
                    NetTerminal::Pin { instance_id, .. } => {
                        let kind = behavior_map
                            .get(&instance_id)
                            .copied()
                            .unwrap_or(BehaviorKind::Combinational);
                        if kind == BehaviorKind::Sequential {
                            GraphNode::SequentialSource(instance_id)
                        } else {
                            GraphNode::Combinational(instance_id)
                        }
                    }
                };

                for load in &net.loads {
                    let load_node = match *load {
                        NetTerminal::Port { id } => GraphNode::OutputPort(id),
                        NetTerminal::Pin { instance_id, .. } => {
                            let kind = behavior_map
                                .get(&instance_id)
                                .copied()
                                .unwrap_or(BehaviorKind::Combinational);
                            if kind == BehaviorKind::Sequential {
                                GraphNode::SequentialSink(instance_id)
                            } else {
                                GraphNode::Combinational(instance_id)
                            }
                        }
                    };

                    graph.add_edge(driver_node, load_node);
                }
            }
        }

        graph
    }

    /// Membangun graf ketergantungan langsung dari `CircuitDefinition` menggunakan pustaka built-in.
    pub fn from_circuit(circuit: &CircuitDefinition, registry: &ComponentRegistry) -> Self {
        let netlist = Netlist::from_circuit(circuit);
        Self::from_circuit_and_netlist(circuit, &netlist, registry)
    }

    /// Mendaftarkan sebuah simpul ke dalam graf.
    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node);
        self.adj.entry(node).or_default();
        self.rev_adj.entry(node).or_default();
    }

    /// Menambahkan sisi terarah dari `from` ke `to` ($from \to to$).
    pub fn add_edge(&mut self, from: GraphNode, to: GraphNode) {
        self.add_node(from);
        self.add_node(to);
        self.adj.entry(from).or_default().insert(to);
        self.rev_adj.entry(to).or_default().insert(from);
    }

    /// Mengambil seluruh simpul penerus (*successors*) dari simpul `node`.
    pub fn successors(&self, node: GraphNode) -> impl Iterator<Item = GraphNode> + '_ {
        self.adj
            .get(&node)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Mengambil seluruh simpul pendahulu (*predecessors*) dari simpul `node`.
    pub fn predecessors(&self, node: GraphNode) -> impl Iterator<Item = GraphNode> + '_ {
        self.rev_adj
            .get(&node)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Derajat masuk (*in-degree*) dari sebuah simpul.
    #[inline]
    pub fn in_degree(&self, node: GraphNode) -> usize {
        self.rev_adj.get(&node).map_or(0, BTreeSet::len)
    }

    /// Derajat keluar (*out-degree*) dari sebuah simpul.
    #[inline]
    pub fn out_degree(&self, node: GraphNode) -> usize {
        self.adj.get(&node).map_or(0, BTreeSet::len)
    }

    /// Jumlah total simpul dalam graf.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Jumlah total sisi terarah dalam graf.
    pub fn edge_count(&self) -> usize {
        self.adj.values().map(BTreeSet::len).sum()
    }

    /// Memeriksa apakah simpul tertentu ada dalam graf.
    #[inline]
    pub fn contains_node(&self, node: GraphNode) -> bool {
        self.nodes.contains(&node)
    }

    /// Memeriksa apakah terdapat sisi terarah langsung dari `from` ke `to`.
    pub fn contains_edge(&self, from: GraphNode, to: GraphNode) -> bool {
        self.adj
            .get(&from)
            .is_some_and(|targets| targets.contains(&to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, WireId};
    use crate::model::wiring::{Endpoint, WireSegment};

    #[test]
    fn test_half_adder_dependency_graph() {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/simple_circuit.json"
        );
        let content = std::fs::read_to_string(fixture_path).expect("Gagal membaca fixture");
        let project: crate::model::project::Project =
            crate::model::serialization::from_json_str(&content).expect("Gagal deserialize");

        let circuit = project
            .get_circuit(CircuitId::new(1))
            .expect("Circuit not found");
        let registry = ComponentRegistry::with_builtins();
        let graph = DependencyGraph::from_circuit(circuit, &registry);

        // Simpul yang diharapkan:
        // - Input Port 2 (a) & 3 (b)
        // - Output Port 4 (sum) & 5 (carry)
        // - Combinational 6 (XOR1) & 10 (AND1)
        assert_eq!(graph.node_count(), 6);

        let in_a = GraphNode::InputPort(PinId::new(2));
        let in_b = GraphNode::InputPort(PinId::new(3));
        let xor1 = GraphNode::Combinational(InstanceId::new(6));
        let and1 = GraphNode::Combinational(InstanceId::new(10));
        let out_sum = GraphNode::OutputPort(PinId::new(4));
        let out_carry = GraphNode::OutputPort(PinId::new(5));

        // Verifikasi sisi terarah
        assert!(graph.contains_edge(in_a, xor1));
        assert!(graph.contains_edge(in_a, and1));
        assert!(graph.contains_edge(in_b, xor1));
        assert!(graph.contains_edge(in_b, and1));
        assert!(graph.contains_edge(xor1, out_sum));
        assert!(graph.contains_edge(and1, out_carry));

        // Derajat in-degree & out-degree
        assert_eq!(graph.in_degree(in_a), 0);
        assert_eq!(graph.out_degree(in_a), 2);
        assert_eq!(graph.in_degree(xor1), 2);
        assert_eq!(graph.out_degree(xor1), 1);
        assert_eq!(graph.in_degree(out_sum), 1);
        assert_eq!(graph.out_degree(out_sum), 0);
    }

    #[test]
    fn test_sequential_dff_breaks_combinational_cycle() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Sequential Feedback Test");
        let registry = ComponentRegistry::with_builtins();

        // 1 buah DFF dan 1 buah gerbang NOT
        // DFF.q -> NOT.in, NOT.out -> DFF.d
        // Pada graf kombinasional:
        // SequentialSource(DFF) -> Combinational(NOT) -> SequentialSink(DFF)
        // Tidak membentuk siklus karena SequentialSink tidak terhubung ke SequentialSource pada graf siklus tunggal.
        let dff_def = registry.get("dlf.builtin.dff").expect("dff definition");
        let not_def = registry.get("dlf.builtin.not").expect("not definition");

        let dff_inst = dff_def
            .instantiate(
                InstanceId::new(10),
                &[
                    PinId::new(11), // d (in)
                    PinId::new(12), // clk (in)
                    PinId::new(13), // en (in)
                    PinId::new(14), // rst (in)
                    PinId::new(15), // q (out)
                ],
                Point::new(0.0, 0.0),
            )
            .unwrap();

        let not_inst = not_def
            .instantiate(
                InstanceId::new(20),
                &[
                    PinId::new(21), // in
                    PinId::new(22), // out
                ],
                Point::new(100.0, 0.0),
            )
            .unwrap();

        circuit.add_component(dff_inst);
        circuit.add_component(not_inst);

        // Kawat 1: DFF.q (15) -> NOT.in (21)
        circuit.add_wire(WireSegment::new(
            WireId::new(101),
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(15),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(21),
            },
        ));

        // Kawat 2: NOT.out (22) -> DFF.d (11)
        circuit.add_wire(WireSegment::new(
            WireId::new(102),
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(22),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
        ));

        let graph = DependencyGraph::from_circuit(&circuit, &registry);

        let dff_src = GraphNode::SequentialSource(InstanceId::new(10));
        let dff_sink = GraphNode::SequentialSink(InstanceId::new(10));
        let not_node = GraphNode::Combinational(InstanceId::new(20));

        assert!(graph.contains_node(dff_src));
        assert!(graph.contains_node(dff_sink));
        assert!(graph.contains_node(not_node));

        // DFF.q -> NOT.in
        assert!(graph.contains_edge(dff_src, not_node));
        // NOT.out -> DFF.d
        assert!(graph.contains_edge(not_node, dff_sink));

        // Tidak ada edge balik dari sink ke source
        assert!(!graph.contains_edge(dff_sink, dff_src));
        assert_eq!(graph.in_degree(dff_src), 0);
        assert_eq!(graph.out_degree(dff_sink), 0);
    }
}
