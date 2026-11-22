//! # Konstruksi Netlist Kanonis dan Model Net (Canonical Netlist & Net Model)
//!
//! Modul ini mengimplementasikan representasi netlist elektris kanonis (`Netlist` dan `Net`)
//! yang dibangun dari partisi konektivitas topologis (`DisjointSet<ConnectionNode>`) dan
//! lembar sirkuit (`CircuitDefinition`) sesuai Bab 3.4.1, 3.4.2, dan 3.10.11 proposal.

use super::dsu::DisjointSet;
use super::node::ConnectionNode;
use super::resolver::resolve_circuit_connectivity;
use crate::diagnostic::SourceRef;
use crate::model::circuit::CircuitDefinition;
use crate::model::id::{InstanceId, PinId};
use crate::model::pin::{PinDirection, SignalWidth};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

/// Pengidentifikasi unik kanonis untuk jaring konektivitas listrik (*Net*).
///
/// Nilai ID kanonis dijamin deterministik karena selalu mengambil ID numerik
/// minimum dari simpul-simpul yang tergabung di dalam partisi DSU terkait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NetId(pub u64);

impl NetId {
    /// Membuat NetId baru dari integer 64-bit.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Mengambil nilai integer 64-bit mentah dari NetId.
    #[inline]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for NetId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl Display for NetId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Net({})", self.0)
    }
}

/// Terminal fungsional yang terlibat dalam pertukaran sinyal listrik (Pin komponen atau Port sirkuit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NetTerminal {
    /// Pin pada instance komponen tertentu di dalam sirkuit.
    Pin {
        /// ID instance komponen pemilik pin.
        instance_id: InstanceId,
        /// ID pin komponen.
        pin_id: PinId,
    },
    /// Port batas antarmuka lembar sirkuit.
    Port {
        /// ID port antarmuka.
        id: PinId,
    },
}

impl NetTerminal {
    /// Mencoba mengonversi `ConnectionNode` menjadi `NetTerminal`.
    ///
    /// Mengembalikan `Some(NetTerminal)` jika simpul adalah Pin atau Port,
    /// atau `None` jika simpul adalah Junction atau Tunnel.
    /// Mencoba mengonversi `ConnectionNode` menjadi `NetTerminal`.
    ///
    /// Mengembalikan `Some(NetTerminal)` jika simpul adalah Pin atau Port,
    /// atau `None` jika simpul adalah Junction atau Tunnel.
    pub fn from_node(node: ConnectionNode) -> Option<Self> {
        match node {
            ConnectionNode::Pin {
                instance_id,
                pin_id,
            } => Some(Self::Pin {
                instance_id,
                pin_id,
            }),
            ConnectionNode::Port { id } => Some(Self::Port { id }),
            ConnectionNode::Junction { .. } | ConnectionNode::Tunnel { .. } => None,
        }
    }

    /// Mengonversi terminal fungsional menjadi referensi visual `SourceRef`.
    #[inline]
    pub const fn to_source_ref(&self) -> SourceRef {
        match *self {
            Self::Pin {
                instance_id,
                pin_id,
            } => SourceRef::Pin {
                instance_id,
                pin_id,
            },
            Self::Port { id } => SourceRef::Port { id },
        }
    }
}

impl Display for NetTerminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pin {
                instance_id,
                pin_id,
            } => write!(f, "Pin({}:{})", instance_id, pin_id),
            Self::Port { id } => write!(f, "Port({})", id),
        }
    }
}

/// Representasi satu jaring konektivitas listrik kanonis (*net*).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Net {
    /// Pengidentifikasi unik kanonis net ini (berasal dari ID terkecil simpul dalam partisi DSU).
    pub id: NetId,
    /// Lebar bus sinyal pada net ini (default `SignalWidth::SINGLE`).
    pub width: SignalWidth,
    /// Daftar terminal yang bertindak sebagai penggerak (*driver*) sinyal.
    pub drivers: Vec<NetTerminal>,
    /// Daftar terminal yang bertindak sebagai beban (*load*) penerima sinyal.
    pub loads: Vec<NetTerminal>,
    /// Seluruh simpul konektivitas (`ConnectionNode`) yang terhubung pada net ini.
    pub nodes: Vec<ConnectionNode>,
}

impl Net {
    /// Memeriksa apakah net memiliki setidaknya satu penggerak (*driver*).
    #[inline]
    pub fn has_driver(&self) -> bool {
        !self.drivers.is_empty()
    }

