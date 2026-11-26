//! # Kompilator Sirkuit & IR Lowering Engine
//!
//! Modul ini mengubah dokumen sirkuit, `Netlist` kanonis, dan `EvaluationSchedule`
//! menjadi representasi terkompilasi datar (`CompiledArtifact`) yang siap dijalankan
//! oleh runtime simulasi.

pub mod allocator;
pub mod lowering;
pub mod pipeline;

pub use allocator::{AllocatedLayout, SignalSlotAllocator};
pub use lowering::lower_to_ir;
pub use pipeline::{CircuitCompiler, CircuitSnapshot, CompilationError, CompilationResult};
