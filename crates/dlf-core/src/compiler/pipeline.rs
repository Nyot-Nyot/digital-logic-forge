//! # Pipeline Snapshot dan Kompilasi Sirkuit
//!
//! Modul ini menyatukan alur `Authoring Document → CircuitSnapshot → Validasi M2 → IR Lowering`
//! menjadi satu *entry point* deterministik `CircuitCompiler::compile()`.

use super::lowering::lower_to_ir;
use crate::analysis::{
    DependencyGraph, EvaluationSchedule, analyze_scc, levelize_dependency_graph,
};
use crate::connectivity::{Netlist, resolve_circuit_connectivity, resolve_circuit_tunnels};
use crate::error::CoreError;
use crate::ir::CompiledArtifact;
use crate::model::circuit::CircuitDefinition;
use crate::model::component::ComponentRegistry;
use crate::validation::{ValidationReport, validate_circuit, validate_netlist_electrical};
use std::fmt::{self, Display, Formatter};

/// Representasi *snapshot* immutable dari suatu `CircuitDefinition` pada titik kompilasi.
///
/// Memastikan editor dapat terus memodifikasi dokumen asli secara independen tanpa
/// mengganggu data yang sedang dikompilasi atau dijalankan oleh simulasi.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitSnapshot {
    /// Salinan beku definisi rangkaian pada saat snapshot dibuat.
    pub circuit: CircuitDefinition,
}

impl CircuitSnapshot {
    /// Membekukan `CircuitDefinition` menjadi *snapshot* immutable.
    pub fn freeze(circuit: &CircuitDefinition) -> Self {
        Self {
            circuit: circuit.clone(),
        }
    }
}

/// Hasil kompilasi lengkap apabila seluruh tahap validasi dan lowering berhasil.
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Snapshot immutable sumber kompilasi.
    pub snapshot: CircuitSnapshot,
    /// Netlist elektris kanonis hasil resolusi DSU.
    pub netlist: Netlist,
    /// Jadwal evaluasi berurutan level.
    pub schedule: EvaluationSchedule,
    /// Artefak IR terkompilasi siap eksekusi oleh runtime B1.
    pub artifact: CompiledArtifact,
    /// Laporan diagnostik non-fatal (peringatan/*warning*) jika ada.
    pub warnings: ValidationReport,
}

/// Kesalahan kompilasi terstruktur yang menyimpan seluruh diagnostik penyebab kegagalan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationError {
    /// Laporan validasi berisi error dan warning yang ditemukan selama kompilasi.
    pub report: ValidationReport,
}

impl CompilationError {
    /// Membuat `CompilationError` dari `ValidationReport`.
    pub fn from_report(report: ValidationReport) -> Self {
        Self { report }
    }
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Kompilasi gagal dengan {} error dan {} warning",
            self.report.error_count(),
            self.report.warning_count()
        )
    }
}

impl std::error::Error for CompilationError {}

impl From<CompilationError> for CoreError {
    fn from(err: CompilationError) -> Self {
        CoreError::ValidationError(err.to_string())
    }
}

/// Kompilator utama yang menerjemahkan dokumen rangkaian menjadi `CompiledArtifact`.
pub struct CircuitCompiler;

