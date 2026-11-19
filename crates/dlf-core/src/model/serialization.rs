//! # Mesin Serialisasi JSON Dokumen (JSON Serialization Engine)
//!
//! Modul ini menyediakan fungsi-fungsi tingkat tinggi untuk mengubah model dokumen rangkaian
//! (`Project`) menjadi format teks JSON yang terstandarisasi sesuai Bab 3.10.1 proposal.

use super::project::Project;
use crate::error::CoreError;

/// Mengubah model dokumen proyek menjadi string JSON dengan format kompak.
///
/// # Errors
/// Mengembalikan `CoreError::ValidationError` jika proses serialisasi gagal.
pub fn to_json_string(project: &Project) -> Result<String, CoreError> {
    serde_json::to_string(project).map_err(|err| {
        CoreError::ValidationError(format!("Gagal serialisasi proyek ke JSON: {}", err))
    })
}

/// Mengubah model dokumen proyek menjadi string JSON yang rapi (*pretty printed* dengan indentasi).
///
/// Format ini sangat ramah untuk dibaca manusia dan mudah dilacak perubahannya
/// pada sistem kontrol versi (Git).
///
/// # Errors
/// Mengembalikan `CoreError::ValidationError` jika proses serialisasi gagal.
pub fn to_json_pretty(project: &Project) -> Result<String, CoreError> {
    serde_json::to_string_pretty(project).map_err(|err| {
        CoreError::ValidationError(format!("Gagal serialisasi proyek ke JSON pretty: {}", err))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{InstanceId, JunctionId, PinId, WireId};
    use crate::model::pin::{PinDirection, SignalWidth};
    use crate::model::wiring::{Endpoint, Junction, WireSegment};

    #[test]
    fn test_serialize_simple_project() {
        let mut project = Project::new("Half Adder Test");
        let reg = ComponentRegistry::with_builtins();

        let root = project
            .get_root_circuit_mut()
            .expect("Root circuit harus ada");

        // Tambah Ports
        root.add_port(crate::model::circuit::PortDefinition::new(
            PinId::new(2),
            "a",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(50.0, 100.0),
        ));
        root.add_port(crate::model::circuit::PortDefinition::new(
            PinId::new(3),
            "b",
            PinDirection::Input,
            SignalWidth::SINGLE,
            Point::new(50.0, 160.0),
        ));
        root.add_port(crate::model::circuit::PortDefinition::new(
            PinId::new(4),
            "sum",
            PinDirection::Output,
            SignalWidth::SINGLE,
            Point::new(400.0, 100.0),
        ));

        // Tambah XOR component
        let xor_def = reg.get("dlf.builtin.xor").unwrap();
        let xor_inst = xor_def
            .instantiate(
                InstanceId::new(6),
                &[PinId::new(7), PinId::new(8), PinId::new(9)],
                Point::new(220.0, 100.0),
            )
            .unwrap();
        root.add_component(xor_inst);

        // Tambah Junction
        root.add_junction(Junction::new(JunctionId::new(14), Point::new(120.0, 100.0)));

        // Tambah Wires
        root.add_wire(WireSegment::new(
            WireId::new(101),
            Endpoint::Port { id: PinId::new(2) },
            Endpoint::Junction {
                id: JunctionId::new(14),
            },
        ));

        let json = to_json_pretty(&project).expect("Serialisasi pretty harus berhasil");
        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"Half Adder Test\""));
        assert!(json.contains("\"dlf.builtin.xor\""));
        assert!(json.contains("\"junctions\""));
        assert!(json.contains("\"wires\""));

        let compact_json = to_json_string(&project).expect("Serialisasi kompak harus berhasil");
        assert!(!compact_json.contains('\n'));
    }
}
