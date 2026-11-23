//! # Test Suite Integrasi Konektivitas dan Analisis Semantik (M2 Integration Tests)
//!
//! Rangkaian tes integrasi end-to-end yang menguji pipeline kompilasi frontend rangkaian:
//! 1. Sirkuit Half-Adder: Konektivitas -> Netlist Kanonis -> Validasi Elektris -> Graf Ketergantungan -> Tarjan SCC -> Levelization
//! 2. Rangkaian Sekuensial: Register (DFF) + Logika Feedback -> Pemutusan siklus kombinasional (DAG valid)
//! 3. Rangkaian Ring Oscillator: Siklus kombinasional ilegal -> Deteksi `DEP-SCC-OSCILLATION` dan penolakan levelization
//! 4. Validasi Elektris: Multi-driver conflict (`NET-MULTI-DRIVER`) dan Undriven net (`NET-UNDRIVEN`)
//! 5. Koneksi Nirkabel: Tunnel pair resolving end-to-end
//! 6. Elaborasi Hierarki: Sub-sirkuit port-to-pin binding dan deteksi rekursi (`HIER-RECURSION`)

use dlf_core::analysis::{
    DependencyGraph, GraphNode, analyze_scc, elaborate_hierarchy, levelize_dependency_graph,
    validate_hierarchy_recursion,
};
use dlf_core::connectivity::{NetTerminal, Netlist, resolve_circuit_connectivity};
use dlf_core::diagnostic::DiagnosticCode;
use dlf_core::model::circuit::CircuitDefinition;
use dlf_core::model::component::{ComponentInstance, ComponentRegistry};
use dlf_core::model::geometry::{Point, Rotation};
use dlf_core::model::id::{CircuitId, InstanceId, PinId, TunnelId, WireId};
use dlf_core::model::project::Project;
use dlf_core::model::wiring::{Endpoint, Tunnel, TunnelScope, WireSegment};
use dlf_core::validation::{
    ValidationReport, validate_multi_drivers, validate_netlist_electrical, validate_undriven_nets,
};
use std::collections::BTreeMap;

#[test]
fn test_half_adder_end_to_end_pipeline() {
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple_circuit.json"
    );
    let content = std::fs::read_to_string(fixture_path).expect("Gagal membaca berkas fixture");
    let project: Project =
        dlf_core::model::serialization::from_json_str(&content).expect("Gagal deserialize proyek");

    let circuit = project
        .get_circuit(CircuitId::new(1))
        .expect("Sirkuit utama harus ada");
    let registry = ComponentRegistry::with_builtins();

    // 1. Ekstraksi DSU dan Netlist kanonis
    let mut dsu = resolve_circuit_connectivity(circuit);
    let netlist = Netlist::from_circuit_and_dsu(circuit, &mut dsu);

    // 4 net fungsional pada Half-Adder
    assert_eq!(netlist.len(), 4);

    // 2. Validasi elektris (harus bersih tanpa error dan tanpa warning)
    let mut val_report = ValidationReport::new();
    validate_netlist_electrical(&netlist, &mut val_report);
    assert!(
        !val_report.has_errors(),
        "Half adder yang valid tidak boleh memiliki error elektris"
    );
    assert_eq!(val_report.error_count(), 0);

    // 3. Graf ketergantungan
    let graph = DependencyGraph::from_circuit_and_netlist(circuit, &netlist, &registry);
    assert_eq!(graph.node_count(), 6);
    assert_eq!(graph.edge_count(), 6);

    // 4. Analisis Tarjan SCC
    let scc_result = analyze_scc(&graph);
    assert!(
        scc_result.is_dag,
        "Graf ketergantungan Half-Adder harus berupa DAG murni"
    );
    assert_eq!(scc_result.cyclic_count(), 0);

    // 5. Levelization dan penjadwalan eksekusi
    let schedule =
        levelize_dependency_graph(&graph).expect("Levelization harus berhasil untuk DAG");
    assert_eq!(schedule.max_depth, 3);
    assert_eq!(schedule.level_count(), 3);

    // Level 0: Port Input a (2) & b (3)
    let lvl0 = schedule.nodes_at_level(0).unwrap();
    assert_eq!(lvl0.len(), 2);
    assert!(lvl0.contains(&GraphNode::InputPort(PinId::new(2))));
    assert!(lvl0.contains(&GraphNode::InputPort(PinId::new(3))));

    // Level 1: Gerbang XOR1 (6) & AND1 (10)
    let lvl1 = schedule.nodes_at_level(1).unwrap();
    assert_eq!(lvl1.len(), 2);
    assert!(lvl1.contains(&GraphNode::Combinational(InstanceId::new(6))));
    assert!(lvl1.contains(&GraphNode::Combinational(InstanceId::new(10))));

    // Level 2: Port Output sum (4) & carry (5)
    let lvl2 = schedule.nodes_at_level(2).unwrap();
    assert_eq!(lvl2.len(), 2);
    assert!(lvl2.contains(&GraphNode::OutputPort(PinId::new(4))));
    assert!(lvl2.contains(&GraphNode::OutputPort(PinId::new(5))));

    // Komponen kombinasional siap dieksekusi
    let exec_comps = schedule.combinational_components();
    assert_eq!(exec_comps, vec![InstanceId::new(6), InstanceId::new(10)]);
}

