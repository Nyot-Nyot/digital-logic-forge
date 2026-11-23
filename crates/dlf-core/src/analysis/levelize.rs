//! # Levelization dan Pengurutan Topologis Kombinasional (Levelized Execution Schedule)
//!
//! Modul ini mengimplementasikan algoritma penghitungan level kedalaman dependensi dan
//! penjadwalan urutan eksekusi statis (*levelized full-cycle execution*) sesuai Bab 2.2.1,
//! Bab 3.4.1, dan Bab 3.10.11 proposal.
//!
//! Pada rangkaian asiklik (DAG), setiap simpul dievaluasi setelah seluruh simpul masukannya
//! siap: $\text{level}(u) = 1 + \max_{(v \to u)} \text{level}(v)$. Simpul pada level yang sama
//! dapat dievaluasi secara independen dan diurutkan secara deterministik berdasarkan ID numerik.

use super::graph::{DependencyGraph, GraphNode};
use super::scc::analyze_scc;
use crate::error::CoreError;
use crate::model::id::InstanceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Jadwal urutan evaluasi statis berbasis level (*Levelized Execution Schedule*).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationSchedule {
    /// Pembagian simpul per tingkatan level kedalaman ($0, 1, 2, \dots$).
    ///
    /// Simpul pada level $L$ dijamin hanya bergantung pada simpul-simpul di level $< L$.
    pub levels: Vec<Vec<GraphNode>>,
    /// Urutan linier datar (*flattened execution order*) dari seluruh simpul.
    pub ordered_nodes: Vec<GraphNode>,
    /// Pemetaan cepat dari setiap simpul graf ke nomor levelnya.
    pub node_to_level: BTreeMap<GraphNode, usize>,
    /// Kedalaman maksimum level rangkaian (panjang jalur kritis kombinasional).
    pub max_depth: usize,
}

impl EvaluationSchedule {
    /// Jumlah total simpul dalam jadwal eksekusi.
    #[inline]
    pub fn len(&self) -> usize {
        self.ordered_nodes.len()
    }

    /// Memeriksa apakah jadwal eksekusi kosong.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ordered_nodes.is_empty()
    }

    /// Jumlah tingkatan level dalam jadwal.
    #[inline]
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Mengambil nomor level untuk suatu simpul graf.
    #[inline]
    pub fn level_of(&self, node: &GraphNode) -> Option<usize> {
        self.node_to_level.get(node).copied()
    }

    /// Mengambil daftar simpul pada tingkatan level tertentu.
    #[inline]
    pub fn nodes_at_level(&self, level: usize) -> Option<&[GraphNode]> {
        self.levels.get(level).map(Vec::as_slice)
    }

    /// Mengekstraksi seluruh `InstanceId` komponen kombinasional yang perlu dievaluasi
    /// sesuai urutan penjadwalan eksekusi.
    pub fn combinational_components(&self) -> Vec<InstanceId> {
        self.ordered_nodes
            .iter()
            .filter_map(|node| match *node {
                GraphNode::Combinational(id) => Some(id),
                _ => None,
            })
            .collect()
    }
}

