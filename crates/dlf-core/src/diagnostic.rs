//! # Kerangka Kerja Diagnostik Rangkaian (Diagnostic Framework)
//!
//! Modul ini mengimplementasikan model diagnostik terstruktur untuk pelaporan error, peringatan,
//! dan informasi sesuai Bab 3.10.8 dan Tabel 3.31 proposal.
//!
//! Setiap diagnostik membawa kode stabil (`DiagnosticCode`), pesan penjelasan, tahap kompilasi/eksekusi,
//! serta referensi objek grafis utama (`primary`) dan objek terkait (`related`) untuk keperluan
//! *highlighting* interaktif pada canvas editor visual (KNF-03).

use crate::model::id::{CircuitId, InstanceId, JunctionId, PinId, TunnelId, WireId};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Tingkat keparahan diagnostik.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Informasi informatif (tidak menghalangi kompilasi).
    Info,
    /// Peringatan potensi masalah (tidak menghalangi pembuatan artefak simulasi).
    Warning,
    /// Kesalahan fatal (menghalangi kompilasi dan eksekusi simulasi).
    Error,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Kode kesalahan stabil yang dapat dilacak secara terprogram dan dipetakan ke dokumentasi/UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticCode {
    /// Pin masukan wajib tidak terhubung ke net atau driver mana pun (`CONN-PIN-UNCONNECTED`).
    #[serde(rename = "CONN-PIN-UNCONNECTED")]
    ConnPinUnconnected,

    /// Segmen kawat memiliki ujung yang menggantung bebas tanpa tersambung (`CONN-WIRE-DANGLING`).
    #[serde(rename = "CONN-WIRE-DANGLING")]
    ConnWireDangling,

    /// Ketidakcocokan lebar bit antara dua terminal yang terhubung (`CONN-WIDTH-MISMATCH`).
    #[serde(rename = "CONN-WIDTH-MISMATCH")]
    ConnWidthMismatch,

    /// Label tunnel tidak memiliki pasangan dengan nama dan cakupan yang sama (`CONN-TUNNEL-UNPAIRED`).
    #[serde(rename = "CONN-TUNNEL-UNPAIRED")]
    ConnTunnelUnpaired,

    /// Pemetaan bit splitter atau merger tidak valid (`CONN-SPLIT-INVALID`).
    #[serde(rename = "CONN-SPLIT-INVALID")]
    ConnSplitInvalid,

    /// Beberapa driver aktif pada net yang tidak diresolusi (`NET-MULTI-DRIVER`).
    #[serde(rename = "NET-MULTI-DRIVER")]
    NetMultiDriver,

    /// Net listrik tidak memiliki sumber penggerak sama sekali (`NET-UNDRIVEN`).
    #[serde(rename = "NET-UNDRIVEN")]
    NetUndriven,

    /// Terjadi siklus instansiasi sirkuit memanggil dirinya sendiri (`HIER-RECURSION`).
    #[serde(rename = "HIER-RECURSION")]
    HierRecursion,

    /// Wilayah umpan balik kombinasional berosilasi tanpa konvergensi (`DEP-SCC-OSCILLATION`).
    #[serde(rename = "DEP-SCC-OSCILLATION")]
    DepSccOscillation,
}

impl DiagnosticCode {
    /// Mengembalikan representasi teks kanonis stabil dari kode diagnostik.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConnPinUnconnected => "CONN-PIN-UNCONNECTED",
            Self::ConnWireDangling => "CONN-WIRE-DANGLING",
            Self::ConnWidthMismatch => "CONN-WIDTH-MISMATCH",
            Self::ConnTunnelUnpaired => "CONN-TUNNEL-UNPAIRED",
            Self::ConnSplitInvalid => "CONN-SPLIT-INVALID",
            Self::NetMultiDriver => "NET-MULTI-DRIVER",
            Self::NetUndriven => "NET-UNDRIVEN",
            Self::HierRecursion => "HIER-RECURSION",
            Self::DepSccOscillation => "DEP-SCC-OSCILLATION",
        }
    }
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Tahapan alur kerja tempat diagnostik dihasilkan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticStage {
    /// Validasi struktural model dokumen tanpa kompilasi penuh.
    ModelValidation,
    /// Resolusi netlist dan penelusuran graf konektivitas.
    Connectivity,
    /// Elaborasi dan perataan (*flattening*) hierarki sub-sirkuit.
    Elaboration,
    /// Penjadwalan urutan level ketergantungan dan analisis SCC.
    Scheduling,
    /// Penerjemahan ke instruksi IR runtime.
    Lowering,
    /// Terjadi saat eksekusi worker simulasi berjalan.
    Runtime,
}

impl Display for DiagnosticStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelValidation => write!(f, "ModelValidation"),
            Self::Connectivity => write!(f, "Connectivity"),
            Self::Elaboration => write!(f, "Elaboration"),
            Self::Scheduling => write!(f, "Scheduling"),
            Self::Lowering => write!(f, "Lowering"),
            Self::Runtime => write!(f, "Runtime"),
        }
    }
}

