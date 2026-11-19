//! # Integration Test: Round-Trip Fixture JSON
//!
//! Pengujian integrasi ini memvalidasi bahwa berkas fixture JSON (`tests/fixtures/simple_circuit.json`)
//! dapat dimuat ke dalam memori, diubah kembali menjadi JSON, dan menghasilkan representasi
//! dokumen yang identik secara semantik (*bit-exact round-trip*) sesuai kebutuhan KF-01.

use dlf_core::model::id::{CircuitId, InstanceId, JunctionId, PinId, WireId};
use dlf_core::model::pin::PinDirection;
use dlf_core::model::wiring::Endpoint;
use dlf_core::model::{from_json_str, to_json_pretty};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_roundtrip_simple_circuit_fixture() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Arahkan ke root workspace tests/fixtures/simple_circuit.json
    fixture_path.pop(); // crates/
    fixture_path.pop(); // root/
    fixture_path.push("tests");
    fixture_path.push("fixtures");
    fixture_path.push("simple_circuit.json");

    assert!(
        fixture_path.exists(),
        "Berkas fixture harus ada pada: {:?}",
        fixture_path
    );

    let raw_json = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Gagal membaca berkas fixture {:?}: {}", fixture_path, e));

    // 1. Deserialisasi dari JSON fixture
    let project = from_json_str(&raw_json).expect("Gagal deserialize project dari fixture JSON");

    // 2. Verifikasi konten dokumen
    assert_eq!(project.schema_version, 1);
    assert_eq!(project.name, "Half Adder Demo");
    assert_eq!(project.next_id, 16);
    assert_eq!(project.root_circuit_id, CircuitId::new(1));
    assert_eq!(project.circuits.len(), 1);

    let main_circuit = project.get_root_circuit().expect("Root circuit harus ada");
    assert_eq!(main_circuit.name, "main");
    assert_eq!(main_circuit.ports.len(), 4);
    assert_eq!(main_circuit.components.len(), 2);
    assert_eq!(main_circuit.junctions.len(), 2);
    assert_eq!(main_circuit.wires.len(), 8);

    // Verifikasi Ports
    let port_a = main_circuit.find_port(PinId::new(2)).unwrap();
    assert_eq!(port_a.name, "a");
    assert_eq!(port_a.direction, PinDirection::Input);

    let port_sum = main_circuit.find_port(PinId::new(4)).unwrap();
    assert_eq!(port_sum.name, "sum");
    assert_eq!(port_sum.direction, PinDirection::Output);

    // Verifikasi Komponen XOR dan AND
    let xor_comp = main_circuit.find_component(InstanceId::new(6)).unwrap();
    assert_eq!(xor_comp.definition_id, "dlf.builtin.xor");
    assert_eq!(xor_comp.label.as_deref(), Some("XOR1"));
    assert_eq!(xor_comp.pins.len(), 3);

    let and_comp = main_circuit.find_component(InstanceId::new(10)).unwrap();
    assert_eq!(and_comp.definition_id, "dlf.builtin.and");
    assert_eq!(and_comp.label.as_deref(), Some("AND1"));

    // Verifikasi Wire koneksi
    let wire_101 = main_circuit.find_wire(WireId::new(101)).unwrap();
    assert_eq!(wire_101.start, Endpoint::Port { id: PinId::new(2) });
    assert_eq!(
        wire_101.end,
        Endpoint::Junction {
            id: JunctionId::new(14)
        }
    );

    let wire_103 = main_circuit.find_wire(WireId::new(103)).unwrap();
    assert_eq!(wire_103.waypoints.len(), 1);
    assert_eq!(wire_103.waypoints[0].x, 120.0);
    assert_eq!(wire_103.waypoints[0].y, 190.0);

    // 3. Serialisasi ulang ke JSON
    let reserialized_json = to_json_pretty(&project).expect("Gagal serialisasi ulang ke JSON");

    // 4. Deserialisasi kedua
    let project_again =
        from_json_str(&reserialized_json).expect("Gagal deserialize dari JSON hasil reserialisasi");

    // 5. Kesetaraan semantik bit-exact
    assert_eq!(
        project, project_again,
        "Proyek hasil round-trip kedua harus identik dengan proyek pertama!"
    );
}
