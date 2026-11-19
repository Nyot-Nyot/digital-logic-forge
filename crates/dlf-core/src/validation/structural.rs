//! # Validator Struktural Rangkaian (Structural Validators)
//!
//! Modul ini mengimplementasikan aturan pemeriksaan integritas dokumen:
//! - Validasi pin masukan wajib yang tidak terhubung (`CONN-PIN-UNCONNECTED`)
//! - Validasi segmen kawat yang menggantung bebas (`CONN-WIRE-DANGLING`)
//! - Validasi ketidakcocokan lebar bus dan integritas referensi ID (`CONN-WIDTH-MISMATCH`)

use super::ValidationReport;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticStage, SourceRef};
use crate::model::circuit::CircuitDefinition;
use crate::model::component::ComponentRegistry;
use crate::model::pin::{PinDirection, SignalWidth};
use crate::model::project::Project;
use crate::model::wiring::Endpoint;
use std::collections::HashSet;

/// Memeriksa apakah terdapat pin masukan wajib yang mengambang / tidak terhubung (M1.11).
///
/// Setiap pin masukan yang dinyatakan `is_mandatory == true` pada `PinDeclaration`
/// wajib terhubung ke sekurangnya satu `WireSegment` pada sirkuit tersebut.
pub fn validate_unconnected_mandatory_pins(
    circuit: &CircuitDefinition,
    registry: &ComponentRegistry,
    report: &mut ValidationReport,
) {
    // Kumpulkan seluruh PinId yang tersambung pada segmen kawat di sirkuit ini
    let mut connected_pins = HashSet::new();

    for wire in &circuit.wires {
        if let Endpoint::Pin { pin_id, .. } = wire.start {
            connected_pins.insert(pin_id);
        }
        if let Endpoint::Pin { pin_id, .. } = wire.end {
            connected_pins.insert(pin_id);
        }
    }

    // Periksa setiap komponen dan pin masukannya
    for comp in &circuit.components {
        let Some(def) = registry.get(&comp.definition_id) else {
            // Komponen tidak dikenal akan dilaporkan oleh validator definisi
            continue;
        };

        for decl in &def.pins {
            if decl.is_mandatory && decl.direction == PinDirection::Input {
                let pin_connected = comp
                    .get_pin_by_declaration(&decl.id)
                    .map(|p| (p.id, connected_pins.contains(&p.id)));

                if let Some((pin_id, false)) = pin_connected {
                    report.add(
                        Diagnostic::error(
                            DiagnosticCode::ConnPinUnconnected,
                            format!(
                                "Pin masukan wajib '{}' pada komponen '{}' (ID: {}) tidak terhubung ke kawat mana pun",
                                decl.display_name,
                                comp.label.as_deref().unwrap_or(&comp.definition_id),
                                comp.id
                            ),
                            DiagnosticStage::ModelValidation,
                            SourceRef::Pin {
                                instance_id: comp.id,
                                pin_id,
                            },
                        )
                        .with_related(vec![SourceRef::Instance { id: comp.id }]),
                    );
                }
            }
        }
    }
}

/// Memeriksa apakah terdapat segmen kawat yang menggantung bebas tanpa tersambung (M1.12).
///
/// Segmen kawat yang memiliki titik ujung `Endpoint::Free` menghasilkan peringatan
/// `CONN-WIRE-DANGLING` agar pengguna mengetahui adanya kawat sisa atau belum selesai ditarik.
pub fn validate_dangling_wires(circuit: &CircuitDefinition, report: &mut ValidationReport) {
    for wire in &circuit.wires {
        if wire.is_dangling() {
            report.add(Diagnostic::warning(
                DiagnosticCode::ConnWireDangling,
                format!(
                    "Segmen kawat (ID: {}) memiliki ujung yang menggantung bebas tanpa tersambung",
                    wire.id
                ),
                DiagnosticStage::ModelValidation,
                SourceRef::Wire { id: wire.id },
            ));
        }
    }
}

/// Mengambil lebar sinyal dari suatu Endpoint (Pin atau Port) jika valid dan ada di sirkuit.
fn get_endpoint_width(circuit: &CircuitDefinition, endpoint: &Endpoint) -> Option<SignalWidth> {
    match endpoint {
        Endpoint::Pin {
            instance_id,
            pin_id,
        } => circuit
            .find_component(*instance_id)
            .and_then(|c| c.get_pin_by_id(*pin_id))
            .map(|p| p.width),
        Endpoint::Port { id } => circuit.find_port(*id).map(|p| p.width),
        _ => None,
    }
}

