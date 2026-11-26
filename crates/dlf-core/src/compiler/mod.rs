//! # Kompilator Sirkuit & IR Lowering Engine
//!
//! Modul ini mengubah dokumen sirkuit, `Netlist` kanonis, dan `EvaluationSchedule`
//! menjadi representasi terkompilasi datar (`CompiledArtifact`) yang siap dijalankan
//! oleh runtime simulasi.

pub mod allocator;

pub use allocator::{AllocatedLayout, SignalSlotAllocator};
