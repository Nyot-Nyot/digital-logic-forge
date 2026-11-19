//! # Mesin Serialisasi JSON Dokumen (JSON Serialization Engine)
//!
//! Modul ini menyediakan fungsi-fungsi tingkat tinggi untuk mengubah model dokumen rangkaian
//! (`Project`) menjadi format teks JSON yang terstandarisasi sesuai Bab 3.10.1 proposal.

use super::project::{CURRENT_SCHEMA_VERSION, Project};
use crate::error::CoreError;

/// Mem-parse string JSON menjadi model dokumen proyek (`Project`) dengan validasi versi skema.
///
/// Fungsi ini memeriksa kecocokan `schema_version`. Jika skema tidak didukung,
/// fungsi akan mengembalikan `CoreError::ValidationError` deskriptif.
/// Setelah parsing berhasil, penghitung `IdGenerator` internal otomatis disinkronkan.
///
/// # Errors
/// Mengembalikan `CoreError::ValidationError` jika sintaks JSON tidak valid atau versi skema tidak cocok.
pub fn from_json_str(json_str: &str) -> Result<Project, CoreError> {
    let mut project: Project = serde_json::from_str(json_str).map_err(|err| {
        CoreError::ValidationError(format!("Gagal mem-parse JSON dokumen proyek: {}", err))
    })?;

    if project.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(CoreError::ValidationError(format!(
            "Versi skema dokumen tidak kompatibel: skema berkas adalah v{}, sedangkan simulator mendukung v{}",
            project.schema_version, CURRENT_SCHEMA_VERSION
        )));
    }

    project.sync_id_generator();
    Ok(project)
}

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

        // Deserialisasi kembali dan verifikasi
        let loaded = from_json_str(&json).expect("Deserialisasi harus berhasil");
        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.name, "Half Adder Test");
        assert_eq!(loaded.circuits.len(), 1);
        let loaded_root = loaded.get_root_circuit().unwrap();
        assert_eq!(loaded_root.ports.len(), 3);
        assert_eq!(loaded_root.components.len(), 1);
        assert_eq!(loaded_root.junctions.len(), 1);
        assert_eq!(loaded_root.wires.len(), 1);
    }

    #[test]
    fn test_deserialize_incompatible_schema_version() {
        let invalid_json = r#"{
            "schema_version": 99,
            "name": "Future Schema Project",
            "next_id": 10,
            "root_circuit_id": 1,
            "circuits": []
        }"#;

        let res = from_json_str(invalid_json);
        assert!(res.is_err());
        let err_msg = format!("{}", res.unwrap_err());
        assert!(err_msg.contains("tidak kompatibel"));
        assert!(err_msg.contains("v99"));
    }

    #[test]
    fn test_deserialize_malformed_json() {
        let malformed = "{ this is not valid json }";
        let res = from_json_str(malformed);
        assert!(res.is_err());
        let err_msg = format!("{}", res.unwrap_err());
        assert!(err_msg.contains("Gagal mem-parse JSON"));
    }
}
