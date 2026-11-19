//! # Definisi Sirkuit dan Port Antarmuka (Circuit Definition & Interface Ports)
//!
//! Modul ini mendefinisikan wadah sirkuit (`CircuitDefinition`) dan port antarmuka luar
//! (`PortDefinition`) sesuai Bab 3.10.1 dan Tabel 3.21 proposal.

use super::component::ComponentInstance;
use super::geometry::Point;
use super::id::{CircuitId, InstanceId, JunctionId, PinId, TunnelId, WireId};
use super::pin::{PinDirection, SignalWidth};
use super::wiring::{Junction, Tunnel, WireSegment};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Port antarmuka batas sirkuit (terminal koneksi untuk hierarki sub-sirkuit chip).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortDefinition {
    /// Stable ID 64-bit unik pin/port antarmuka ini.
    pub id: PinId,
    /// Nama pengenal port (misal `"a"`, `"b"`, `"sum"`, `"carry"`).
    pub name: String,
    /// Arah aliran sinyal pada batas sirkuit.
    pub direction: PinDirection,
    /// Lebar bus sinyal port.
    pub width: SignalWidth,
    /// Posisi visual port pada lembar sirkuit.
    pub position: Point,
}

impl PortDefinition {
    /// Membuat port antarmuka baru.
    pub fn new(
        id: PinId,
        name: impl Into<String>,
        direction: PinDirection,
        width: SignalWidth,
        position: Point,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            direction,
            width,
            position,
        }
    }
}

impl Display for PortDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Port({}, name='{}', dir={}, width={})",
            self.id, self.name, self.direction, self.width
        )
    }
}

/// Definisi lengkap satu lembar sirkuit logika (*CircuitDefinition*).
///
/// Lembar sirkuit dapat bertindak sebagai rangkaian utama (*main circuit*) maupun
/// sebagai sub-sirkuit (*chip*) hierarkis yang dapat diinstansiasi di sirkuit lain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitDefinition {
    /// Stable ID 64-bit unik dari definisi sirkuit ini.
    pub id: CircuitId,
    /// Nama sirkuit (misal `"main"`, `"Half Adder"`, `"ALU 8-bit"`).
    pub name: String,
    /// Daftar port antarmuka eksternal sirkuit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortDefinition>,
    /// Daftar instance komponen yang diletakkan pada lembar sirkuit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<ComponentInstance>,
    /// Daftar segmen kawat penghubung visual.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wires: Vec<WireSegment>,
    /// Daftar titik sambung cabang kawat.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub junctions: Vec<Junction>,
    /// Daftar koneksi nirkabel berbasis label (*tunnel*).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tunnels: Vec<Tunnel>,
}

impl CircuitDefinition {
    /// Membuat definisi sirkuit baru yang kosong.
    pub fn new(id: CircuitId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            ports: Vec::new(),
            components: Vec::new(),
            wires: Vec::new(),
            junctions: Vec::new(),
            tunnels: Vec::new(),
        }
    }

    /// Menambahkan port antarmuka ke sirkuit.
    pub fn add_port(&mut self, port: PortDefinition) {
        self.ports.push(port);
    }

    /// Menambahkan komponen instance ke sirkuit.
    pub fn add_component(&mut self, component: ComponentInstance) {
        self.components.push(component);
    }

    /// Menambahkan segmen kawat ke sirkuit.
    pub fn add_wire(&mut self, wire: WireSegment) {
        self.wires.push(wire);
    }

    /// Menambahkan junction ke sirkuit.
    pub fn add_junction(&mut self, junction: Junction) {
        self.junctions.push(junction);
    }

    /// Menambahkan tunnel ke sirkuit.
    pub fn add_tunnel(&mut self, tunnel: Tunnel) {
        self.tunnels.push(tunnel);
    }

    /// Mencari port antarmuka berdasarkan `PinId`.
    pub fn find_port(&self, port_id: PinId) -> Option<&PortDefinition> {
        self.ports.iter().find(|p| p.id == port_id)
    }

    /// Mencari instance komponen berdasarkan `InstanceId`.
    pub fn find_component(&self, instance_id: InstanceId) -> Option<&ComponentInstance> {
        self.components.iter().find(|c| c.id == instance_id)
    }

    /// Mencari segmen kawat berdasarkan `WireId`.
    pub fn find_wire(&self, wire_id: WireId) -> Option<&WireSegment> {
        self.wires.iter().find(|w| w.id == wire_id)
    }

    /// Mencari junction berdasarkan `JunctionId`.
    pub fn find_junction(&self, junction_id: JunctionId) -> Option<&Junction> {
        self.junctions.iter().find(|j| j.id == junction_id)
    }

    /// Mencari tunnel berdasarkan `TunnelId`.
    pub fn find_tunnel(&self, tunnel_id: TunnelId) -> Option<&Tunnel> {
        self.tunnels.iter().find(|t| t.id == tunnel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_definition_crud() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "SubCircuit");

        circuit.add_port(PortDefinition::new(
            PinId::new(2),
            "in_a",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(10.0, 20.0),
        ));

        assert_eq!(circuit.ports.len(), 1);
        let port = circuit.find_port(PinId::new(2)).expect("Port harus ada");
        assert_eq!(port.name, "in_a");
        assert_eq!(port.direction, PinDirection::Input);
    }
}
