//! # Analisis Semantik dan Graf Ketergantungan (Semantic Analysis & Dependency Graph)
//!
//! Modul ini bertanggung jawab atas pembentukan graf ketergantungan kombinasional,
//! deteksi siklus umpan balik via algoritma Tarjan SCC, serta pengurutan level topologis
//! (*topological levelization*) sesuai Bab 3.4.1 dan Bab 3.10.11 proposal.

pub mod graph;

pub use graph::{DependencyGraph, GraphNode};
