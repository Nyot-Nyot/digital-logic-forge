//! # Runtime Eksekusi Simulasi (Reference B0 & Compiled B1)
//!
//! Modul ini menampung dua evaluator independen:
//! - `ReferenceEvaluator` (B0): evaluator semantik bit-demi-bit sebagai *oracle* kebenaran.
//! - `CompiledRuntime` (B1): evaluator terkompilasi berkinerja tinggi berbasis array kontigu.

pub mod compiled;
pub mod reference;
pub mod value;

pub use compiled::CompiledRuntime;
pub use reference::ReferenceEvaluator;
pub use value::LogicValue;
