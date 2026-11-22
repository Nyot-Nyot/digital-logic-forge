//! # Resolusi Konektivitas dan Topologi Sirkuit (Circuit Connectivity)
//!
//! Modul ini bertanggung jawab atas pembangunan netlist kanonis dari representasi dokumen grafis,
//! resolusi percabangan junction, penyatuan tunnel nirkabel, serta pengelompokan partisi kawat
//! menggunakan algoritma *Disjoint Set Union* (DSU) sesuai Bab 3.4.2 dan Bab 3.10.11 proposal.

pub mod dsu;
pub mod netlist;
pub mod node;
pub mod resolver;
pub mod tunnel;

pub use netlist::{Net, NetId, NetTerminal, Netlist};
pub use resolver::{resolve_circuit_connectivity, resolve_circuit_connectivity_with_report};
pub use tunnel::resolve_circuit_tunnels;
