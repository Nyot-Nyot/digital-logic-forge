//! # Analisis Komponen Terhubung Kuat (Strongly Connected Components / Tarjan SCC)
//!
//! Modul ini mengimplementasikan algoritma Tarjan untuk mendeteksi *Strongly Connected Components* (SCC)
//! pada graf ketergantungan kombinasional (`DependencyGraph`) sesuai Bab 3.4.1 dan Bab 3.10.11 proposal.
//!
//! SCC digunakan untuk:
//! 1. Memisahkan wilayah rangkaian asiklik murni (DAG) dari wilayah umpan balik (*feedback loop*).
//! 2. Mendeteksi siklus kombinasional ilegal yang berpotensi menyebabkan ketidakstabilan logika
//!    atau osilasi tak berhingga, lalu melaporkannya via diagnostik `DEP-SCC-OSCILLATION`.

use super::graph::{DependencyGraph, GraphNode};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticStage};
use crate::validation::ValidationReport;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Representasi satu komponen terhubung kuat (*Strongly Connected Component* / SCC).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StronglyConnectedComponent {
    /// Indeks urutan SCC.
    pub id: usize,
    /// Daftar simpul anggota SCC yang diurutkan secara deterministik.
    pub members: Vec<GraphNode>,
    /// Menandakan apakah SCC ini membentuk siklus tertutup (*cyclic feedback loop*).
    ///
    /// Bernilai `true` jika komponen memiliki lebih dari 1 simpul, atau 1 simpul dengan *self-loop*.
    pub is_cyclic: bool,
}

impl StronglyConnectedComponent {
    /// Membuat SCC baru.
    pub fn new(id: usize, mut members: Vec<GraphNode>, is_cyclic: bool) -> Self {
        members.sort_unstable();
        Self {
            id,
            members,
            is_cyclic,
        }
    }

    /// Jumlah simpul dalam SCC ini.
    #[inline]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Memeriksa apakah SCC kosong.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Hasil evaluasi dekomposisi SCC pada graf ketergantungan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SccAnalysisResult {
    /// Seluruh SCC yang ditemukan.
    pub sccs: Vec<StronglyConnectedComponent>,
    /// Menandakan apakah seluruh graf merupakan Directed Acyclic Graph (DAG) murni.
    pub is_dag: bool,
    /// Daftar SCC yang membentuk siklus ketergantungan kombinasional.
    pub cyclic_sccs: Vec<StronglyConnectedComponent>,
}

impl SccAnalysisResult {
    /// Memvalidasi siklus kombinasional dan menerbitkan diagnostik `DEP-SCC-OSCILLATION`
    /// untuk setiap komponen yang bersifat siklik.
    pub fn validate_cycles(&self, report: &mut ValidationReport) {
        for scc in &self.cyclic_sccs {
            if scc.members.is_empty() {
                continue;
            }

            let primary = scc.members[0].to_source_ref();
            let related = scc.members[1..]
                .iter()
                .map(GraphNode::to_source_ref)
                .collect();

            let member_names: Vec<String> = scc.members.iter().map(|n| n.to_string()).collect();

            let message = format!(
                "Terdeteksi siklus kombinasional ilegal (feedback loop berosilasi) melibatkan {} simpul: [{}]",
                scc.members.len(),
                member_names.join(", ")
            );

            let diagnostic = Diagnostic::error(
                DiagnosticCode::DepSccOscillation,
                message,
                DiagnosticStage::Scheduling,
                primary,
            )
            .with_related(related);

            report.add(diagnostic);
        }
    }

    /// Jumlah siklus kombinasional yang terdeteksi.
    #[inline]
    pub fn cyclic_count(&self) -> usize {
        self.cyclic_sccs.len()
    }
}

/// Struktur bantu pelacak status DFS algoritma Tarjan.
struct TarjanContext<'a> {
    graph: &'a DependencyGraph,
    index: usize,
    indices: BTreeMap<GraphNode, usize>,
    lowlink: BTreeMap<GraphNode, usize>,
    stack: Vec<GraphNode>,
    on_stack: BTreeSet<GraphNode>,
    sccs: Vec<StronglyConnectedComponent>,
}

impl<'a> TarjanContext<'a> {
    fn new(graph: &'a DependencyGraph) -> Self {
        Self {
            graph,
            index: 0,
            indices: BTreeMap::new(),
            lowlink: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: BTreeSet::new(),
            sccs: Vec::new(),
        }
    }