    /// Memeriksa apakah net memiliki setidaknya satu beban (*load*).
    #[inline]
    pub fn has_load(&self) -> bool {
        !self.loads.is_empty()
    }

    /// Memeriksa apakah net memiliki lebih dari satu penggerak aktif (*multi-driver conflict*).
    #[inline]
    pub fn is_multi_driver(&self) -> bool {
        self.drivers.len() > 1
    }

    /// Memeriksa apakah net memiliki beban tetapi tidak memiliki penggerak sama sekali (*undriven net*).
    #[inline]
    pub fn is_undriven(&self) -> bool {
        self.drivers.is_empty() && !self.loads.is_empty()
    }
}

/// Kumpulan seluruh jaring konektivitas listrik kanonis (*net*) dalam lembar sirkuit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Netlist {
    /// Seluruh net yang terdaftar, dipetakan dari `NetId` kanonis secara terurut deterministik.
    pub nets: BTreeMap<NetId, Net>,
    /// Pemetaan cepat dari terminal fungsional (`NetTerminal`) ke `NetId` pemiliknya.
    pub terminal_to_net: BTreeMap<NetTerminal, NetId>,
    /// Pemetaan cepat dari seluruh simpul konektivitas (`ConnectionNode`) ke `NetId` kanonisnya.
    pub node_to_net: BTreeMap<ConnectionNode, NetId>,
}

impl Netlist {
    /// Membuat kontainer `Netlist` baru yang kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Membangun `Netlist` kanonis langsung dari `CircuitDefinition`.
    ///
    /// Menjalankan resolusi topologi kawat, junction, dan tunnel via DSU,
    /// lalu mengelompokkan simpul menjadi netlist terstruktur.
    pub fn from_circuit(circuit: &CircuitDefinition) -> Self {
        let mut dsu = resolve_circuit_connectivity(circuit);
        Self::from_circuit_and_dsu(circuit, &mut dsu)
    }

    /// Membangun `Netlist` kanonis dari `CircuitDefinition` dan partisi `DisjointSet` yang sudah ada.
    pub fn from_circuit_and_dsu(
        circuit: &CircuitDefinition,
        dsu: &mut DisjointSet<ConnectionNode>,
    ) -> Self {
        let mut netlist = Self::new();
        let sets = dsu.sets();

        // 1. Indeks port antarmuka untuk lookup cepat O(log P)
        let mut port_map = BTreeMap::new();
        for port in &circuit.ports {
            port_map.insert(port.id, (port.direction, port.width));
        }

        // 2. Indeks pin komponen untuk lookup cepat O(log C)
        let mut comp_pin_map = BTreeMap::new();
        for comp in &circuit.components {
            for pin in &comp.pins {
                comp_pin_map.insert((comp.id, pin.id), (pin.direction, pin.width));
            }
        }

        // 3. Bangun objek Net untuk setiap partisi himpunan saling lepas
        for (canon_node, members) in sets {
            let net_id = NetId::new(canon_node.id_u64());
            let mut drivers = Vec::new();
            let mut loads = Vec::new();
            let mut driver_width: Option<SignalWidth> = None;
            let mut load_width: Option<SignalWidth> = None;

            for &node in &members {
                match node {
                    ConnectionNode::Pin {
                        instance_id,
                        pin_id,
                    } => {
                        let terminal = NetTerminal::Pin {
                            instance_id,
                            pin_id,
                        };
                        netlist.terminal_to_net.insert(terminal, net_id);

                        if let Some(&(direction, width)) = comp_pin_map.get(&(instance_id, pin_id))
                        {
                            match direction {
                                PinDirection::Output => {
                                    drivers.push(terminal);
                                    if driver_width.is_none() {
                                        driver_width = Some(width);
                                    }
                                }
                                PinDirection::Input => {
                                    loads.push(terminal);
                                    if load_width.is_none() {
                                        load_width = Some(width);
                                    }
                                }
                                PinDirection::Inout => {
                                    drivers.push(terminal);
                                    loads.push(terminal);
                                    if driver_width.is_none() {
                                        driver_width = Some(width);
                                    }
                                }
                            }
                        }
                    }
                    ConnectionNode::Port { id } => {
                        let terminal = NetTerminal::Port { id };
                        netlist.terminal_to_net.insert(terminal, net_id);

                        if let Some(&(direction, width)) = port_map.get(&id) {
                            // Di dalam lembar sirkuit:
                            // Port Input menyuntikkan sinyal dari luar sirkuit -> bertindak sebagai DRIVER net internal.
                            // Port Output menerima sinyal dari net internal -> bertindak sebagai LOAD net internal.
                            match direction {
                                PinDirection::Input => {
                                    drivers.push(terminal);
                                    if driver_width.is_none() {
                                        driver_width = Some(width);
                                    }
                                }
                                PinDirection::Output => {
                                    loads.push(terminal);
                                    if load_width.is_none() {
                                        load_width = Some(width);
                                    }
                                }
                                PinDirection::Inout => {
                                    drivers.push(terminal);
                                    loads.push(terminal);
                                    if driver_width.is_none() {
                                        driver_width = Some(width);
                                    }
                                }
                            }
                        }
                    }
                    ConnectionNode::Junction { .. } | ConnectionNode::Tunnel { .. } => {
                        // Titik percabangan murni, bukan terminal elektris aktif
                    }
                }

                netlist.node_to_net.insert(node, net_id);
            }

            // Urutkan terminal secara stabil dan deterministik
            drivers.sort_unstable();
            drivers.dedup();
            loads.sort_unstable();
            loads.dedup();

            let width = driver_width.or(load_width).unwrap_or(SignalWidth::SINGLE);

            let net = Net {
                id: net_id,
                width,
                drivers,
                loads,
                nodes: members,
            };

            netlist.nets.insert(net_id, net);
        }

        netlist
    }

