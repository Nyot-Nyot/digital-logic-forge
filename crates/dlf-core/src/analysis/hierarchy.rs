//! # Elaborasi Hierarki Sub-Sirkuit Dasar (Hierarchy Elaboration)
//!
//! Modul ini mengimplementasikan ekspansi dan validasi hierarki sub-sirkuit kustom,
//! pemetaan port-to-pin antarmuka, serta pencegahan rekursi instansiasi tak berhingga
//! (`HIER-RECURSION`) sesuai Bab 3.4.1 dan Tabel 3.10 & 3.31 proposal.

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticStage, SourceRef};
use crate::error::CoreError;
use crate::model::id::{CircuitId, InstanceId, PinId};
use crate::model::pin::{PinDirection, SignalWidth};
use crate::model::project::Project;
use crate::validation::ValidationReport;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Pemetaan antara port batas antarmuka sub-sirkuit anak ke pin instance di sirkuit induk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortBinding {
    /// ID port pada sirkuit anak.
    pub port_id: PinId,
    /// Nama pengenal port pada sirkuit anak.
    pub port_name: String,
    /// Arah aliran sinyal pada port sirkuit anak.
    pub direction: PinDirection,
    /// Lebar bus sinyal.
    pub width: SignalWidth,
    /// ID pin konkret pada instance komponen di sirkuit induk.
    pub instance_pin_id: PinId,
}

/// Simpul pohon hierarki instansiasi rangkaian (*Hierarchy Instance Node*).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInstanceNode {
    /// Jalur hierarki kanonis stabil (misal: `"root"` atau `"root/alu0/adder0"`).
    pub path: String,
    /// ID sirkuit yang diinstansiasi.
    pub circuit_id: CircuitId,
    /// ID instance pada sirkuit induk (`None` untuk root circuit).
    pub instance_id: Option<InstanceId>,
    /// Pemetaan port anak ke pin instance induk.
    pub bindings: Vec<PortBinding>,
    /// Sub-instance anak yang diinstansiasi di dalam sirkuit ini.
    pub children: Vec<CircuitInstanceNode>,
}

