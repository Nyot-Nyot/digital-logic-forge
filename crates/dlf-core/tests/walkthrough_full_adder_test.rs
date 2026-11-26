//! # Walkthrough End-to-End: Kompilasi & Simulasi Full Adder 1-Bit
//!
//! Test ini dibuat khusus sebagai dokumentasi hidup (*executable walkthrough*)
//! untuk memperlihatkan alur kerja lengkap dari M1 → M2 → M3 pada satu contoh nyata:
//! **Full Adder 1-Bit** (`A + B + CIN → SUM, COUT`).
//!
//! Jalankan dengan perintah berikut untuk melihat seluruh detail tahapan di terminal:
//! ```powershell
//! cargo test -p dlf-core --test walkthrough_full_adder_test -- --nocapture
//! ```

use dlf_core::analysis::{DependencyGraph, analyze_scc, levelize_dependency_graph};
use dlf_core::compiler::CircuitCompiler;
use dlf_core::connectivity::Netlist;
use dlf_core::model::circuit::{CircuitDefinition, PortDefinition};
use dlf_core::model::component::ComponentRegistry;
use dlf_core::model::geometry::Point;
use dlf_core::model::id::IdGenerator;
use dlf_core::model::pin::{PinDirection, SignalWidth};
use dlf_core::model::wiring::{Endpoint, Junction, WireSegment};
use dlf_core::runtime::{CompiledRuntime, DifferentialHarness, LogicValue, ReferenceEvaluator};
use dlf_core::validation::{ValidationReport, validate_circuit, validate_netlist_electrical};

/// Membantu memformat `LogicValue` 1-bit menjadi karakter `0`, `1`, `X`, atau `Z`.
fn format_bit(lv: LogicValue) -> &'static str {
    match (lv.unk & 1, lv.val & 1) {
        (0, 0) => "0",
        (0, 1) => "1",
        (1, 0) => "X",
        (1, 1) => "Z",
        _ => unreachable!(),
    }
}

