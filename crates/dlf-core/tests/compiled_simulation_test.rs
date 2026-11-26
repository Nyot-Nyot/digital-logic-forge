//! # End-to-End Compiled Simulation Integration Suite (Milestone M3)
//!
//! Menguji pipeline lengkap `CircuitDefinition → CircuitCompiler → CompiledRuntime (B1)
//! ↔ ReferenceEvaluator (B0)` pada rangkaian kombinasional non-trivial (W01–W04):
//! 1. 4-Bit Ripple-Carry Adder (exhaustive 512 vektor + fuzzing 4-nilai)
//! 2. 8-Bit ALU dengan 2-bit Opcode Select (MUX tree + bus operations + 1.000 vektor fuzz)
//! 3. 8-Bit Barrel Shifter (SLICE + CONCAT + CONST + MUX + Probe SourceMap)
//! 4. 4-to-2 Priority Encoder (exhaustive + fuzzing)

use dlf_core::compiler::CircuitCompiler;
use dlf_core::model::circuit::{CircuitDefinition, PortDefinition};
use dlf_core::model::component::{ComponentInstance, ComponentRegistry, ParameterValue};
use dlf_core::model::geometry::Point;
use dlf_core::model::id::{IdGenerator, JunctionId, PinId};
use dlf_core::model::pin::{PinDirection, SignalWidth};
use dlf_core::model::wiring::{Endpoint, Junction, WireSegment};
use dlf_core::runtime::{CompiledRuntime, DifferentialHarness, LogicValue};

/// Pembangun sirkuit ergonomis untuk menyusun fixture uji kombinasional kompleks.
struct CircuitBuilder {
    circuit: CircuitDefinition,
    registry: ComponentRegistry,
    ids: IdGenerator,
    /// Melacak junction per sumber agar satu driver dapat bercabang ke banyak load tanpa konflik.
    fanout_junctions: Vec<(Endpoint, JunctionId)>,
}

impl CircuitBuilder {
    fn new(name: &str) -> Self {
        let mut ids = IdGenerator::new(1);
        let cid = ids.allocate_circuit_id();
        Self {
            circuit: CircuitDefinition::new(cid, name),
            registry: ComponentRegistry::with_builtins(),
            ids,
            fanout_junctions: Vec::new(),
        }
    }

    fn add_port(&mut self, name: &str, dir: PinDirection, width: u8) -> Endpoint {
        let pid = self.ids.allocate_pin_id();
        self.circuit.add_port(PortDefinition::new(
            pid,
            name,
            dir,
            SignalWidth::new(width).unwrap(),
            Point::ZERO,
        ));
        Endpoint::Port { id: pid }
    }

    fn add_gate(&mut self, def_id: &str, width: u8) -> ComponentInstance {
        let def = self.registry.get(def_id).unwrap();
        let iid = self.ids.allocate_instance_id();
        let pin_ids: Vec<PinId> = (0..def.pins.len())
            .map(|_| self.ids.allocate_pin_id())
            .collect();
        let mut inst = def.instantiate(iid, &pin_ids, Point::ZERO).unwrap();
        for pin in &mut inst.pins {
            // Pada MUX, pin 'sel' tetap selebar 1 bit
            if def_id == "dlf.builtin.mux" && pin.declaration_id == "sel" {
                pin.width = SignalWidth::SINGLE;
            } else {
                pin.width = SignalWidth::new(width).unwrap();
            }
        }
        self.circuit.add_component(inst.clone());
        inst
    }

    fn add_const(&mut self, width: u8, value: u64) -> Endpoint {
        let def = self.registry.get("dlf.builtin.const").unwrap();
        let iid = self.ids.allocate_instance_id();
        let pin_id = self.ids.allocate_pin_id();
        let mut inst = def.instantiate(iid, &[pin_id], Point::ZERO).unwrap();
        inst.pins[0].width = SignalWidth::new(width).unwrap();
        inst.parameters
            .insert("width".to_string(), ParameterValue::Integer(width as i64));
        inst.parameters
            .insert("value".to_string(), ParameterValue::Integer(value as i64));
        self.circuit.add_component(inst);
        Endpoint::Pin {
            instance_id: iid,
            pin_id,
        }
    }