    /// Mengambil referensi ke net berdasarkan `NetId`.
    #[inline]
    pub fn get_net(&self, id: NetId) -> Option<&Net> {
        self.nets.get(&id)
    }

    /// Mencari `NetId` yang terhubung ke suatu terminal fungsional (`NetTerminal`).
    #[inline]
    pub fn net_for_terminal(&self, terminal: &NetTerminal) -> Option<NetId> {
        self.terminal_to_net.get(terminal).copied()
    }

    /// Mencari `NetId` yang terhubung ke pin instance komponen tertentu.
    #[inline]
    pub fn net_for_pin(&self, instance_id: InstanceId, pin_id: PinId) -> Option<NetId> {
        self.terminal_to_net
            .get(&NetTerminal::Pin {
                instance_id,
                pin_id,
            })
            .copied()
    }

    /// Mencari `NetId` yang terhubung ke port antarmuka sirkuit.
    #[inline]
    pub fn net_for_port(&self, port_id: PinId) -> Option<NetId> {
        self.terminal_to_net
            .get(&NetTerminal::Port { id: port_id })
            .copied()
    }

    /// Mencari `NetId` yang terhubung ke simpul konektivitas (`ConnectionNode`).
    #[inline]
    pub fn net_for_node(&self, node: &ConnectionNode) -> Option<NetId> {
        self.node_to_net.get(node).copied()
    }

    /// Mengembalikan jumlah net yang terdaftar.
    #[inline]
    pub fn len(&self) -> usize {
        self.nets.len()
    }

