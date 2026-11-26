//! # Representasi Menengah (*Intermediate Representation* / IR)
//!
//! Modul ini mendefinisikan struktur instruksi terkompilasi (`Opcode`, `CompiledOp`),
//! artefak kompilasi (`CompiledArtifact`), serta peta simbolik (`SignalBinding`, `SourceMap`)
//! sebagai jembatan antara frontend konektivitas/analisis (M2) dan eksekusi runtime (M3).

pub mod artifact;
pub mod instruction;

pub use artifact::{CompiledArtifact, SignalBinding, SourceMap};
pub use instruction::{CompiledOp, Opcode, op_flags, width_mask};