    fn add_slice(&mut self, src: Endpoint, offset: u8, out_width: u8) -> Endpoint {
        let def = self.registry.get("dlf.builtin.slice").unwrap();
        let iid = self.ids.allocate_instance_id();
        let pin_a = self.ids.allocate_pin_id();
        let pin_y = self.ids.allocate_pin_id();
        let mut inst = def.instantiate(iid, &[pin_a, pin_y], Point::ZERO).unwrap();
        inst.pins[0].width = SignalWidth::new(8).unwrap();
        inst.pins[1].width = SignalWidth::new(out_width).unwrap();
        inst.parameters
            .insert("offset".to_string(), ParameterValue::Integer(offset as i64));
        inst.parameters.insert(
            "width".to_string(),
            ParameterValue::Integer(out_width as i64),
        );
        self.circuit.add_component(inst);

        self.connect(
            src,
            Endpoint::Pin {
                instance_id: iid,
                pin_id: pin_a,
            },
        );
        Endpoint::Pin {
            instance_id: iid,
            pin_id: pin_y,
        }
    }

    fn add_slice_width(
        &mut self,
        src: Endpoint,
        in_width: u8,
        offset: u8,
        out_width: u8,
    ) -> Endpoint {
        let def = self.registry.get("dlf.builtin.slice").unwrap();
        let iid = self.ids.allocate_instance_id();
        let pin_a = self.ids.allocate_pin_id();
        let pin_y = self.ids.allocate_pin_id();
        let mut inst = def.instantiate(iid, &[pin_a, pin_y], Point::ZERO).unwrap();
        inst.pins[0].width = SignalWidth::new(in_width).unwrap();
        inst.pins[1].width = SignalWidth::new(out_width).unwrap();
        inst.parameters
            .insert("offset".to_string(), ParameterValue::Integer(offset as i64));
        inst.parameters.insert(
            "width".to_string(),
            ParameterValue::Integer(out_width as i64),
        );
        self.circuit.add_component(inst);

        self.connect(
            src,
            Endpoint::Pin {
                instance_id: iid,
                pin_id: pin_a,
            },
        );
        Endpoint::Pin {
            instance_id: iid,
            pin_id: pin_y,
        }
    }

    fn add_concat(
        &mut self,
        low: Endpoint,
        low_width: u8,
        high: Endpoint,
        high_width: u8,
    ) -> Endpoint {
        let def = self.registry.get("dlf.builtin.concat").unwrap();
        let iid = self.ids.allocate_instance_id();
        let pin_l = self.ids.allocate_pin_id();
        let pin_h = self.ids.allocate_pin_id();
        let pin_y = self.ids.allocate_pin_id();
        let mut inst = def
            .instantiate(iid, &[pin_l, pin_h, pin_y], Point::ZERO)
            .unwrap();
        inst.pins[0].width = SignalWidth::new(low_width).unwrap();
        inst.pins[1].width = SignalWidth::new(high_width).unwrap();
        inst.pins[2].width = SignalWidth::new(low_width + high_width).unwrap();
        self.circuit.add_component(inst);

        self.connect(
            low,
            Endpoint::Pin {
                instance_id: iid,
                pin_id: pin_l,
            },
        );
        self.connect(
            high,
            Endpoint::Pin {
                instance_id: iid,
                pin_id: pin_h,
            },
        );
        Endpoint::Pin {
            instance_id: iid,
            pin_id: pin_y,
        }
    }

    fn pin_ep(inst: &ComponentInstance, decl: &str) -> Endpoint {
        let pin = inst.get_pin_by_declaration(decl).unwrap();
        Endpoint::Pin {
            instance_id: inst.id,
            pin_id: pin.id,
        }
    }

    fn add_unary(&mut self, def_id: &str, width: u8, a: Endpoint) -> Endpoint {
        let inst = self.add_gate(def_id, width);
        let pin_a = Self::pin_ep(&inst, "a");
        let pin_y = Self::pin_ep(&inst, "y");
        self.connect(a, pin_a);
        pin_y
    }

