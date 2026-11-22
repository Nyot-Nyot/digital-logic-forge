//! # Validasi Aturan Elektris Netlist (Electrical Validation)
//!
//! Modul ini mengimplementasikan aturan pemeriksaan integritas elektris pada `Netlist` kanonis
//! sesuai Bab 3.4.1, 3.4.3, dan Bab 3.10.8 proposal.
//!
//! Mencakup:
//! - Deteksi konflik multi-driver (`NET-MULTI-DRIVER`): lebih dari satu driver aktif pada net yang tidak diresolusi.
//! - Deteksi net melayang tanpa penggerak (`NET-UNDRIVEN`): net yang memiliki beban tetapi tanpa sumber sinyal.

use crate::connectivity::{NetTerminal, Netlist};
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticStage};
use crate::validation::ValidationReport;

/// Memeriksa seluruh net di dalam `Netlist` untuk mendeteksi konflik multi-driver (`NET-MULTI-DRIVER`).
///
/// Jika sebuah net memiliki dua atau lebih driver aktif tanpa resolusi logika (seperti tri-state),
/// aturan ini menerbitkan error diagnostik dengan `primary` menunjuk ke driver pertama dan `related`
/// memuat seluruh driver lain yang berkonflik.
pub fn validate_multi_drivers(netlist: &Netlist, report: &mut ValidationReport) {
    for (&_net_id, net) in netlist.iter() {
        if net.is_multi_driver() {
            let primary = net.drivers[0].to_source_ref();
            let related = net.drivers[1..]
                .iter()
                .map(NetTerminal::to_source_ref)
                .collect();

            let message = format!(
                "Net {} memiliki {} driver aktif berkonflik tanpa resolusi logika (multi-driver)",
                net.id,
                net.drivers.len()
            );

            let diagnostic = Diagnostic::error(
                DiagnosticCode::NetMultiDriver,
                message,
                DiagnosticStage::Connectivity,
                primary,
            )
            .with_related(related);

            report.add(diagnostic);
        }
    }
}

/// Memeriksa seluruh net di dalam `Netlist` untuk mendeteksi net tanpa penggerak (`NET-UNDRIVEN`).
///
/// Jika sebuah net memiliki terminal beban (*load*) tetapi tidak memiliki driver sama sekali,
/// aturan ini menerbitkan error diagnostik dengan `primary` menunjuk ke beban pertama dan `related`
/// memuat seluruh beban lain yang terdampak.
pub fn validate_undriven_nets(netlist: &Netlist, report: &mut ValidationReport) {
    for (&_net_id, net) in netlist.iter() {
        if net.is_undriven() {
            let primary = net.loads[0].to_source_ref();
            let related = net.loads[1..]
                .iter()
                .map(NetTerminal::to_source_ref)
                .collect();

            let message = format!(
                "Net {} memiliki {} terminal beban tetapi tidak memiliki penggerak sinyal (undriven)",
                net.id,
                net.loads.len()
            );

            let diagnostic = Diagnostic::error(
                DiagnosticCode::NetUndriven,
                message,
                DiagnosticStage::Connectivity,
                primary,
            )
            .with_related(related);

            report.add(diagnostic);
        }
    }
}