impl CircuitInstanceNode {
    /// Membuat simpul hierarki baru.
    pub fn new(
        path: impl Into<String>,
        circuit_id: CircuitId,
        instance_id: Option<InstanceId>,
    ) -> Self {
        Self {
            path: path.into(),
            circuit_id,
            instance_id,
            bindings: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Menghitung jumlah total instance (termasuk seluruh turunan hierarki).
    pub fn total_instance_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(CircuitInstanceNode::total_instance_count)
            .sum::<usize>()
    }
}

/// Mencoba mencari ID sirkuit yang dirujuk oleh string `definition_id`.
///
/// Mendukung konvensi:
/// - `"dlf.circuit.<id>"` atau `"circuit:<id>"`
/// - Pencocokan langsung dengan nama sirkuit di dalam `Project`.
pub fn resolve_referenced_circuit_id(definition_id: &str, project: &Project) -> Option<CircuitId> {
    let parsed_id = definition_id
        .strip_prefix("dlf.circuit.")
        .or_else(|| definition_id.strip_prefix("circuit:"))
        .and_then(|id_str| id_str.parse::<u64>().ok())
        .map(CircuitId::new);

    if let Some(cid) = parsed_id.filter(|&cid| project.get_circuit(cid).is_some()) {
        return Some(cid);
    }

    for circuit in &project.circuits {
        if circuit.name == definition_id {
            return Some(circuit.id);
        }
    }

    None
}

/// Memeriksa apakah terdapat siklus instansiasi rekursif pada proyek rangkaian (`HIER-RECURSION`).
///
/// Jika terdeteksi siklus pemanggilan (misal Sirkuit A memuat Sirkuit B dan Sirkuit B memuat Sirkuit A),
/// fungsi ini akan menambahkan error diagnostik `DiagnosticCode::HierRecursion` ke dalam `report`.
pub fn validate_hierarchy_recursion(project: &Project, report: &mut ValidationReport) {
    // Bangun graf pemanggilan antar sirkuit: CircuitId -> Vec<(CircuitId, InstanceId)>
    let mut call_graph: BTreeMap<CircuitId, Vec<(CircuitId, InstanceId)>> = BTreeMap::new();

    for circuit in &project.circuits {
        let calls = call_graph.entry(circuit.id).or_default();
        for comp in &circuit.components {
            if let Some(target_cid) = resolve_referenced_circuit_id(&comp.definition_id, project) {
                calls.push((target_cid, comp.id));
            }
        }
    }

    // Penelusuran siklus dengan DFS berbasis recursion stack (3-color DFS)
    let mut visited: BTreeSet<CircuitId> = BTreeSet::new();
    let mut rec_stack: BTreeSet<CircuitId> = BTreeSet::new();

    for circuit in &project.circuits {
        if !visited.contains(&circuit.id) {
            check_recursion_dfs(
                circuit.id,
                &call_graph,
                &mut visited,
                &mut rec_stack,
                report,
            );
        }
    }
}

fn check_recursion_dfs(
    curr: CircuitId,
    call_graph: &BTreeMap<CircuitId, Vec<(CircuitId, InstanceId)>>,
    visited: &mut BTreeSet<CircuitId>,
    rec_stack: &mut BTreeSet<CircuitId>,
    report: &mut ValidationReport,
) {
    visited.insert(curr);
    rec_stack.insert(curr);

    if let Some(targets) = call_graph.get(&curr) {
        for &(target_cid, instance_id) in targets {
            if rec_stack.contains(&target_cid) {
                let message = format!(
                    "Terdeteksi siklus rekursi instansiasi sub-sirkuit tak berhingga (HIER-RECURSION): Circuit({}) memanggil Circuit({})",
                    curr, target_cid
                );

                let diagnostic = Diagnostic::error(
                    DiagnosticCode::HierRecursion,
                    message,
                    DiagnosticStage::Elaboration,
                    SourceRef::Instance { id: instance_id },
                );

                report.add(diagnostic);
            } else if !visited.contains(&target_cid) {
                check_recursion_dfs(target_cid, call_graph, visited, rec_stack, report);
            }
        }
    }

    rec_stack.remove(&curr);
}

/// Melakukan elaborasi hierarki pohon instance dari sirkuit akar (*root circuit*).
///
/// # Errors
/// Mengembalikan `CoreError::ValidationError` jika terdeteksi siklus rekursi instansiasi
/// atau jika sirkuit akar tidak ditemukan.
pub fn elaborate_hierarchy(project: &Project) -> Result<CircuitInstanceNode, CoreError> {
    let mut report = ValidationReport::new();
    validate_hierarchy_recursion(project, &mut report);

    if report.has_errors() {
        return Err(CoreError::ValidationError(format!(
            "Elaborasi hierarki dibatalkan karena terdeteksi {} kesalahan rekursi instansiasi",
            report.error_count()
        )));
    }

    let root_circuit = project
        .get_circuit(project.root_circuit_id)
        .ok_or_else(|| {
            CoreError::ValidationError(format!(
                "Sirkuit akar dengan ID {} tidak ditemukan dalam proyek",
                project.root_circuit_id
            ))
        })?;

    elaborate_node_recursive("root", root_circuit.id, None, project)
}

fn elaborate_node_recursive(
    current_path: &str,
    circuit_id: CircuitId,
    instance_id: Option<InstanceId>,
    project: &Project,
) -> Result<CircuitInstanceNode, CoreError> {
    let circuit = project.get_circuit(circuit_id).ok_or_else(|| {
        CoreError::ValidationError(format!("Sirkuit ID {} tidak ditemukan", circuit_id))
    })?;

    let mut node = CircuitInstanceNode::new(current_path, circuit_id, instance_id);

    for comp in &circuit.components {
        if let Some(child_cid) = resolve_referenced_circuit_id(&comp.definition_id, project) {
            let child_circuit = project.get_circuit(child_cid).ok_or_else(|| {
                CoreError::ValidationError(format!("Sirkuit anak ID {} tidak ditemukan", child_cid))
            })?;

            let label_str = comp.label.as_deref().unwrap_or("subcircuit");
            let child_path = format!("{}/{}", current_path, label_str);
            let mut child_node =
                elaborate_node_recursive(&child_path, child_cid, Some(comp.id), project)?;

            // Bentuk port-to-pin binding
            for port in &child_circuit.ports {
                // Cari pin pada comp yang deklarasinya cocok dengan nama atau ID port
                if let Some(pin) = comp.pins.iter().find(|p| {
                    p.declaration_id == port.name || p.declaration_id == port.id.to_string()
                }) {
                    child_node.bindings.push(PortBinding {
                        port_id: port.id,
                        port_name: port.name.clone(),
                        direction: port.direction,
                        width: port.width,
                        instance_pin_id: pin.id,
                    });
                }
            }

            node.children.push(child_node);
        }
    }

    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::{CircuitDefinition, PortDefinition};
    use crate::model::component::ComponentInstance;
    use crate::model::geometry::{Point, Rotation};
    use crate::model::pin::PinInstance;

    #[test]
    fn test_valid_subcircuit_elaboration() {
        let mut project = Project::new("Hierarchy Test");
        let root_id = project.root_circuit_id;

        // Sirkuit anak (SubCircuit)
        let child_cid = project.allocate_id().into();
        let mut child_circuit = CircuitDefinition::new(child_cid, "SubCircuit");
        let port_in_id = project.allocate_id().into();
        let port_out_id = project.allocate_id().into();

        child_circuit.add_port(PortDefinition::new(
            port_in_id,
            "in_a",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(0.0, 0.0),
        ));
        child_circuit.add_port(PortDefinition::new(
            port_out_id,
            "out_y",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::new(100.0, 0.0),
        ));

        // Instance di sirkuit induk (root)
        let comp_inst_id = project.allocate_id().into();
        let pin1_id = project.allocate_id().into();
        let pin2_id = project.allocate_id().into();

        let comp = ComponentInstance {
            id: comp_inst_id,
            definition_id: format!("dlf.circuit.{}", child_cid),
            schema_version: 1,
            label: Some("chip1".to_string()),
            position: Point::new(50.0, 50.0),
            rotation: Rotation::R0,
            mirror_x: false,
            mirror_y: false,
            pins: vec![
                PinInstance::new(
                    pin1_id,
                    "in_a",
                    PinDirection::Input,
                    SignalWidth::SINGLE,
                    crate::model::pin::PinRole::Signal,
                ),
                PinInstance::new(
                    pin2_id,
                    "out_y",
                    PinDirection::Output,
                    SignalWidth::SINGLE,
                    crate::model::pin::PinRole::Signal,
                ),
            ],
            parameters: BTreeMap::new(),
        };

        let root_circuit = project.get_circuit_mut(root_id).expect("root circuit");
        root_circuit.add_component(comp);
        project.circuits.push(child_circuit);

        let hierarchy = elaborate_hierarchy(&project).expect("elaboration succeeds");
        assert_eq!(hierarchy.path, "root");
        assert_eq!(hierarchy.total_instance_count(), 2);
        assert_eq!(hierarchy.children.len(), 1);

        let child = &hierarchy.children[0];
        assert_eq!(child.path, "root/chip1");
        assert_eq!(child.circuit_id, child_cid);
        assert_eq!(child.instance_id, Some(comp_inst_id));
        assert_eq!(child.bindings.len(), 2);
    }

    #[test]
    fn test_direct_recursive_instantiation_detected() {
        let mut project = Project::new("Direct Recursion Test");
        let root_id = project.root_circuit_id;

        // Sirkuit akar menginstansiasi dirinya sendiri
        let comp = ComponentInstance {
            id: project.allocate_id().into(),
            definition_id: format!("dlf.circuit.{}", root_id),
            schema_version: 1,
            label: Some("self_ref".to_string()),
            position: Point::new(0.0, 0.0),
            rotation: Rotation::R0,
            mirror_x: false,
            mirror_y: false,
            pins: Vec::new(),
            parameters: BTreeMap::new(),
        };

        let root = project.get_circuit_mut(root_id).unwrap();
        root.add_component(comp);

        let mut report = ValidationReport::new();
        validate_hierarchy_recursion(&project, &mut report);

        assert_eq!(report.error_count(), 1);
        assert_eq!(report.diagnostics[0].code, DiagnosticCode::HierRecursion);

        let res = elaborate_hierarchy(&project);
        assert!(res.is_err());
    }

    #[test]
    fn test_mutual_recursive_instantiation_detected() {
        let mut project = Project::new("Mutual Recursion Test");
        let c1_id = project.root_circuit_id;
        let c2_id: CircuitId = project.allocate_id().into();

        let mut c2 = CircuitDefinition::new(c2_id, "Circuit2");

        // C1 instansiasi C2
        let comp1 = ComponentInstance {
            id: project.allocate_id().into(),
            definition_id: format!("dlf.circuit.{}", c2_id),
            schema_version: 1,
            label: Some("to_c2".to_string()),
            position: Point::new(0.0, 0.0),
            rotation: Rotation::R0,
            mirror_x: false,
            mirror_y: false,
            pins: Vec::new(),
            parameters: BTreeMap::new(),
        };
        project.get_circuit_mut(c1_id).unwrap().add_component(comp1);

        // C2 instansiasi C1 -> membentuk siklus mutual recursion!
        let comp2 = ComponentInstance {
            id: project.allocate_id().into(),
            definition_id: format!("dlf.circuit.{}", c1_id),
            schema_version: 1,
            label: Some("to_c1".to_string()),
            position: Point::new(0.0, 0.0),
            rotation: Rotation::R0,
            mirror_x: false,
            mirror_y: false,
            pins: Vec::new(),
            parameters: BTreeMap::new(),
        };
        c2.add_component(comp2);
        project.circuits.push(c2);

        let mut report = ValidationReport::new();
        validate_hierarchy_recursion(&project, &mut report);

        assert_eq!(report.error_count(), 1);
        assert_eq!(report.diagnostics[0].code, DiagnosticCode::HierRecursion);
    }
}