    fn add_binary(&mut self, def_id: &str, width: u8, a: Endpoint, b: Endpoint) -> Endpoint {
        let inst = self.add_gate(def_id, width);
        let pin_a = Self::pin_ep(&inst, "a");
        let pin_b = Self::pin_ep(&inst, "b");
        let pin_y = Self::pin_ep(&inst, "y");
        self.connect(a, pin_a);
        self.connect(b, pin_b);
        pin_y
    }

    fn add_mux(&mut self, width: u8, d0: Endpoint, d1: Endpoint, sel: Endpoint) -> Endpoint {
        let inst = self.add_gate("dlf.builtin.mux", width);
        let p_d0 = Self::pin_ep(&inst, "d0");
        let p_d1 = Self::pin_ep(&inst, "d1");
        let p_sel = Self::pin_ep(&inst, "sel");
        let p_y = Self::pin_ep(&inst, "y");
        self.connect(d0, p_d0);
        self.connect(d1, p_d1);
        self.connect(sel, p_sel);
        p_y
    }

    /// Menghubungkan satu titik sumber (`src`) ke titik beban (`dst`), secara otomatis
    /// menyisipkan `Junction` apabila `src` memiliki lebih dari satu cabang fanout.
    fn connect(&mut self, src: Endpoint, dst: Endpoint) {
        let jid = if let Some(&(_, existing_jid)) =
            self.fanout_junctions.iter().find(|(ep, _)| *ep == src)
        {
            existing_jid
        } else {
            let new_jid = self.ids.allocate_junction_id();
            self.circuit
                .add_junction(Junction::new(new_jid, Point::ZERO));
            let wid = self.ids.allocate_wire_id();
            self.circuit.add_wire(WireSegment::new(
                wid,
                src,
                Endpoint::Junction { id: new_jid },
            ));
            self.fanout_junctions.push((src, new_jid));
            new_jid
        };

        let wid = self.ids.allocate_wire_id();
        self.circuit
            .add_wire(WireSegment::new(wid, Endpoint::Junction { id: jid }, dst));
    }

    /// Membangun satu sel *Full Adder* 1-bit dan mengembalikan `(sum_bit, carry_out)`.
    fn build_full_adder(
        &mut self,
        a: Endpoint,
        b: Endpoint,
        cin: Endpoint,
    ) -> (Endpoint, Endpoint) {
        let axorb = self.add_binary("dlf.builtin.xor", 1, a, b);
        let sum = self.add_binary("dlf.builtin.xor", 1, axorb, cin);
        let aandb = self.add_binary("dlf.builtin.and", 1, a, b);
        let cin_and_axorb = self.add_binary("dlf.builtin.and", 1, cin, axorb);
        let cout = self.add_binary("dlf.builtin.or", 1, aandb, cin_and_axorb);
        (sum, cout)
    }

    fn build(self) -> (CircuitDefinition, ComponentRegistry) {
        (self.circuit, self.registry)
    }
}