/// Menghitung urutan eksekusi statis berbasis level (*levelization*) pada `DependencyGraph`.
///
/// # Errors
/// Mengembalikan `CoreError::ValidationError` jika graf memuat siklus kombinasional (bukan DAG).
pub fn levelize_dependency_graph(graph: &DependencyGraph) -> Result<EvaluationSchedule, CoreError> {
    // 1. Verifikasi bahwa graf adalah DAG murni via Tarjan SCC
    let scc_result = analyze_scc(graph);
    if !scc_result.is_dag {
        let cyclic_count = scc_result.cyclic_count();
        return Err(CoreError::ValidationError(format!(
            "Tidak dapat melakukan levelization kombinasional: terdeteksi {} siklus feedback (graf bukan DAG)",
            cyclic_count
        )));
    }

    if graph.nodes.is_empty() {
        return Ok(EvaluationSchedule {
            levels: Vec::new(),
            ordered_nodes: Vec::new(),
            node_to_level: BTreeMap::new(),
            max_depth: 0,
        });
    }

    // 2. Hitung in-degree dan siapkan antrean simpul akar (in-degree == 0)
    let mut in_degrees: BTreeMap<GraphNode, usize> = BTreeMap::new();
    let mut node_levels: BTreeMap<GraphNode, usize> = BTreeMap::new();
    let mut queue = VecDeque::new();

    for &node in &graph.nodes {
        let in_deg = graph.in_degree(node);
        in_degrees.insert(node, in_deg);
        if in_deg == 0 {
            node_levels.insert(node, 0);
            queue.push_back(node);
        }
    }

    // 3. Algoritma Kahn dengan penentuan level kedalaman: level(v) = max(level(v), level(u) + 1)
    let mut visited_count = 0;
    while let Some(u) = queue.pop_front() {
        visited_count += 1;
        let u_level = node_levels[&u];

        if let Some(successors) = graph.adj.get(&u) {
            for &v in successors {
                let v_level = node_levels.entry(v).or_insert(0);
                *v_level = (*v_level).max(u_level + 1);

                let deg = in_degrees.get_mut(&v).expect("v in-degree exists");
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(v);
                }
            }
        }
    }

    if visited_count != graph.nodes.len() {
        return Err(CoreError::ValidationError(
            "Gagal menyelesaikan levelization: jumlah simpul yang diproses tidak sesuai"
                .to_string(),
        ));
    }

    // 4. Kelompokkan simpul berdasarkan level secara terurut deterministik
    let max_depth = node_levels.values().copied().max().map_or(0, |d| d + 1);
    let mut levels: Vec<Vec<GraphNode>> = vec![Vec::new(); max_depth];

    // Gunakan BTreeMap bertingkat untuk menjaga determinisme: Level -> BTreeSet<GraphNode>
    let mut level_groups: BTreeMap<usize, BTreeSet<GraphNode>> = BTreeMap::new();
    for (&node, &lvl) in &node_levels {
        level_groups.entry(lvl).or_default().insert(node);
    }

    for (lvl, nodes) in level_groups {
        levels[lvl] = nodes.into_iter().collect();
    }

    // 5. Bentuk urutan linier datar (flattened ordered nodes)
    let mut ordered_nodes = Vec::with_capacity(graph.nodes.len());
    for lvl_nodes in &levels {
        ordered_nodes.extend_from_slice(lvl_nodes);
    }

    Ok(EvaluationSchedule {
        levels,
        ordered_nodes,
        node_to_level: node_levels,
        max_depth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::component::ComponentRegistry;
    use crate::model::id::PinId;

    #[test]
    fn test_levelize_empty_graph() {
        let graph = DependencyGraph::new();
        let schedule = levelize_dependency_graph(&graph).expect("empty schedule");

        assert_eq!(schedule.len(), 0);
        assert_eq!(schedule.level_count(), 0);
        assert_eq!(schedule.max_depth, 0);
        assert!(schedule.is_empty());
    }

    #[test]
    fn test_levelize_linear_chain() {
        let mut graph = DependencyGraph::new();
        let a = GraphNode::Combinational(InstanceId::new(1));
        let b = GraphNode::Combinational(InstanceId::new(2));
        let c = GraphNode::Combinational(InstanceId::new(3));

        // A (level 0) -> B (level 1) -> C (level 2)
        graph.add_edge(a, b);
        graph.add_edge(b, c);

        let schedule = levelize_dependency_graph(&graph).expect("levelize success");

        assert_eq!(schedule.max_depth, 3);
        assert_eq!(schedule.level_count(), 3);
        assert_eq!(schedule.level_of(&a), Some(0));
        assert_eq!(schedule.level_of(&b), Some(1));
        assert_eq!(schedule.level_of(&c), Some(2));

        assert_eq!(schedule.nodes_at_level(0), Some([a].as_slice()));
        assert_eq!(schedule.nodes_at_level(1), Some([b].as_slice()));
        assert_eq!(schedule.nodes_at_level(2), Some([c].as_slice()));

        assert_eq!(
            schedule.combinational_components(),
            vec![InstanceId::new(1), InstanceId::new(2), InstanceId::new(3)]
        );
    }

    #[test]
    fn test_levelize_half_adder_fixture() {
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

        let schedule = levelize_dependency_graph(&graph).expect("levelize half adder");

        // Pada Half Adder:
        // Level 0: Port input a (2), Port input b (3)
        // Level 1: Gerbang XOR1 (6), AND1 (10)
        // Level 2: Port output sum (4), carry (5)
        assert_eq!(schedule.max_depth, 3);
        assert_eq!(schedule.level_count(), 3);

        let in_a = GraphNode::InputPort(PinId::new(2));
        let in_b = GraphNode::InputPort(PinId::new(3));
        let xor1 = GraphNode::Combinational(InstanceId::new(6));
        let and1 = GraphNode::Combinational(InstanceId::new(10));
        let out_sum = GraphNode::OutputPort(PinId::new(4));
        let out_carry = GraphNode::OutputPort(PinId::new(5));

        assert_eq!(schedule.level_of(&in_a), Some(0));
        assert_eq!(schedule.level_of(&in_b), Some(0));
        assert_eq!(schedule.level_of(&xor1), Some(1));
        assert_eq!(schedule.level_of(&and1), Some(1));
        assert_eq!(schedule.level_of(&out_sum), Some(2));
        assert_eq!(schedule.level_of(&out_carry), Some(2));

        // Komponen kombinasional terjadwal: XOR1 (6) dan AND1 (10) pada level 1
        assert_eq!(
            schedule.combinational_components(),
            vec![InstanceId::new(6), InstanceId::new(10)]
        );
    }

    #[test]
    fn test_levelize_fails_on_cyclic_graph() {
        let mut graph = DependencyGraph::new();
        let a = GraphNode::Combinational(InstanceId::new(1));
        let b = GraphNode::Combinational(InstanceId::new(2));

        // A -> B -> A (Siklus)
        graph.add_edge(a, b);
        graph.add_edge(b, a);

        let result = levelize_dependency_graph(&graph);
        assert!(result.is_err());
    }
}