impl CircuitCompiler {
    /// Membekukan `circuit` menjadi `CircuitSnapshot`, memvalidasi struktur, konektivitas,
    /// aturan elektris, dan keasiklikan graf, lalu menurunkannya menjadi `CompiledArtifact`.
    ///
    /// # Errors
    /// Mengembalikan `CompilationError` apabila ditemukan error struktural, konflik multi-driver,
    /// net tanpa penggerak, siklus kombinasional (SCC), atau kegagalan lowering IR.
    pub fn compile(
        circuit: &CircuitDefinition,
        registry: &ComponentRegistry,
    ) -> Result<CompilationResult, CompilationError> {
        let snapshot = CircuitSnapshot::freeze(circuit);
        let mut report = ValidationReport::new();

        // 1. Validasi struktural dokumen (M1)
        let structural_report = validate_circuit(&snapshot.circuit, registry);
        report.merge(structural_report);

        // 2. Resolusi konektivitas kawat, junction, dan tunnel (M2)
        let mut dsu = resolve_circuit_connectivity(&snapshot.circuit);
        resolve_circuit_tunnels(&snapshot.circuit, &mut dsu, &mut report);
        let netlist = Netlist::from_circuit_and_dsu(&snapshot.circuit, &mut dsu);

        // 3. Validasi elektris netlist (M2)
        validate_netlist_electrical(&netlist, &mut report);

        if report.has_errors() {
            return Err(CompilationError::from_report(report));
        }

        // 4. Analisis graf ketergantungan dan deteksi siklus Tarjan SCC (M2)
        let graph =
            DependencyGraph::from_circuit_and_netlist(&snapshot.circuit, &netlist, registry);
        let scc_result = analyze_scc(&graph);
        scc_result.validate_cycles(&mut report);

        if report.has_errors() {
            return Err(CompilationError::from_report(report));
        }

        // 5. Levelization & topological sorting (M2)
        let schedule = levelize_dependency_graph(&graph).map_err(|err| {
            let mut err_report = report.clone();
            err_report.add(crate::diagnostic::Diagnostic::error(
                crate::diagnostic::DiagnosticCode::DepSccOscillation,
                err.to_string(),
                crate::diagnostic::DiagnosticStage::Scheduling,
                crate::diagnostic::SourceRef::Circuit {
                    id: snapshot.circuit.id,
                },
            ));
            CompilationError::from_report(err_report)
        })?;

        // 6. IR Lowering & Validasi Artefak (M3)
        let artifact = lower_to_ir(&snapshot.circuit, &netlist, &schedule).map_err(|err| {
            let mut err_report = report.clone();
            err_report.add(crate::diagnostic::Diagnostic::error(
                crate::diagnostic::DiagnosticCode::NetUndriven,
                err.to_string(),
                crate::diagnostic::DiagnosticStage::Lowering,
                crate::diagnostic::SourceRef::Circuit {
                    id: snapshot.circuit.id,
                },
            ));
            CompilationError::from_report(err_report)
        })?;

        Ok(CompilationResult {
            snapshot,
            netlist,
            schedule,
            artifact,
            warnings: report,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::DiagnosticCode;
    use crate::model::circuit::PortDefinition;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, InstanceId, PinId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, WireSegment};

    #[test]
    fn test_pipeline_compiles_valid_inverter_chain() {
        let registry = ComponentRegistry::with_builtins();
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Inverter");

        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "IN",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));
        circuit.add_port(PortDefinition::new(
            PinId::new(2),
            "OUT",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::new(100.0, 0.0),
        ));

        let not_inst = registry
            .get("dlf.builtin.not")
            .unwrap()
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)],
                Point::new(50.0, 0.0),
            )
            .unwrap();
        circuit.add_component(not_inst);

        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(101),
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(12),
            },
            Endpoint::Port { id: PinId::new(2) },
        ));

        let res = CircuitCompiler::compile(&circuit, &registry).expect("Kompilasi harus sukses");
        assert_eq!(res.artifact.operations.len(), 1);
        assert_eq!(res.artifact.input_bindings.len(), 1);
        assert_eq!(res.artifact.output_bindings.len(), 1);
    }

    #[test]
    fn test_pipeline_rejects_oscillating_loop() {
        let registry = ComponentRegistry::with_builtins();
        let mut circuit = CircuitDefinition::new(CircuitId::new(2), "RingOscillator");

        let not_inst = registry
            .get("dlf.builtin.not")
            .unwrap()
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)],
                Point::ZERO,
            )
            .unwrap();
        circuit.add_component(not_inst);

        // Self-loop: y -> a
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(12),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
        ));

        let err = CircuitCompiler::compile(&circuit, &registry)
            .expect_err("Siklus kombinasional harus ditolak");
        assert!(
            err.report
                .diagnostics
                .iter()
                .any(|d| d.code == DiagnosticCode::DepSccOscillation)
        );
    }
}