#[test]
fn test_4bit_ripple_carry_adder_differential() {
    let mut b = CircuitBuilder::new("RippleCarryAdder4");
    let a = b.add_port("A", PinDirection::Input, 4);
    let rhs = b.add_port("B", PinDirection::Input, 4);
    let cin = b.add_port("CIN", PinDirection::Input, 1);
    let sum_out = b.add_port("SUM", PinDirection::Output, 4);
    let cout_out = b.add_port("COUT", PinDirection::Output, 1);

    let mut sum_bits = Vec::with_capacity(4);
    let mut carry = cin;
    for bit in 0..4 {
        let a_i = b.add_slice_width(a, 4, bit, 1);
        let b_i = b.add_slice_width(rhs, 4, bit, 1);
        let (s_i, c_next) = b.build_full_adder(a_i, b_i, carry);
        sum_bits.push(s_i);
        carry = c_next;
    }

    let low2 = b.add_concat(sum_bits[0], 1, sum_bits[1], 1);
    let high2 = b.add_concat(sum_bits[2], 1, sum_bits[3], 1);
    let sum4 = b.add_concat(low2, 2, high2, 2);

    b.connect(sum4, sum_out);
    b.connect(carry, cout_out);

    let (circuit, registry) = b.build();

    // 1. Verifikasi fungsional langsung di B1
    let compiled = CircuitCompiler::compile(&circuit, &registry).unwrap();
    let mut rt = CompiledRuntime::from_artifact(compiled.artifact).unwrap();
    for av in 0..16u64 {
        for bv in 0..16u64 {
            for cv in 0..2u64 {
                rt.set_input("A", LogicValue::from_u64(av, 4)).unwrap();
                rt.set_input("B", LogicValue::from_u64(bv, 4)).unwrap();
                rt.set_input("CIN", LogicValue::from_u64(cv, 1)).unwrap();
                rt.step_cycle();

                let total = av + bv + cv;
                assert_eq!(rt.get_output("SUM").unwrap().val, total & 0xF);
                assert_eq!(rt.get_output("COUT").unwrap().val, (total >> 4) & 1);
            }
        }
    }

    // 2. Verifikasi diferensial B0 vs B1 exhaustive (512 vektor) + 4-nilai fuzzing (300 vektor)
    let mut harness = DifferentialHarness::new(&circuit, &registry).unwrap();
    let ex_report = harness.check_exhaustive_2val().unwrap();
    assert!(ex_report.is_clean());
    assert_eq!(ex_report.vectors_tested, 512);

    let fuzz_report = harness.fuzz_random(300, 0xA44E_0001, true).unwrap();
    assert!(fuzz_report.is_clean());
}

#[test]
fn test_8bit_alu_opcode_select_differential() {
    let mut b = CircuitBuilder::new("ALU8");
    let a = b.add_port("A", PinDirection::Input, 8);
    let rhs = b.add_port("B", PinDirection::Input, 8);
    let op = b.add_port("OP", PinDirection::Input, 2);
    let y = b.add_port("Y", PinDirection::Output, 8);

    let op0 = b.add_slice_width(op, 2, 0, 1);
    let op1 = b.add_slice_width(op, 2, 1, 1);

    let r_and = b.add_binary("dlf.builtin.and", 8, a, rhs); // OP=00
    let r_or = b.add_binary("dlf.builtin.or", 8, a, rhs); // OP=01
    let r_xor = b.add_binary("dlf.builtin.xor", 8, a, rhs); // OP=10
    let r_not_a = b.add_unary("dlf.builtin.not", 8, a); // OP=11

    let mux_low = b.add_mux(8, r_and, r_or, op0);
    let mux_high = b.add_mux(8, r_xor, r_not_a, op0);
    let res = b.add_mux(8, mux_low, mux_high, op1);

    b.connect(res, y);

    let (circuit, registry) = b.build();

    // 1. Cek fungsional operasi ALU pada B1
    let compiled = CircuitCompiler::compile(&circuit, &registry).unwrap();
    let mut rt = CompiledRuntime::from_artifact(compiled.artifact).unwrap();
    rt.set_input("A", LogicValue::from_u64(0xA5, 8)).unwrap();
    rt.set_input("B", LogicValue::from_u64(0x3C, 8)).unwrap();

    for (opcode, expected) in [
        (0b00, 0xA5 & 0x3C),
        (0b01, 0xA5 | 0x3C),
        (0b10, 0xA5 ^ 0x3C),
        (0b11, (!0xA5u64) & 0xFF),
    ] {
        rt.set_input("OP", LogicValue::from_u64(opcode, 2)).unwrap();
        rt.step_cycle();
        assert_eq!(rt.get_output("Y").unwrap().val, expected);
    }

    // 2. Fuzzing 1.000 vektor termasuk X/Z pada differential harness B0 vs B1
    let mut harness = DifferentialHarness::new(&circuit, &registry).unwrap();
    let fuzz_report = harness.fuzz_random(1000, 0xA100_0008, true).unwrap();
    assert!(fuzz_report.is_clean());
    assert_eq!(fuzz_report.vectors_tested, 1000);
}