#[test]
fn test_sequential_loop_pipeline() {
    let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Sequential Counter Loop");
    let registry = ComponentRegistry::with_builtins();

    let dff_def = registry.get("dlf.builtin.dff").expect("dff");
    let not_def = registry.get("dlf.builtin.not").expect("not");

    // DFF (Instance 10)
    let dff = dff_def
        .instantiate(
            InstanceId::new(10),
            &[
                PinId::new(11), // d
                PinId::new(12), // clk
                PinId::new(13), // en
                PinId::new(14), // rst
                PinId::new(15), // q
            ],
            Point::new(0.0, 0.0),
        )
        .unwrap();

    // NOT (Instance 20)
    let not = not_def
        .instantiate(
            InstanceId::new(20),
            &[PinId::new(21), PinId::new(22)], // in, out
            Point::new(100.0, 0.0),
        )
        .unwrap();

    circuit.add_component(dff);
    circuit.add_component(not);

    // Kawat: DFF.q (15) -> NOT.in (21)
    circuit.add_wire(WireSegment::new(
        WireId::new(101),
        Endpoint::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(15),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(21),
        },
    ));

    // Kawat: NOT.out (22) -> DFF.d (11)
    circuit.add_wire(WireSegment::new(
        WireId::new(102),
        Endpoint::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(22),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(11),
        },
    ));

    let graph = DependencyGraph::from_circuit(&circuit, &registry);
    let scc_result = analyze_scc(&graph);

    // Register memutus siklus kombinasional antar siklus clock, sehingga tetap DAG!
    assert!(scc_result.is_dag);
    assert_eq!(scc_result.cyclic_count(), 0);

    let schedule = levelize_dependency_graph(&graph).expect("Levelize sequential circuit");
    assert_eq!(
        schedule.level_of(&GraphNode::SequentialSource(InstanceId::new(10))),
        Some(0)
    );
    assert_eq!(
        schedule.level_of(&GraphNode::Combinational(InstanceId::new(20))),
        Some(1)
    );
    assert_eq!(
        schedule.level_of(&GraphNode::SequentialSink(InstanceId::new(10))),
        Some(2)
    );
}