/// Menjalankan seluruh pemeriksaan aturan elektris pada `Netlist`.
pub fn validate_netlist_electrical(netlist: &Netlist, report: &mut ValidationReport) {
    validate_multi_drivers(netlist, report);
    validate_undriven_nets(netlist, report);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectivity::Netlist;
    use crate::diagnostic::SourceRef;
    use crate::model::circuit::{CircuitDefinition, PortDefinition};
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, InstanceId, PinId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, WireSegment};

    #[test]
    fn test_multi_driver_detection() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Multi Driver Test");
        let registry = ComponentRegistry::with_builtins();

        // 2 buah gerbang NOT
        let not_def = registry.get("dlf.builtin.not").expect("not definition");
        let comp1 = not_def
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)], // 11: in, 12: out
                Point::new(0.0, 0.0),
            )
            .unwrap();
        let comp2 = not_def
            .instantiate(
                InstanceId::new(20),
                &[PinId::new(21), PinId::new(22)], // 21: in, 22: out
                Point::new(100.0, 0.0),
            )
            .unwrap();

        circuit.add_component(comp1);
        circuit.add_component(comp2);

        // Kawat menghubungkan output comp1 (12) langsung ke output comp2 (22)
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
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
        let mut report = ValidationReport::new();
        validate_multi_drivers(&netlist, &mut report);

        assert_eq!(report.error_count(), 1);
        let diag = &report.diagnostics[0];
        assert_eq!(diag.code, DiagnosticCode::NetMultiDriver);
        assert_eq!(
            diag.primary,
            SourceRef::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(12),
            }
        );
        assert_eq!(diag.related.len(), 1);
        assert_eq!(
            diag.related[0],
            SourceRef::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(22),
            }
        );
    }

    #[test]
    fn test_valid_circuit_has_no_multi_driver() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Valid Driver Test");
        let registry = ComponentRegistry::with_builtins();

        // 1 port input dan 1 gerbang NOT
        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "in",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(0.0, 0.0),
        ));

        let not_def = registry.get("dlf.builtin.not").expect("not definition");
        let comp = not_def
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)],
                Point::new(50.0, 0.0),
            )
            .unwrap();
        circuit.add_component(comp);

        // Kawat dari Port input (1) ke pin in gerbang NOT (11)
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
        ));

        let netlist = Netlist::from_circuit(&circuit);
        let mut report = ValidationReport::new();
        validate_multi_drivers(&netlist, &mut report);

        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_undriven_net_detection() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Undriven Test");
        let registry = ComponentRegistry::with_builtins();

        // 2 buah gerbang NOT
        let not_def = registry.get("dlf.builtin.not").expect("not definition");
        let comp1 = not_def
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)], // 11: in, 12: out
                Point::new(0.0, 0.0),
            )
            .unwrap();
        let comp2 = not_def
            .instantiate(
                InstanceId::new(20),
                &[PinId::new(21), PinId::new(22)], // 21: in, 22: out
                Point::new(100.0, 0.0),
            )
            .unwrap();

        circuit.add_component(comp1);
        circuit.add_component(comp2);

        // Kawat menghubungkan input comp1 (11) langsung ke input comp2 (21)
        // Net ini memiliki 2 load dan 0 driver!
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
            Endpoint::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(21),
            },
        ));

        let netlist = Netlist::from_circuit(&circuit);
        let mut report = ValidationReport::new();
        validate_undriven_nets(&netlist, &mut report);

        // Harusnya mendeteksi net input (11 & 21) sebagai undriven.
        // Output pins 12 dan 22 berdiri sendiri (has driver, 0 loads), jadi bukan undriven.
        assert_eq!(report.error_count(), 1);
        let diag = &report.diagnostics[0];
        assert_eq!(diag.code, DiagnosticCode::NetUndriven);
        assert_eq!(
            diag.primary,
            SourceRef::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            }
        );
        assert_eq!(diag.related.len(), 1);
        assert_eq!(
            diag.related[0],
            SourceRef::Pin {
                instance_id: InstanceId::new(20),
                pin_id: PinId::new(21),
            }
        );
    }

    #[test]
    fn test_valid_circuit_has_no_undriven_net() {
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "Fully Driven Test");
        let registry = ComponentRegistry::with_builtins();

        // 1 port input dan 1 gerbang NOT
        circuit.add_port(PortDefinition::new(
            PinId::new(1),
            "in",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(0.0, 0.0),
        ));

        let not_def = registry.get("dlf.builtin.not").expect("not definition");
        let comp = not_def
            .instantiate(
                InstanceId::new(10),
                &[PinId::new(11), PinId::new(12)],
                Point::new(50.0, 0.0),
            )
            .unwrap();
        circuit.add_component(comp);

        // Kawat dari Port input (1) ke pin in gerbang NOT (11)
        circuit.add_wire(WireSegment::new(
            WireId::new(100),
            Endpoint::Port { id: PinId::new(1) },
            Endpoint::Pin {
                instance_id: InstanceId::new(10),
                pin_id: PinId::new(11),
            },
        ));

        let netlist = Netlist::from_circuit(&circuit);
        let mut report = ValidationReport::new();
        validate_undriven_nets(&netlist, &mut report);

        assert_eq!(report.error_count(), 0);
    }
}