/// Memeriksa kesesuaian lebar bit antara terminal yang terhubung dan integritas referensi ID (M1.13).
pub fn validate_bus_width_and_references(
    circuit: &CircuitDefinition,
    report: &mut ValidationReport,
) {
    for wire in &circuit.wires {
        // 1. Verifikasi integritas referensi endpoint
        if let Endpoint::Pin {
            instance_id,
            pin_id,
        } = wire.start
        {
            if circuit.find_component(instance_id).is_none() {
                report.add(Diagnostic::error(
                    DiagnosticCode::ConnPinUnconnected,
                    format!(
                        "Segmen kawat {} merujuk ke InstanceId {} yang tidak ada di sirkuit",
                        wire.id, instance_id
                    ),
                    DiagnosticStage::ModelValidation,
                    SourceRef::Wire { id: wire.id },
                ));
            } else if circuit
                .find_component(instance_id)
                .unwrap()
                .get_pin_by_id(pin_id)
                .is_none()
            {
                report.add(Diagnostic::error(
                    DiagnosticCode::ConnPinUnconnected,
                    format!(
                        "Segmen kawat {} merujuk ke PinId {} yang tidak ada pada komponen {}",
                        wire.id, pin_id, instance_id
                    ),
                    DiagnosticStage::ModelValidation,
                    SourceRef::Wire { id: wire.id },
                ));
            }
        }

        // 2. Verifikasi kecocokan lebar bit jika wire menghubungkan langsung dua terminal
        let width_start = get_endpoint_width(circuit, &wire.start);
        let width_end = get_endpoint_width(circuit, &wire.end);

        if let (Some(w_start), Some(w_end)) = (width_start, width_end)
            && w_start != w_end
        {
            let mut related = Vec::new();
            if let Endpoint::Pin {
                instance_id,
                pin_id,
            } = wire.start
            {
                related.push(SourceRef::Pin {
                    instance_id,
                    pin_id,
                });
            }
            if let Endpoint::Pin {
                instance_id,
                pin_id,
            } = wire.end
            {
                related.push(SourceRef::Pin {
                    instance_id,
                    pin_id,
                });
            }

            report.add(
                Diagnostic::error(
                    DiagnosticCode::ConnWidthMismatch,
                    format!(
                        "Ketidakcocokan lebar bus pada kawat ID {}: terminal awal {} vs terminal akhir {}",
                        wire.id, w_start, w_end
                    ),
                    DiagnosticStage::ModelValidation,
                    SourceRef::Wire { id: wire.id },
                )
                .with_related(related),
            );
        }
    }
}

/// Menjalankan seluruh pemeriksaan validasi struktural pada suatu lembar sirkuit.
pub fn validate_circuit(
    circuit: &CircuitDefinition,
    registry: &ComponentRegistry,
) -> ValidationReport {
    let mut report = ValidationReport::new();
    validate_unconnected_mandatory_pins(circuit, registry, &mut report);
    validate_dangling_wires(circuit, &mut report);
    validate_bus_width_and_references(circuit, &mut report);
    report
}

/// Menjalankan seluruh pemeriksaan validasi struktural pada seluruh sirkuit di dalam proyek.
pub fn validate_project(project: &Project, registry: &ComponentRegistry) -> ValidationReport {
    let mut total_report = ValidationReport::new();
    for circuit in &project.circuits {
        let circuit_report = validate_circuit(circuit, registry);
        total_report.merge(circuit_report);
    }
    total_report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::PortDefinition;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, InstanceId, PinId, WireId};
    use crate::model::wiring::WireSegment;

    #[test]
    fn test_unconnected_mandatory_pins_detected() {
        let registry = ComponentRegistry::with_builtins();
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Test Circuit");

        // Letakkan gerbang AND (input 'a' dan 'b' adalah mandatory)
        let and_def = registry.get("dlf.builtin.and").unwrap();
        let and_inst = and_def
            .instantiate(
                InstanceId::new(1),
                &[PinId::new(2), PinId::new(3), PinId::new(4)],
                Point::ZERO,
            )
            .unwrap();
        circuit.add_component(and_inst);

        // Tanpa kawat sama sekali -> harus ada 2 error CONN-PIN-UNCONNECTED
        let mut report = ValidationReport::new();
        validate_unconnected_mandatory_pins(&circuit, &registry, &mut report);

        assert_eq!(report.error_count(), 2);
        assert!(
            report
                .diagnostics
                .iter()
                .all(|d| d.code == DiagnosticCode::ConnPinUnconnected)
        );
    }

    #[test]
    fn test_dangling_wire_detected() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Test Circuit");

        // Kawat menggantung bebas di salah satu ujungnya
        circuit.add_wire(WireSegment::new(
            WireId::new(10),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Free {
                position: Point::new(100.0, 100.0),
            },
        ));

        let mut report = ValidationReport::new();
        validate_dangling_wires(&circuit, &mut report);

        assert_eq!(report.warning_count(), 1);
        assert_eq!(report.diagnostics[0].code, DiagnosticCode::ConnWireDangling);
    }

    #[test]
    fn test_bus_width_mismatch_detected() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Test Circuit");

        // Port 1-bit
        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "p1",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::ZERO,
        ));

        // Port 8-bit
        circuit.add_port(PortDefinition::new(
            PinId::new(2),
            "p2",
            PinDirection::Output,
            SignalWidth::new(8).unwrap(),
            Point::new(100.0, 0.0),
        ));

        // Sambungkan keduanya secara langsung dengan kawat
        circuit.add_wire(WireSegment::new(
            WireId::new(99),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Port { id: PinId::new(2) },
        ));

        let mut report = ValidationReport::new();
        validate_bus_width_and_references(&circuit, &mut report);

        assert_eq!(report.error_count(), 1);
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::ConnWidthMismatch
        );
    }
}
