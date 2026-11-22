//! # Geometri Kawat, Titik Sambung, dan Tunnel (Wiring, Junctions, & Tunnels)
//!
//! Modul ini memodelkan entitas grafis kawat penghubung (`WireSegment`), titik sambung topologis
//! cabang (`Junction`), serta label nirkabel (`Tunnel`) sesuai Bab 3.10.1 dan Tabel 3.21 proposal.
//!
//! Entitas di modul ini murni merepresentasikan objek visual dan keterikatan endpoint editor.
//! Topologi sirkuit listrik (*net*) baru dikonstruksi saat proses kompilasi pada Milestone M2.

use super::geometry::Point;
use super::id::{InstanceId, JunctionId, PinId, TunnelId, WireId};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Representasi titik ujung (*endpoint*) dari sebuah segmen kawat.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Endpoint {
    /// Terhubung langsung ke terminal pin suatu instance komponen.
    Pin {
        /// ID instance komponen pemilik pin.
        instance_id: InstanceId,
        /// ID pin konkret yang dituju.
        pin_id: PinId,
    },
    /// Terhubung ke port antarmuka luar rangkaian (*circuit boundary port*).
    Port {
        /// ID pin/port rangkaian.
        id: PinId,
    },
    /// Terhubung ke titik cabang sambungan kawat (*junction*).
    Junction {
        /// ID titik junction.
        id: JunctionId,
    },
    /// Menggantung bebas pada koordinat canvas tertentu (tidak tersambung).
    Free {
        /// Posisi koordinat terminal bebas.
        position: Point,
    },
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pin {
                instance_id,
                pin_id,
            } => write!(f, "Pin({}:{})", instance_id, pin_id),
            Self::Port { id } => write!(f, "Port({})", id),
            Self::Junction { id } => write!(f, "Junction({})", id),
            Self::Free { position } => write!(f, "Free({})", position),
        }
    }
}

/// Segmen kawat penghubung visual antara dua endpoint dengan kemungkinan titik belok (*waypoints*).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireSegment {
    /// Stable ID 64-bit unik kawat ini.
    pub id: WireId,
    /// Titik awal segmen kawat.
    pub start: Endpoint,
    /// Titik akhir segmen kawat.
    pub end: Endpoint,
    /// Daftar titik belok ortogonal di antara start dan end (jika ada).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<Point>,
}

impl WireSegment {
    /// Membuat segmen kawat baru antara dua endpoint.
    pub fn new(id: WireId, start: Endpoint, end: Endpoint) -> Self {
        Self {
            id,
            start,
            end,
            waypoints: Vec::new(),
        }
    }

    /// Membuat segmen kawat dengan titik belok ortogonal (*waypoints*).
    pub fn with_waypoints(
        id: WireId,
        start: Endpoint,
        end: Endpoint,
        waypoints: Vec<Point>,
    ) -> Self {
        Self {
            id,
            start,
            end,
            waypoints,
        }
    }

    /// Memeriksa apakah segmen kawat ini memiliki endpoint yang menggantung bebas (`Endpoint::Free`).
    pub fn is_dangling(&self) -> bool {
        matches!(self.start, Endpoint::Free { .. }) || matches!(self.end, Endpoint::Free { .. })
    }
}

/// Titik sambung cabang topologis antara 3 atau lebih kawat yang bertemu pada koordinat yang sama.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Junction {
    /// Stable ID 64-bit unik junction ini.
    pub id: JunctionId,
    /// Posisi koordinat visual titik sambung.
    pub position: Point,
}

impl Junction {
    /// Membuat junction baru pada posisi tertentu.
    pub const fn new(id: JunctionId, position: Point) -> Self {
        Self { id, position }
    }
}

/// Cakupan visibilitas koneksi nirkabel untuk entitas `Tunnel`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum TunnelScope {
    /// Hanya menghubungkan tunnel bernama sama di dalam sirkuit yang sama (*circuit-local*).
    #[default]
    Local,
    /// Menghubungkan tunnel melintasi batas hierarki proyek (*global*).
    Global,
}

impl Display for TunnelScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Global => write!(f, "Global"),
        }
    }
}

/// Entitas penghubung sinyal nirkabel berbasis nama label (*tunnel*).
///
/// Dua atau lebih tunnel dengan nama yang sama dan cakupan (*scope*) yang sama
/// akan digabungkan ke dalam satu net listrik yang sama saat kompilasi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tunnel {
    /// Stable ID 64-bit unik tunnel ini.
    pub id: TunnelId,
    /// Nama label identitas tunnel (case-sensitive).
    pub name: String,
    /// Cakupan visibilitas tunnel.
    pub scope: TunnelScope,
    /// Posisi visual label di canvas.
    pub position: Point,
    /// ID pin terminal yang terpasang pada tunnel ini.
    pub pin_id: PinId,
}

impl Tunnel {
    /// Membuat instance tunnel baru.
    pub fn new(
        id: TunnelId,
        name: impl Into<String>,
        scope: TunnelScope,
        position: Point,
        pin_id: PinId,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            scope,
            position,
            pin_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_segment_creation_and_dangling() {
        let wire = WireSegment::new(
            WireId::new(1),
            Endpoint::Port { id: PinId::new(2) },
            Endpoint::Junction {
                id: JunctionId::new(3),
            },
        );

        assert_eq!(wire.id.get(), 1);
        assert!(!wire.is_dangling());

        let dangling_wire = WireSegment::new(
            WireId::new(2),
            Endpoint::Free {
                position: Point::new(10.0, 20.0),
            },
            Endpoint::Port { id: PinId::new(4) },
        );

        assert!(dangling_wire.is_dangling());
    }

    #[test]
    fn test_wire_with_waypoints() {
        let wire = WireSegment::with_waypoints(
            WireId::new(10),
            Endpoint::Pin {
                instance_id: InstanceId::new(1),
                pin_id: PinId::new(2),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(3),
                pin_id: PinId::new(4),
            },
            vec![Point::new(50.0, 100.0), Point::new(150.0, 100.0)],
        );

        assert_eq!(wire.waypoints.len(), 2);
    }

    #[test]
    fn test_junction_and_tunnel_creation() {
        let j = Junction::new(JunctionId::new(5), Point::new(30.0, 40.0));
        assert_eq!(j.id.get(), 5);
        assert_eq!(j.position.x, 30.0);

        let t = Tunnel::new(
            TunnelId::new(6),
            "CLK_BUS",
            TunnelScope::Local,
            Point::new(100.0, 200.0),
            PinId::new(7),
        );
        assert_eq!(t.name, "CLK_BUS");
        assert_eq!(t.scope, TunnelScope::Local);
        assert_eq!(t.pin_id.get(), 7);
    }

    #[test]
    fn test_serde_wire_roundtrip() {
        let wire = WireSegment::with_waypoints(
            WireId::new(101),
            Endpoint::Port { id: PinId::new(2) },
            Endpoint::Junction {
                id: JunctionId::new(14),
            },
            vec![Point::new(120.0, 100.0)],
        );

        let json = serde_json::to_string(&wire).expect("Gagal serialize WireSegment");
        let decoded: WireSegment =
            serde_json::from_str(&json).expect("Gagal deserialize WireSegment");

        assert_eq!(wire, decoded);
    }
}