#[test]
fn test_ring_oscillator_cycle_detection() {
    let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Ring Oscillator");
    let registry = ComponentRegistry::with_builtins();
    let not_def = registry.get("dlf.builtin.not").expect("not");

    // 3 gerbang NOT kombinasional: 1 -> 2 -> 3 -> 1
    let not1 = not_def
        .instantiate(
            InstanceId::new(10),
            &[PinId::new(11), PinId::new(12)],
            Point::new(0.0, 0.0),
        )
        .unwrap();
    let not2 = not_def
        .instantiate(
            InstanceId::new(20),
            &[PinId::new(21), PinId::new(22)],
            Point::new(50.0, 0.0),
        )
        .unwrap();
    let not3 = not_def
        .instantiate(
            InstanceId::new(30),
            &[PinId::new(31), PinId::new(32)],
            Point::new(100.0, 0.0),
        )
        .unwrap();

    circuit.add_component(not1);
    circuit.add_component(not2);
    circuit.add_component(not3);

    // Kawat NOT1.out -> NOT2.in
    circuit.add_wire(WireSegment::new(
        WireId::new(101),
        Endpoint::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(12),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(21),
        },
    ));
    // Kawat NOT2.out -> NOT3.in
    circuit.add_wire(WireSegment::new(
        WireId::new(102),
        Endpoint::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(22),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(30),
            pin_id: PinId::new(31),
        },
    ));
    // Kawat NOT3.out -> NOT1.in (Menutup siklus kombinasional ilegal!)
    circuit.add_wire(WireSegment::new(
        WireId::new(103),
        Endpoint::Pin {
            instance_id: InstanceId::new(30),
            pin_id: PinId::new(32),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(11),
        },
    ));

    let graph = DependencyGraph::from_circuit(&circuit, &registry);
    let scc_result = analyze_scc(&graph);

    assert!(!scc_result.is_dag);
    assert_eq!(scc_result.cyclic_count(), 1);

    let mut report = ValidationReport::new();
    scc_result.validate_cycles(&mut report);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        DiagnosticCode::DepSccOscillation
    );

    // Levelization harus menolak graf yang memiliki siklus
    let level_res = levelize_dependency_graph(&graph);
    assert!(level_res.is_err());
}

#[test]
fn test_electrical_conflict_diagnostics() {
    let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Electrical Diagnostics");
    let registry = ComponentRegistry::with_builtins();
    let not_def = registry.get("dlf.builtin.not").expect("not");

    let not1 = not_def
        .instantiate(
            InstanceId::new(10),
            &[PinId::new(11), PinId::new(12)],
            Point::new(0.0, 0.0),
        )
        .unwrap();
    let not2 = not_def
        .instantiate(
            InstanceId::new(20),
            &[PinId::new(21), PinId::new(22)],
            Point::new(50.0, 0.0),
        )
        .unwrap();

    circuit.add_component(not1);
    circuit.add_component(not2);

    // Kawat antara NOT1.out (12) dan NOT2.out (22) -> Multi Driver!
    circuit.add_wire(WireSegment::new(
        WireId::new(101),
        Endpoint::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(12),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(22),
        },
    ));

    let netlist = Netlist::from_circuit(&circuit);

    // 1. Uji Multi-Driver
    let mut md_report = ValidationReport::new();
    validate_multi_drivers(&netlist, &mut md_report);
    assert_eq!(md_report.error_count(), 1);
    assert_eq!(
        md_report.diagnostics[0].code,
        DiagnosticCode::NetMultiDriver
    );

    // 2. Uji Undriven Net (pin masukan 11 dan 21 berdiri sendiri tanpa driver)
    let mut ud_report = ValidationReport::new();
    validate_undriven_nets(&netlist, &mut ud_report);
    assert_eq!(ud_report.error_count(), 2);
    assert_eq!(ud_report.diagnostics[0].code, DiagnosticCode::NetUndriven);
    assert_eq!(ud_report.diagnostics[1].code, DiagnosticCode::NetUndriven);
}