    /// Memeriksa apakah netlist tidak memiliki net sama sekali.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nets.is_empty()
    }

    /// Iterasi atas seluruh pasangan (`NetId`, `Net`) secara terurut deterministik.
    pub fn iter(&self) -> impl Iterator<Item = (&NetId, &Net)> {
        self.nets.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, WireId};
    use crate::model::wiring::{Endpoint, WireSegment};

    #[test]
    fn test_empty_circuit_netlist() {
        let circuit = CircuitDefinition::new(CircuitId::new(1), "Empty");
        let netlist = Netlist::from_circuit(&circuit);

        assert_eq!(netlist.len(), 0);
        assert!(netlist.is_empty());
    }

    #[test]
    fn test_half_adder_fixture_netlist() {
        // Muat fixture sirkuit half-adder dari berkas JSON
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
        let netlist = Netlist::from_circuit(circuit);

        // Pada Half Adder:
        // Ada 4 net fungsional:
        // - Net A: input Port(2) -> XOR1(6:7) & AND1(10:11) via Junction(14) (Root: Net(2))
        // - Net B: input Port(3) -> XOR1(6:8) & AND1(10:12) via Junction(15) (Root: Net(3))
        // - Net Sum: XOR1(6:9) -> output Port(4) (Root: Net(4))
        // - Net Carry: AND1(10:13) -> output Port(5) (Root: Net(5))
        assert_eq!(netlist.len(), 4);

        // Periksa Net A (NetId 2)
        let net_a = netlist.get_net(NetId::new(2)).expect("Net A must exist");
        assert_eq!(net_a.id, NetId::new(2));
        assert_eq!(net_a.drivers.len(), 1);
        assert_eq!(net_a.drivers[0], NetTerminal::Port { id: PinId::new(2) });
        assert_eq!(net_a.loads.len(), 2);
        assert!(net_a.loads.contains(&NetTerminal::Pin {
            instance_id: InstanceId::new(6),
            pin_id: PinId::new(7),
        }));
        assert!(net_a.loads.contains(&NetTerminal::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(11),
        }));
        assert!(!net_a.is_multi_driver());
        assert!(!net_a.is_undriven());

        // Periksa Net B (NetId 3)
        let net_b = netlist.get_net(NetId::new(3)).expect("Net B must exist");
        assert_eq!(net_b.id, NetId::new(3));
        assert_eq!(net_b.drivers.len(), 1);
        assert_eq!(net_b.drivers[0], NetTerminal::Port { id: PinId::new(3) });
        assert_eq!(net_b.loads.len(), 2);

        // Periksa Net Sum (NetId 4)
        let net_sum = netlist.get_net(NetId::new(4)).expect("Net Sum must exist");
        assert_eq!(net_sum.id, NetId::new(4));
        assert_eq!(net_sum.drivers.len(), 1);
        assert_eq!(
            net_sum.drivers[0],
            NetTerminal::Pin {
                instance_id: InstanceId::new(6),
                pin_id: PinId::new(9),
            }
        );
        assert_eq!(net_sum.loads.len(), 1);
        assert_eq!(net_sum.loads[0], NetTerminal::Port { id: PinId::new(4) });

        // Periksa Net Carry (NetId 5)
        let net_carry = netlist
            .get_net(NetId::new(5))
            .expect("Net Carry must exist");
        assert_eq!(net_carry.id, NetId::new(5));
        assert_eq!(net_carry.drivers.len(), 1);
        assert_eq!(
            net_carry.drivers[0],
            NetTerminal::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(13),
            }
        );
        assert_eq!(net_carry.loads.len(), 1);
        assert_eq!(net_carry.loads[0], NetTerminal::Port { id: PinId::new(5) });

        // Periksa lookup helpers
        assert_eq!(netlist.net_for_port(PinId::new(2)), Some(NetId::new(2)));
        assert_eq!(
            netlist.net_for_pin(InstanceId::new(6), PinId::new(9)),
            Some(NetId::new(4))
        );
    }

    #[test]
    fn test_multi_driver_and_undriven_detection() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Conflict Test");
        let registry = ComponentRegistry::with_builtins();

        // 2 buah gerbang NOT
        let not_def = registry.get("dlf.builtin.not").expect("not definition");
        let comp1 = not_def
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)], // 11: input, 12: output
                Point::new(0.0, 0.0),
            )
            .unwrap();
        let comp2 = not_def
            .instantiate(
                InstanceId::new(20),
                &[PinId::new(21), PinId::new(22)], // 21: input, 22: output
                Point::new(100.0, 0.0),
            )
            .unwrap();

        circuit.add_component(comp1);
        circuit.add_component(comp2);

        // Kawat menghubungkan output comp1 (12) langsung ke output comp2 (22) -> Multi-driver!
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(12),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(22),
            },
        ));

        let netlist = Netlist::from_circuit(&circuit);

        // Net gabungan output comp1 & comp2 (NetId min: 12)
        let conflict_net = netlist.get_net(NetId::new(12)).expect("conflict net");
        assert!(conflict_net.is_multi_driver());
        assert_eq!(conflict_net.drivers.len(), 2);
        assert_eq!(conflict_net.loads.len(), 0);

        // Input pin 11 dan 21 berdiri sendiri tanpa driver -> undriven!
        let undriven_net1 = netlist.get_net(NetId::new(11)).expect("undriven net 1");
        assert!(undriven_net1.is_undriven());
        assert_eq!(undriven_net1.drivers.len(), 0);
        assert_eq!(undriven_net1.loads.len(), 1);

        let undriven_net2 = netlist.get_net(NetId::new(21)).expect("undriven net 2");
        assert!(undriven_net2.is_undriven());
        assert_eq!(undriven_net2.drivers.len(), 0);
        assert_eq!(undriven_net2.loads.len(), 1);
    }
}