#[test]
fn test_8bit_barrel_shifter_and_probe_sourcemap() {
    let mut b = CircuitBuilder::new("BarrelShifterLeft8");
    let data = b.add_port("DATA", PinDirection::Input, 8);
    let shamt = b.add_port("SHAMT", PinDirection::Input, 2);
    let y = b.add_port("Y", PinDirection::Output, 8);

    let s0 = b.add_slice_width(shamt, 2, 0, 1);
    let s1 = b.add_slice_width(shamt, 2, 1, 1);

    // Stage 0: geser kiri 1 bit jika s0 == 1
    let zero1 = b.add_const(1, 0);
    let low7 = b.add_slice(data, 0, 7);
    let shifted1 = b.add_concat(zero1, 1, low7, 7);
    let stage0 = b.add_mux(8, data, shifted1, s0);

    // Stage 1: geser kiri 2 bit jika s1 == 1
    let zero2 = b.add_const(2, 0);
    let low6 = b.add_slice(stage0, 0, 6);
    let shifted2 = b.add_concat(zero2, 2, low6, 6);
    let stage1 = b.add_mux(8, stage0, shifted2, s1);

    b.connect(stage1, y);

    let (circuit, registry) = b.build();
    let compiled = CircuitCompiler::compile(&circuit, &registry).unwrap();

    // Ambil NetId internal dari netlist untuk menguji fitur probe_net()
    let all_nets: Vec<_> = compiled.netlist.nets.keys().copied().collect();
    assert!(!all_nets.is_empty());

    let mut rt = CompiledRuntime::from_artifact(compiled.artifact).unwrap();
    rt.set_input("DATA", LogicValue::from_u64(0b0001_0110, 8))
        .unwrap();
    for shift in 0..4u64 {
        rt.set_input("SHAMT", LogicValue::from_u64(shift, 2))
            .unwrap();
        rt.step_cycle();
        assert_eq!(
            rt.get_output("Y").unwrap().val,
            (0b0001_0110u64 << shift) & 0xFF
        );
    }

    // Pastikan seluruh net internal dapat di-probe via SourceMap tanpa panic/error
    for net_id in all_nets {
        assert!(rt.probe_net(net_id).is_ok());
    }

    let mut harness = DifferentialHarness::new(&circuit, &registry).unwrap();
    let ex_report = harness.check_exhaustive_2val().unwrap();
    assert!(ex_report.is_clean());
    assert_eq!(ex_report.vectors_tested, 1024);
}

#[test]
fn test_4to2_priority_encoder_differential() {
    // Priority encoder: d[3] prioritas tertinggi
    // valid = d3 | d2 | d1 | d0
    // code[1] = d3 | d2
    // code[0] = d3 | (!d2 & d1)
    let mut b = CircuitBuilder::new("PriorityEncoder4to2");
    let d = b.add_port("D", PinDirection::Input, 4);
    let code_out = b.add_port("CODE", PinDirection::Output, 2);
    let valid_out = b.add_port("VALID", PinDirection::Output, 1);

    let d0 = b.add_slice_width(d, 4, 0, 1);
    let d1 = b.add_slice_width(d, 4, 1, 1);
    let d2 = b.add_slice_width(d, 4, 2, 1);
    let d3 = b.add_slice_width(d, 4, 3, 1);

    let d0_or_d1 = b.add_binary("dlf.builtin.or", 1, d0, d1);
    let d2_or_d3 = b.add_binary("dlf.builtin.or", 1, d2, d3);
    let valid = b.add_binary("dlf.builtin.or", 1, d0_or_d1, d2_or_d3);

    let not_d2 = b.add_unary("dlf.builtin.not", 1, d2);
    let not_d2_and_d1 = b.add_binary("dlf.builtin.and", 1, not_d2, d1);
    let code0 = b.add_binary("dlf.builtin.or", 1, d3, not_d2_and_d1);
    let code1 = d2_or_d3;

    let code = b.add_concat(code0, 1, code1, 1);
    b.connect(code, code_out);
    b.connect(valid, valid_out);

    let (circuit, registry) = b.build();

    let mut harness = DifferentialHarness::new(&circuit, &registry).unwrap();
    let ex_report = harness.check_exhaustive_2val().unwrap();
    assert!(ex_report.is_clean());
    assert_eq!(ex_report.vectors_tested, 16);

    let fuzz_report = harness.fuzz_random(200, 0xE4C0_DE04, true).unwrap();
    assert!(fuzz_report.is_clean());
}