#[test]
fn test_wireless_tunnel_resolution_end_to_end() {
    let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Tunnel Circuit");
    let registry = ComponentRegistry::with_builtins();
    let not_def = registry.get("dlf.builtin.not").expect("not");

    let not1 = not_def
        .instantiate(
            InstanceId::new(10),
            &[PinId::new(11), PinId::new(12)], // 12: out
            Point::new(0.0, 0.0),
        )
        .unwrap();
    let not2 = not_def
        .instantiate(
            InstanceId::new(20),
            &[PinId::new(21), PinId::new(22)], // 21: in
            Point::new(200.0, 0.0),
        )
        .unwrap();

    circuit.add_component(not1);
    circuit.add_component(not2);

    // Tunnel A (terhubung ke NOT1.out)
    circuit.add_tunnel(Tunnel::new(
        TunnelId::new(1001),
        "SIG_BUS",
        TunnelScope::Local,
        Point::new(30.0, 0.0),
        PinId::new(1001),
    ));
    circuit.add_wire(WireSegment::new(
        WireId::new(201),
        Endpoint::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(12),
        },
        Endpoint::Port {
            id: PinId::new(1001),
        },
    ));

    // Tunnel B (terhubung ke NOT2.in) dengan nama label identik "SIG_BUS"
    circuit.add_tunnel(Tunnel::new(
        TunnelId::new(1002),
        "SIG_BUS",
        TunnelScope::Local,
        Point::new(170.0, 0.0),
        PinId::new(1002),
    ));
    circuit.add_wire(WireSegment::new(
        WireId::new(202),
        Endpoint::Port {
            id: PinId::new(1002),
        },
        Endpoint::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(21),
        },
    ));

    let netlist = Netlist::from_circuit(&circuit);

    // Harus terhubung dalam satu net bersama
    let net_id_out = netlist
        .net_for_pin(InstanceId::new(10), PinId::new(12))
        .unwrap();
    let net_id_in = netlist
        .net_for_pin(InstanceId::new(20), PinId::new(21))
        .unwrap();
    assert_eq!(net_id_out, net_id_in);

    let joined_net = netlist.get_net(net_id_out).unwrap();
    assert_eq!(joined_net.drivers.len(), 1);
    assert_eq!(
        joined_net.drivers[0],
        NetTerminal::Pin {
            instance_id: InstanceId::new(10),
            pin_id: PinId::new(12),
        }
    );
    assert_eq!(joined_net.loads.len(), 1);
    assert_eq!(
        joined_net.loads[0],
        NetTerminal::Pin {
            instance_id: InstanceId::new(20),
            pin_id: PinId::new(21),
        }
    );
}

#[test]
fn test_hierarchy_recursion_prevention() {
    let mut project = Project::new("Hierarchy Recursion Check");
    let c1_id = project.root_circuit_id;
    let c2_id = project.create_circuit("ChildCircuit");

    // C1 instansiasi C2
    let comp1 = ComponentInstance {
        id: project.allocate_id().into(),
        definition_id: format!("dlf.circuit.{}", c2_id),
        schema_version: 1,
        label: Some("chip2".to_string()),
        position: Point::new(0.0, 0.0),
        rotation: Rotation::R0,
        mirror_x: false,
        mirror_y: false,
        pins: Vec::new(),
        parameters: BTreeMap::new(),
    };
    project.get_circuit_mut(c1_id).unwrap().add_component(comp1);

    // C2 instansiasi C1 kembali -> rekursi mutual
    let comp2 = ComponentInstance {
        id: project.allocate_id().into(),
        definition_id: format!("dlf.circuit.{}", c1_id),
        schema_version: 1,
        label: Some("chip1_loop".to_string()),
        position: Point::new(0.0, 0.0),
        rotation: Rotation::R0,
        mirror_x: false,
        mirror_y: false,
        pins: Vec::new(),
        parameters: BTreeMap::new(),
    };
    project.get_circuit_mut(c2_id).unwrap().add_component(comp2);

    let mut report = ValidationReport::new();
    validate_hierarchy_recursion(&project, &mut report);

    assert!(report.has_errors());
    assert_eq!(report.error_count(), 1);
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::HierRecursion);

    assert!(elaborate_hierarchy(&project).is_err());
}