    fn strongconnect(&mut self, u: GraphNode) {
        self.indices.insert(u, self.index);
        self.lowlink.insert(u, self.index);
        self.index += 1;
        self.stack.push(u);
        self.on_stack.insert(u);

        // Kunjungi seluruh simpul penerus secara deterministik
        if let Some(successors) = self.graph.adj.get(&u) {
            for &v in successors {
                if !self.indices.contains_key(&v) {
                    // Penerus belum dikunjungi
                    self.strongconnect(v);
                    let low_v = self.lowlink[&v];
                    let low_u = self.lowlink.get_mut(&u).expect("lowlink u exists");
                    *low_u = (*low_u).min(low_v);
                } else if self.on_stack.contains(&v) {
                    // Penerus ada di dalam stack (back edge dalam cabang DFS yang sama)
                    let idx_v = self.indices[&v];
                    let low_u = self.lowlink.get_mut(&u).expect("lowlink u exists");
                    *low_u = (*low_u).min(idx_v);
                }
            }
        }

        // Jika u adalah root dari SCC
        if self.lowlink[&u] == self.indices[&u] {
            let mut members = Vec::new();
            while let Some(top) = self.stack.pop() {
                self.on_stack.remove(&top);
                members.push(top);
                if top == u {
                    break;
                }
            }

            let scc_id = self.sccs.len();
            let is_cyclic = members.len() > 1 || self.graph.contains_edge(u, u);
            self.sccs
                .push(StronglyConnectedComponent::new(scc_id, members, is_cyclic));
        }
    }
}

/// Menjalankan algoritma Tarjan pada `DependencyGraph` untuk menemukan seluruh SCC.
pub fn analyze_scc(graph: &DependencyGraph) -> SccAnalysisResult {
    let mut ctx = TarjanContext::new(graph);

    // Iterasi seluruh simpul dalam urutan BTreeSet yang deterministik
    for &node in &graph.nodes {
        if !ctx.indices.contains_key(&node) {
            ctx.strongconnect(node);
        }
    }

    let mut cyclic_sccs = Vec::new();
    for scc in &ctx.sccs {
        if scc.is_cyclic {
            cyclic_sccs.push(scc.clone());
        }
    }

    let is_dag = cyclic_sccs.is_empty();

    SccAnalysisResult {
        sccs: ctx.sccs,
        is_dag,
        cyclic_sccs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::component::ComponentRegistry;
    use crate::model::id::InstanceId;

    #[test]
    fn test_tarjan_acyclic_dag() {
        let mut graph = DependencyGraph::new();
        let a = GraphNode::Combinational(InstanceId::new(1));
        let b = GraphNode::Combinational(InstanceId::new(2));
        let c = GraphNode::Combinational(InstanceId::new(3));

        // A -> B -> C
        graph.add_edge(a, b);
        graph.add_edge(b, c);

        let result = analyze_scc(&graph);
        assert!(result.is_dag);
        assert_eq!(result.cyclic_count(), 0);
        assert_eq!(result.sccs.len(), 3);

        let mut report = ValidationReport::new();
        result.validate_cycles(&mut report);
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_tarjan_simple_cycle_detected() {
        let mut graph = DependencyGraph::new();
        let a = GraphNode::Combinational(InstanceId::new(1));
        let b = GraphNode::Combinational(InstanceId::new(2));

        // A -> B -> A (Siklus 2 simpul)
        graph.add_edge(a, b);
        graph.add_edge(b, a);

        let result = analyze_scc(&graph);
        assert!(!result.is_dag);
        assert_eq!(result.cyclic_count(), 1);

        let cycle = &result.cyclic_sccs[0];
        assert_eq!(cycle.members.len(), 2);
        assert!(cycle.members.contains(&a));
        assert!(cycle.members.contains(&b));
        assert!(cycle.is_cyclic);

        let mut report = ValidationReport::new();
        result.validate_cycles(&mut report);
        assert_eq!(report.error_count(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::DepSccOscillation
        );
    }

    #[test]
    fn test_tarjan_self_loop_detected() {
        let mut graph = DependencyGraph::new();
        let a = GraphNode::Combinational(InstanceId::new(1));

        // A -> A (Self-loop)
        graph.add_edge(a, a);

        let result = analyze_scc(&graph);
        assert!(!result.is_dag);
        assert_eq!(result.cyclic_count(), 1);
        assert!(result.cyclic_sccs[0].is_cyclic);

        let mut report = ValidationReport::new();
        result.validate_cycles(&mut report);
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_half_adder_fixture_is_dag() {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/simple_circuit.json"
        );
        let content = std::fs::read_to_string(fixture_path).expect("Gagal membaca fixture");
        let project: crate::model::project::Project =
            crate::model::serialization::from_json_str(&content).expect("Gagal deserialize");

        let circuit = project
            .get_circuit(crate::model::id::CircuitId::new(1))
            .expect("Circuit not found");
        let registry = ComponentRegistry::with_builtins();
        let graph = DependencyGraph::from_circuit(circuit, &registry);

        let result = analyze_scc(&graph);
        assert!(result.is_dag);
        assert_eq!(result.cyclic_count(), 0);
    }
}