/// Rujukan ke entitas grafis visual di canvas untuk penyorotan (*highlighting*) lokasi error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "ref_type")]
pub enum SourceRef {
    /// Menunjuk definisi sirkuit tertentu.
    Circuit {
        /// ID sirkuit.
        id: CircuitId,
    },
    /// Menunjuk instance komponen di canvas.
    Instance {
        /// ID instance komponen.
        id: InstanceId,
    },
    /// Menunjuk pin terminal pada suatu komponen.
    Pin {
        /// ID instance komponen pemilik pin.
        instance_id: InstanceId,
        /// ID pin konkret yang disorot.
        pin_id: PinId,
    },
    /// Menunjuk port batas antarmuka sirkuit.
    Port {
        /// ID port antarmuka.
        id: PinId,
    },
    /// Menunjuk segmen kawat penghubung.
    Wire {
        /// ID segmen kawat.
        id: WireId,
    },
    /// Menunjuk titik cabang kawat (*junction*).
    Junction {
        /// ID titik junction.
        id: JunctionId,
    },
    /// Menunjuk entitas label tunnel.
    Tunnel {
        /// ID entitas tunnel.
        id: TunnelId,
    },
}

impl Display for SourceRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Circuit { id } => write!(f, "Circuit({})", id),
            Self::Instance { id } => write!(f, "Instance({})", id),
            Self::Pin {
                instance_id,
                pin_id,
            } => write!(f, "Pin({}:{})", instance_id, pin_id),
            Self::Port { id } => write!(f, "Port({})", id),
            Self::Wire { id } => write!(f, "Wire({})", id),
            Self::Junction { id } => write!(f, "Junction({})", id),
            Self::Tunnel { id } => write!(f, "Tunnel({})", id),
        }
    }
}

/// Laporan diagnostik lengkap untuk umpan balik ke pengguna dan UI editor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Tingkat keparahan (Error, Warning, Info).
    pub severity: Severity,
    /// Kode kesalahan kanonis stabil.
    pub code: DiagnosticCode,
    /// Pesan deskriptif yang mudah dipahami manusia.
    pub message: String,
    /// Tahap tempat diagnostik terdeteksi.
    pub stage: DiagnosticStage,
    /// Objek visual utama yang menjadi fokus penyorotan.
    pub primary: SourceRef,
    /// Daftar objek visual terkait lainnya yang ikut terlibat dalam anomali ini.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<SourceRef>,
}

impl Diagnostic {
    /// Membuat diagnostik bertingkat kesalahan fatal (*Error*).
    pub fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        stage: DiagnosticStage,
        primary: SourceRef,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            stage,
            primary,
            related: Vec::new(),
        }
    }

    /// Membuat diagnostik bertingkat peringatan (*Warning*).
    pub fn warning(
        code: DiagnosticCode,
        message: impl Into<String>,
        stage: DiagnosticStage,
        primary: SourceRef,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            message: message.into(),
            stage,
            primary,
            related: Vec::new(),
        }
    }

    /// Menambahkan daftar objek visual terkait (*related sources*).
    pub fn with_related(mut self, related: Vec<SourceRef>) -> Self {
        self.related = related;
        self
    }

    /// Memeriksa apakah diagnostik ini merupakan kesalahan fatal (*Error*).
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] [{}] {}: {} (target: {})",
            self.stage, self.severity, self.code, self.message, self.primary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation_and_formatting() {
        let diag = Diagnostic::error(
            DiagnosticCode::ConnPinUnconnected,
            "Pin masukan wajib 'a' belum tersambung ke kawat mana pun",
            DiagnosticStage::ModelValidation,
            SourceRef::Pin {
                instance_id: InstanceId::new(5),
                pin_id: PinId::new(6),
            },
        )
        .with_related(vec![SourceRef::Instance {
            id: InstanceId::new(5),
        }]);

        assert!(diag.is_error());
        assert_eq!(diag.code.as_str(), "CONN-PIN-UNCONNECTED");
        assert_eq!(diag.related.len(), 1);

        let diag_str = format!("{}", diag);
        assert!(diag_str.contains("CONN-PIN-UNCONNECTED"));
        assert!(diag_str.contains("Pin(5:6)"));
    }

    #[test]
    fn test_diagnostic_serde_roundtrip() {
        let diag = Diagnostic::warning(
            DiagnosticCode::ConnWireDangling,
            "Segmen kawat menggantung tanpa tersambung",
            DiagnosticStage::ModelValidation,
            SourceRef::Wire {
                id: WireId::new(42),
            },
        );

        let json = serde_json::to_string(&diag).expect("Gagal serialize Diagnostic");
        let decoded: Diagnostic =
            serde_json::from_str(&json).expect("Gagal deserialize Diagnostic");

        assert_eq!(diag, decoded);
    }
}