#[test]
fn walkthrough_full_adder_compile_and_runtime() {
    println!("\n=================================================================");
    println!("  WALKTHROUGH END-TO-END: FULL ADDER 1-BIT (M1 -> M2 -> M3)");
    println!("=================================================================\n");

    // =========================================================================
    // TAHAP 1: AUTHORING DOCUMENT (M1)
    // =========================================================================
    let mut ids = IdGenerator::new(1);
    let registry = ComponentRegistry::with_builtins();
    let mut circuit = CircuitDefinition::new(ids.allocate_circuit_id(), "FullAdder_1Bit");

    // 1.1 Buat Port Masukan (A, B, CIN) dan Port Keluaran (SUM, COUT)
    let port_a = ids.allocate_pin_id();
    let port_b = ids.allocate_pin_id();
    let port_cin = ids.allocate_pin_id();
    let port_sum = ids.allocate_pin_id();
    let port_cout = ids.allocate_pin_id();

    circuit.add_port(PortDefinition::new(
        port_a,
        "A",
        PinDirection::Input,
        SignalWidth::SINGLE,
        Point::new(0.0, 0.0),
    ));
    circuit.add_port(PortDefinition::new(
        port_b,
        "B",
        PinDirection::Input,
        SignalWidth::SINGLE,
        Point::new(0.0, 40.0),
    ));
    circuit.add_port(PortDefinition::new(
        port_cin,
        "CIN",
        PinDirection::Input,
        SignalWidth::SINGLE,
        Point::new(0.0, 80.0),
    ));
    circuit.add_port(PortDefinition::new(
        port_sum,
        "SUM",
        PinDirection::Output,
        SignalWidth::SINGLE,
        Point::new(300.0, 20.0),
    ));
    circuit.add_port(PortDefinition::new(
        port_cout,
        "COUT",
        PinDirection::Output,
        SignalWidth::SINGLE,
        Point::new(300.0, 80.0),
    ));

    // 1.2 Buat 5 Gerbang Logika Full Adder:
    //   - XOR1: A ^ B
    //   - XOR2: (A ^ B) ^ CIN  -> SUM
    //   - AND1: A & B
    //   - AND2: (A ^ B) & CIN
    //   - OR1:  AND1 | AND2    -> COUT
    let xor1_id = ids.allocate_instance_id();
    let xor1_pins = [
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
    ];
    let xor1 = registry
        .get("dlf.builtin.xor")
        .unwrap()
        .instantiate(xor1_id, &xor1_pins, Point::new(100.0, 0.0))
        .unwrap();

    let xor2_id = ids.allocate_instance_id();
    let xor2_pins = [
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
    ];
    let xor2 = registry
        .get("dlf.builtin.xor")
        .unwrap()
        .instantiate(xor2_id, &xor2_pins, Point::new(200.0, 20.0))
        .unwrap();

    let and1_id = ids.allocate_instance_id();
    let and1_pins = [
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
    ];
    let and1 = registry
        .get("dlf.builtin.and")
        .unwrap()
        .instantiate(and1_id, &and1_pins, Point::new(100.0, 80.0))
        .unwrap();

    let and2_id = ids.allocate_instance_id();
    let and2_pins = [
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
    ];
    let and2 = registry
        .get("dlf.builtin.and")
        .unwrap()
        .instantiate(and2_id, &and2_pins, Point::new(200.0, 80.0))
        .unwrap();

    let or1_id = ids.allocate_instance_id();
    let or1_pins = [
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
        ids.allocate_pin_id(),
    ];
    let or1 = registry
        .get("dlf.builtin.or")
        .unwrap()
        .instantiate(or1_id, &or1_pins, Point::new(260.0, 80.0))
        .unwrap();

    circuit.add_component(xor1);
    circuit.add_component(xor2);
    circuit.add_component(and1);
    circuit.add_component(and2);
    circuit.add_component(or1);

    // 1.3 Buat Junction untuk percabangan sinyal (fanout)
    let j_a = ids.allocate_junction_id(); // A -> XOR1.a, AND1.a
    let j_b = ids.allocate_junction_id(); // B -> XOR1.b, AND1.b
    let j_axorb = ids.allocate_junction_id(); // XOR1.y -> XOR2.a, AND2.a
    let j_cin = ids.allocate_junction_id(); // CIN -> XOR2.b, AND2.b

    for (jid, x, y) in [
        (j_a, 50.0, 0.0),
        (j_b, 50.0, 40.0),
        (j_axorb, 150.0, 20.0),
        (j_cin, 150.0, 80.0),
    ] {
        circuit.add_junction(Junction::new(jid, Point::new(x, y)));
    }

    // 1.4 Hubungkan seluruh segmen kawat (WireSegment)
    let mut add_wire = |start: Endpoint, end: Endpoint| {
        let wid = ids.allocate_wire_id();
        circuit.add_wire(WireSegment::new(wid, start, end));
    };

    // Cabang A
    add_wire(
        Endpoint::Port { id: port_a },
        Endpoint::Junction { id: j_a },
    );
    add_wire(
        Endpoint::Junction { id: j_a },
        Endpoint::Pin {
            instance_id: xor1_id,
            pin_id: xor1_pins[0],
        },
    );
    add_wire(
        Endpoint::Junction { id: j_a },
        Endpoint::Pin {
            instance_id: and1_id,
            pin_id: and1_pins[0],
        },
    );

    // Cabang B
    add_wire(
        Endpoint::Port { id: port_b },
        Endpoint::Junction { id: j_b },
    );
    add_wire(
        Endpoint::Junction { id: j_b },
        Endpoint::Pin {
            instance_id: xor1_id,
            pin_id: xor1_pins[1],
        },
    );
    add_wire(
        Endpoint::Junction { id: j_b },
        Endpoint::Pin {
            instance_id: and1_id,
            pin_id: and1_pins[1],
        },
    );

    // Cabang (A ^ B) dari XOR1.y
    add_wire(
        Endpoint::Pin {
            instance_id: xor1_id,
            pin_id: xor1_pins[2],
        },
        Endpoint::Junction { id: j_axorb },
    );
    add_wire(
        Endpoint::Junction { id: j_axorb },
        Endpoint::Pin {
            instance_id: xor2_id,
            pin_id: xor2_pins[0],
        },
    );
    add_wire(
        Endpoint::Junction { id: j_axorb },
        Endpoint::Pin {
            instance_id: and2_id,
            pin_id: and2_pins[0],
        },
    );

    // Cabang CIN
    add_wire(
        Endpoint::Port { id: port_cin },
        Endpoint::Junction { id: j_cin },
    );
    add_wire(
        Endpoint::Junction { id: j_cin },
        Endpoint::Pin {
            instance_id: xor2_id,
            pin_id: xor2_pins[1],
        },
    );
    add_wire(
        Endpoint::Junction { id: j_cin },
        Endpoint::Pin {
            instance_id: and2_id,
            pin_id: and2_pins[1],
        },
    );

    // Output SUM = XOR2.y
    add_wire(
        Endpoint::Pin {
            instance_id: xor2_id,
            pin_id: xor2_pins[2],
        },
        Endpoint::Port { id: port_sum },
    );

    // COUT = AND1.y | AND2.y
    add_wire(
        Endpoint::Pin {
            instance_id: and1_id,
            pin_id: and1_pins[2],
        },
        Endpoint::Pin {
            instance_id: or1_id,
            pin_id: or1_pins[0],
        },
    );
    add_wire(
        Endpoint::Pin {
            instance_id: and2_id,
            pin_id: and2_pins[2],
        },
        Endpoint::Pin {
            instance_id: or1_id,
            pin_id: or1_pins[1],
        },
    );
    add_wire(
        Endpoint::Pin {
            instance_id: or1_id,
            pin_id: or1_pins[2],
        },
        Endpoint::Port { id: port_cout },
    );

    let structural_report = validate_circuit(&circuit, &registry);
    assert!(!structural_report.has_errors());

    println!("[TAHAP 1: AUTHORING DOCUMENT (M1)]");
    println!("  Nama Sirkuit : {}", circuit.name);
    println!(
        "  Ports        : {} port (A, B, CIN, SUM, COUT)",
        circuit.ports.len()
    );
    println!(
        "  Komponen     : {} gerbang (XOR1={}, XOR2={}, AND1={}, AND2={}, OR1={})",
        circuit.components.len(),
        xor1_id,
        xor2_id,
        and1_id,
        and2_id,
        or1_id
    );
    println!("  Junctions    : {}", circuit.junctions.len());
    println!("  WireSegments : {}", circuit.wires.len());
    println!("  Validasi M1  : OK (0 error)\n");

    // =========================================================================
    // TAHAP 2: CONNECTIVITY & SEMANTIC ANALYSIS (M2)
    // =========================================================================
    let netlist = Netlist::from_circuit(&circuit);
    let mut elec_report = ValidationReport::new();
    validate_netlist_electrical(&netlist, &mut elec_report);
    assert!(!elec_report.has_errors());

    let graph = DependencyGraph::from_circuit_and_netlist(&circuit, &netlist, &registry);
    let scc_result = analyze_scc(&graph);
    assert!(scc_result.is_dag);

    let schedule = levelize_dependency_graph(&graph).unwrap();

    println!("[TAHAP 2: CONNECTIVITY & SEMANTIC ANALYSIS (M2)]");
    println!(
        "  Netlist Kanonis (DSU menghasilkan {} Net):",
        netlist.nets.len()
    );
    for (net_id, net) in &netlist.nets {
        println!(
            "    - {} (width={}): drivers={:?}, loads={:?}",
            net_id,
            net.width.get(),
            net.drivers,
            net.loads
        );
    }
    println!("  Validasi Elektris : OK (0 multi-driver, 0 undriven)");
    println!(
        "  Tarjan SCC        : is_dag={} ({} SCC berukuran 1)",
        scc_result.is_dag,
        scc_result.sccs.len()
    );
    println!(
        "  EvaluationSchedule (Levelization -> {} level):",
        schedule.level_count()
    );
    for (lvl, nodes) in schedule.levels.iter().enumerate() {
        println!("    - Level {}: {:?}", lvl, nodes);
    }
    println!();

    // =========================================================================
    // TAHAP 3: SNAPSHOT, SLOT ALLOCATION & IR LOWERING (M3)
    // =========================================================================
    let compiled = CircuitCompiler::compile(&circuit, &registry).unwrap();
    let artifact = &compiled.artifact;

    println!("[TAHAP 3: COMPILER & IR LOWERING (M3)]");
    println!("  Total Slot Memori : {} slot", artifact.slot_count);
    println!("  Input Bindings    :");
    for b in &artifact.input_bindings {
        println!(
            "    - '{}' ({}) -> slot[{}] (width={})",
            b.name, b.pin_id, b.slot, b.width
        );
    }
    println!("  Output Bindings   :");
    for b in &artifact.output_bindings {
        println!(
            "    - '{}' ({}) -> slot[{}] (width={})",
            b.name, b.pin_id, b.slot, b.width
        );
    }
    println!("  SourceMap (Slot -> NetId):");
    for (slot, net_id) in &artifact.source_map.slot_to_net {
        println!("    - slot[{}] <-> {}", slot, net_id);
    }
    println!(
        "  Compiled Operations ({} instruksi terurut level):",
        artifact.operations.len()
    );
    for (lvl, range) in artifact.level_ranges.iter().enumerate() {
        println!("    -- Level {} (ops {:?}) --", lvl, range);
        for op_idx in range.clone() {
            let op = &artifact.operations[op_idx];
            let src_comp = artifact.source_map.op_to_instance.get(&op_idx).unwrap();
            println!(
                "       op[{}]: {:<4} dst=slot[{}]  src_a=slot[{}]  src_b=slot[{}]  width={}  (dari {})",
                op_idx, op.opcode, op.dst, op.src_a, op.src_b, op.width, src_comp
            );
        }
    }
    println!();

    // =========================================================================
    // TAHAP 4: EKSEKUSI RUNTIME (B1 COMPILED vs B0 REFERENCE ORACLE)
    // =========================================================================
    let mut b1 = CompiledRuntime::from_artifact(compiled.artifact.clone()).unwrap();
    let mut b0 = ReferenceEvaluator::new(
        compiled.snapshot.circuit.clone(),
        compiled.netlist.clone(),
        compiled.schedule.clone(),
    );

    let print_all_slots = |rt: &CompiledRuntime, label: &str| {
        let states: Vec<String> = (0..rt.slot_count())
            .map(|s| format!("s{}={}", s, format_bit(rt.probe_slot(s).unwrap())))
            .collect();
        println!("    {:<28}: [{}]", label, states.join(" | "));
    };

    println!("[TAHAP 4: RUNTIME EXECUTION STATE (B1 vs B0)]");
    print_all_slots(&b1, "State Awal (Reset/X)");

    // Skenario 1: A=1, B=1, CIN=0 -> SUM=0, COUT=1
    b1.set_input("A", LogicValue::from_u64(1, 1)).unwrap();
    b1.set_input("B", LogicValue::from_u64(1, 1)).unwrap();
    b1.set_input("CIN", LogicValue::from_u64(0, 1)).unwrap();
    print_all_slots(&b1, "Setelah SetInput(1,1,0)");

    b1.step_cycle();
    print_all_slots(&b1, "Setelah step_cycle() [B1]");

    b0.set_input("A", LogicValue::from_u64(1, 1)).unwrap();
    b0.set_input("B", LogicValue::from_u64(1, 1)).unwrap();
    b0.set_input("CIN", LogicValue::from_u64(0, 1)).unwrap();
    b0.step_cycle().unwrap();

    assert_eq!(b1.get_output("SUM").unwrap(), b0.get_output("SUM").unwrap());
    assert_eq!(
        b1.get_output("COUT").unwrap(),
        b0.get_output("COUT").unwrap()
    );
    assert_eq!(b1.get_output("SUM").unwrap().val, 0);
    assert_eq!(b1.get_output("COUT").unwrap().val, 1);

    // Skenario 2: A=1, B=1, CIN=X -> A^B=0, A&B=1, sehingga COUT=1 (pasti!), SUM=X
    println!();
    b1.set_input("A", LogicValue::from_u64(1, 1)).unwrap();
    b1.set_input("B", LogicValue::from_u64(1, 1)).unwrap();
    b1.set_input("CIN", LogicValue::all_x(1)).unwrap();
    b1.step_cycle();
    print_all_slots(&b1, "Setelah Input(1,1,X) [B1]");

    b0.set_input("A", LogicValue::from_u64(1, 1)).unwrap();
    b0.set_input("B", LogicValue::from_u64(1, 1)).unwrap();
    b0.set_input("CIN", LogicValue::all_x(1)).unwrap();
    b0.step_cycle().unwrap();

    let sum_x = b1.get_output("SUM").unwrap();
    let cout_x = b1.get_output("COUT").unwrap();
    println!(
        "    Hasil Konservatif (1,1,X) : SUM={}, COUT={} (B0 & B1 identik!)",
        format_bit(sum_x),
        format_bit(cout_x)
    );
    assert_eq!(sum_x, b0.get_output("SUM").unwrap());
    assert_eq!(cout_x, b0.get_output("COUT").unwrap());
    assert_eq!(sum_x, LogicValue::all_x(1));
    assert_eq!(cout_x, LogicValue::from_u64(1, 1));

    // Skenario 3: Differential Testing Exhaustive seluruh 8 kombinasi biner (000..111)
    let mut harness = DifferentialHarness::new(&circuit, &registry).unwrap();
    let report = harness.check_exhaustive_2val().unwrap();
    assert!(report.is_clean());
    println!(
        "\n  Differential Check (B0 vs B1): {}/{} kombinasi biner LULUS 100% bit-exact!",
        report.vectors_tested, report.vectors_tested
    );
    println!("=================================================================\n");
}
