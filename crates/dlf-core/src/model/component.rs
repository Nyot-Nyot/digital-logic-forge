//! # Definisi Komponen, Instansiasi, dan Registry Pustaka Primitif
//!
//! Modul ini mengimplementasikan kontrak definisi komponen (`ComponentDefinition`),
//! instansiasi komponen di canvas (`ComponentInstance`), serta pustaka primitif bawaan
//! (`ComponentRegistry`) sesuai Bab 3.10.2 dan Tabel 3.22–3.23 proposal.

use super::geometry::{Point, Rotation};
use super::id::{InstanceId, PinId};
use super::pin::{PinDeclaration, PinDirection, PinInstance, PinRole, SignalWidth};
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

/// Klasifikasi perilaku fungsional komponen dalam rantai kompilasi dan simulasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorKind {
    /// Rangkaian kombinasional murni (output seketika bergantung input).
    Combinational,
    /// Rangkaian sekuensial dengan state tersimpan pada active clock edge.
    Sequential,
    /// Subsistem memori dengan blok data besar (RAM/ROM).
    Memory,
    /// Komponen struktural (misal splitter/merger atau sub-sirkuit chip).
    Structural,
    /// Titik masukan/keluaran eksternal (switch, LED, probe).
    IO,
}

impl Display for BehaviorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Combinational => write!(f, "Combinational"),
            Self::Sequential => write!(f, "Sequential"),
            Self::Memory => write!(f, "Memory"),
            Self::Structural => write!(f, "Structural"),
            Self::IO => write!(f, "IO"),
        }
    }
}

/// Jenis implementasi komponen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplementationKind {
    /// Primitif yang dievaluasi langsung oleh instruksi IR runtime bawaan mesin.
    NativePrimitive,
    /// Komposit/hierarki yang diekspansi menjadi netlist komponen anak.
    CompositeCircuit,
}

/// Nilai parameter dinamis suatu komponen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// Nilai integer 64-bit (misal jumlah input, lebar bit).
    Integer(i64),
    /// Nilai boolean flag.
    Boolean(bool),
    /// Nilai teks / string (misal jalur file ROM image).
    String(String),
}

impl Display for ParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{}", i),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Deklarasi parameter statis pada definisi komponen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDeclaration {
    /// Nama pengenal unik parameter (misal `"inputs"`, `"width"`).
    pub name: String,
    /// Deskripsi penjelasan parameter untuk inspector UI.
    pub description: String,
    /// Nilai bawaan (*default value*).
    pub default_value: ParameterValue,
}

impl ParameterDeclaration {
    /// Membuat deklarasi parameter baru.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        default_value: ParameterValue,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            default_value,
        }
    }
}

/// Kontrak definisi komponen yang terdaftar dalam pustaka (*library component*).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDefinition {
    /// Pengenal stabil independen nama tampilan (misal `"dlf.builtin.and"`).
    pub definition_id: String,
    /// Versi skema kontrak komponen untuk migrasi format berkas.
    pub schema_version: u32,
    /// Nama tampilan pada palet/library editor.
    pub display_name: String,
    /// Kategori komponen (misal `"Gates"`, `"Memory"`, `"Plexers"`).
    pub category: String,
    /// Karakteristik perilaku eksekusi simulasi.
    pub behavior_kind: BehaviorKind,
    /// Bentuk implementasi (native primitive vs composite chip).
    pub implementation_kind: ImplementationKind,
    /// Daftar antarmuka pin yang dimiliki komponen.
    pub pins: Vec<PinDeclaration>,
    /// Daftar parameter yang dapat dikonfigurasi.
    pub parameters: Vec<ParameterDeclaration>,
}

impl ComponentDefinition {
    /// Membuat builder baru untuk mengonstruksi `ComponentDefinition` secara bertahap dan terbaca.
    pub fn builder(
        definition_id: impl Into<String>,
        schema_version: u32,
    ) -> ComponentDefinitionBuilder {
        ComponentDefinitionBuilder::new(definition_id, schema_version)
    }

    /// Menginstansiasi definisi ini menjadi `ComponentInstance` dengan alokasi PinId konkret.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::ValidationError` jika jumlah `pin_ids` tidak sama dengan
    /// jumlah pin yang dideklarasikan.
    pub fn instantiate(
        &self,
        instance_id: InstanceId,
        pin_ids: &[PinId],
        position: Point,
    ) -> Result<ComponentInstance, CoreError> {
        if pin_ids.len() != self.pins.len() {
            return Err(CoreError::ValidationError(format!(
                "Jumlah PinId ({}) tidak cocok dengan deklarasi pin komponen '{}' ({})",
                pin_ids.len(),
                self.definition_id,
                self.pins.len()
            )));
        }

        let pin_instances = self
            .pins
            .iter()
            .zip(pin_ids.iter())
            .map(|(decl, &pid)| {
                PinInstance::new(pid, &decl.id, decl.direction, decl.default_width, decl.role)
            })
            .collect();

        let mut default_params = BTreeMap::new();
        for p in &self.parameters {
            default_params.insert(p.name.clone(), p.default_value.clone());
        }

        Ok(ComponentInstance {
            id: instance_id,
            definition_id: self.definition_id.clone(),
            schema_version: self.schema_version,
            label: None,
            position,
            rotation: Rotation::default(),
            mirror_x: false,
            mirror_y: false,
            parameters: default_params,
            pins: pin_instances,
        })
    }
}

/// Builder untuk mengonstruksi `ComponentDefinition` dengan antarmuka yang bersih dan deskriptif.
#[derive(Debug, Clone)]
pub struct ComponentDefinitionBuilder {
    definition_id: String,
    schema_version: u32,
    display_name: String,
    category: String,
    behavior_kind: BehaviorKind,
    implementation_kind: ImplementationKind,
    pins: Vec<PinDeclaration>,
    parameters: Vec<ParameterDeclaration>,
}

impl ComponentDefinitionBuilder {
    /// Inisialisasi builder dengan ID definisi dan versi skema dasar.
    pub fn new(definition_id: impl Into<String>, schema_version: u32) -> Self {
        Self {
            definition_id: definition_id.into(),
            schema_version,
            display_name: String::new(),
            category: "General".to_string(),
            behavior_kind: BehaviorKind::Combinational,
            implementation_kind: ImplementationKind::NativePrimitive,
            pins: Vec::new(),
            parameters: Vec::new(),
        }
    }

    /// Menentukan nama tampilan untuk editor/palet.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Menentukan kategori palet komponen.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Menentukan jenis perilaku komponen.
    pub fn behavior_kind(mut self, kind: BehaviorKind) -> Self {
        self.behavior_kind = kind;
        self
    }

    /// Menentukan jenis implementasi komponen.
    pub fn implementation_kind(mut self, kind: ImplementationKind) -> Self {
        self.implementation_kind = kind;
        self
    }

    /// Menambahkan satu deklarasi pin ke komponen.
    pub fn pin(mut self, pin: PinDeclaration) -> Self {
        self.pins.push(pin);
        self
    }

    /// Menetapkan daftar seluruh pin komponen.
    pub fn pins(mut self, pins: Vec<PinDeclaration>) -> Self {
        self.pins = pins;
        self
    }

    /// Menambahkan satu deklarasi parameter ke komponen.
    pub fn parameter(mut self, param: ParameterDeclaration) -> Self {
        self.parameters.push(param);
        self
    }

    /// Menetapkan daftar seluruh parameter komponen.
    pub fn parameters(mut self, params: Vec<ParameterDeclaration>) -> Self {
        self.parameters = params;
        self
    }

    /// Menyelesaikan konstruksi dan menghasilkan instance `ComponentDefinition`.
    pub fn build(self) -> ComponentDefinition {
        ComponentDefinition {
            definition_id: self.definition_id,
            schema_version: self.schema_version,
            display_name: self.display_name,
            category: self.category,
            behavior_kind: self.behavior_kind,
            implementation_kind: self.implementation_kind,
            pins: self.pins,
            parameters: self.parameters,
        }
    }
}

/// Instansiasi komponen konkret yang diletakkan pada lembar rangkaian (*Authoring Document*).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInstance {
    /// Stable ID 64-bit unik instance ini.
    pub id: InstanceId,
    /// ID definisi pustaka yang dirujuk.
    pub definition_id: String,
    /// Versi skema definisi saat instansiasi.
    pub schema_version: u32,
    /// Label opsional yang diberikan pengguna di visual editor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Koordinat posisi penempatan di canvas.
    pub position: Point,
    /// Sudut orientasi rotasi.
    pub rotation: Rotation,
    /// Pencerminan horizontal.
    pub mirror_x: bool,
    /// Pencerminan vertikal.
    pub mirror_y: bool,
    /// Konfigurasi parameter konkret instance.
    pub parameters: BTreeMap<String, ParameterValue>,
    /// Daftar pin konkret instance ini.
    pub pins: Vec<PinInstance>,
}

impl ComponentInstance {
    /// Mencari `PinInstance` berdasarkan `declaration_id` (misal `"a"`, `"b"`, `"y"`).
    pub fn get_pin_by_declaration(&self, decl_id: &str) -> Option<&PinInstance> {
        self.pins.iter().find(|p| p.declaration_id == decl_id)
    }

    /// Mencari `PinInstance` berdasarkan `PinId` konkret.
    pub fn get_pin_by_id(&self, pin_id: PinId) -> Option<&PinInstance> {
        self.pins.iter().find(|p| p.id == pin_id)
    }
}

/// Pustaka terpusat (*registry*) penyimpan definisi komponen bawaan dan kustom.
#[derive(Debug, Clone, Default)]
pub struct ComponentRegistry {
    definitions: BTreeMap<String, ComponentDefinition>,
}

impl ComponentRegistry {
    /// Membuat registry kosong.
    pub fn new() -> Self {
        Self {
            definitions: BTreeMap::new(),
        }
    }

    /// Membuat registry lengkap yang terisi seluruh komponen primitif bawaan L0 dan L1.
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        reg.register_builtins();
        reg
    }

    /// Mendaftarkan definisi komponen baru ke dalam registry.
    pub fn register(&mut self, definition: ComponentDefinition) {
        self.definitions
            .insert(definition.definition_id.clone(), definition);
    }

    /// Mengambil referensi definisi komponen berdasarkan `definition_id`.
    pub fn get(&self, definition_id: &str) -> Option<&ComponentDefinition> {
        self.definitions.get(definition_id)
    }

    /// Memeriksa apakah suatu `definition_id` terdaftar di dalam registry.
    pub fn contains(&self, definition_id: &str) -> bool {
        self.definitions.contains_key(definition_id)
    }

    /// Jumlah definisi komponen yang terdaftar.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Memeriksa apakah registry kosong.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Mendaftarkan pustaka komponen primitif bawaan (L0, L1, L2 dasar).
    fn register_builtins(&mut self) {
        // Gerbang Logika Dasar 2-Masukan (AND, OR, XOR, NAND, NOR, XNOR)
        let basic_gates = [
            ("dlf.builtin.and", "AND Gate"),
            ("dlf.builtin.or", "OR Gate"),
            ("dlf.builtin.xor", "XOR Gate"),
            ("dlf.builtin.nand", "NAND Gate"),
            ("dlf.builtin.nor", "NOR Gate"),
            ("dlf.builtin.xnor", "XNOR Gate"),
        ];

        for (id, display) in basic_gates {
            self.register(
                ComponentDefinition::builder(id, 1)
                    .display_name(display)
                    .category("Gates")
                    .behavior_kind(BehaviorKind::Combinational)
                    .implementation_kind(ImplementationKind::NativePrimitive)
                    .pins(vec![
                        PinDeclaration::new(
                            "a",
                            "Input A",
                            PinDirection::Input,
                            SignalWidth::SINGLE,
                            PinRole::Signal,
                            true,
                        ),
                        PinDeclaration::new(
                            "b",
                            "Input B",
                            PinDirection::Input,
                            SignalWidth::SINGLE,
                            PinRole::Signal,
                            true,
                        ),
                        PinDeclaration::new(
                            "y",
                            "Output Y",
                            PinDirection::Output,
                            SignalWidth::SINGLE,
                            PinRole::Signal,
                            false,
                        ),
                    ])
                    .parameters(vec![
                        ParameterDeclaration::new(
                            "width",
                            "Lebar bit data",
                            ParameterValue::Integer(1),
                        ),
                        ParameterDeclaration::new(
                            "inputs",
                            "Jumlah pin masukan",
                            ParameterValue::Integer(2),
                        ),
                    ])
                    .build(),
            );
        }

        // NOT Gate & Buffer
        for (id, display) in [
            ("dlf.builtin.not", "NOT Gate"),
            ("dlf.builtin.buf", "Buffer"),
        ] {
            self.register(
                ComponentDefinition::builder(id, 1)
                    .display_name(display)
                    .category("Gates")
                    .behavior_kind(BehaviorKind::Combinational)
                    .implementation_kind(ImplementationKind::NativePrimitive)
                    .pins(vec![
                        PinDeclaration::new(
                            "a",
                            "Input A",
                            PinDirection::Input,
                            SignalWidth::SINGLE,
                            PinRole::Signal,
                            true,
                        ),
                        PinDeclaration::new(
                            "y",
                            "Output Y",
                            PinDirection::Output,
                            SignalWidth::SINGLE,
                            PinRole::Signal,
                            false,
                        ),
                    ])
                    .parameter(ParameterDeclaration::new(
                        "width",
                        "Lebar bit data",
                        ParameterValue::Integer(1),
                    ))
                    .build(),
            );
        }

        // D Flip-Flop (DFF) / Register Sekuensial
        self.register(
            ComponentDefinition::builder("dlf.builtin.dff", 1)
                .display_name("D Flip-Flop")
                .category("Memory")
                .behavior_kind(BehaviorKind::Sequential)
                .implementation_kind(ImplementationKind::NativePrimitive)
                .pins(vec![
                    PinDeclaration::new(
                        "d",
                        "Data In",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Data,
                        true,
                    ),
                    PinDeclaration::new(
                        "clk",
                        "Clock",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Clock,
                        true,
                    ),
                    PinDeclaration::new(
                        "en",
                        "Enable",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Enable,
                        false,
                    ),
                    PinDeclaration::new(
                        "rst",
                        "Reset",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Reset,
                        false,
                    ),
                    PinDeclaration::new(
                        "q",
                        "Data Out",
                        PinDirection::Output,
                        SignalWidth::SINGLE,
                        PinRole::Data,
                        false,
                    ),
                ])
                .parameter(ParameterDeclaration::new(
                    "width",
                    "Lebar bit data register",
                    ParameterValue::Integer(1),
                ))
                .build(),
        );

        // Multiplexer (MUX)
        self.register(
            ComponentDefinition::builder("dlf.builtin.mux", 1)
                .display_name("Multiplexer")
                .category("Plexers")
                .behavior_kind(BehaviorKind::Combinational)
                .implementation_kind(ImplementationKind::NativePrimitive)
                .pins(vec![
                    PinDeclaration::new(
                        "d0",
                        "Data 0",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Data,
                        true,
                    ),
                    PinDeclaration::new(
                        "d1",
                        "Data 1",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Data,
                        true,
                    ),
                    PinDeclaration::new(
                        "sel",
                        "Select",
                        PinDirection::Input,
                        SignalWidth::SINGLE,
                        PinRole::Signal,
                        true,
                    ),
                    PinDeclaration::new(
                        "y",
                        "Output Y",
                        PinDirection::Output,
                        SignalWidth::SINGLE,
                        PinRole::Data,
                        false,
                    ),
                ])
                .parameters(vec![
                    ParameterDeclaration::new(
                        "width",
                        "Lebar bit data",
                        ParameterValue::Integer(1),
                    ),
                    ParameterDeclaration::new(
                        "select_bits",
                        "Lebar bit pemilih",
                        ParameterValue::Integer(1),
                    ),
                ])
                .build(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry_lookup() {
        let registry = ComponentRegistry::with_builtins();

        assert!(registry.contains("dlf.builtin.and"));
        assert!(registry.contains("dlf.builtin.or"));
        assert!(registry.contains("dlf.builtin.xor"));
        assert!(registry.contains("dlf.builtin.not"));
        assert!(registry.contains("dlf.builtin.dff"));
        assert!(registry.contains("dlf.builtin.mux"));
        assert!(!registry.contains("dlf.nonexistent"));

        let and_def = registry.get("dlf.builtin.and").unwrap();
        assert_eq!(and_def.pins.len(), 3);
        assert_eq!(and_def.behavior_kind, BehaviorKind::Combinational);
    }

    #[test]
    fn test_component_instantiation() {
        let registry = ComponentRegistry::with_builtins();
        let xor_def = registry.get("dlf.builtin.xor").unwrap();

        let pin_ids = [PinId::new(10), PinId::new(11), PinId::new(12)];
        let instance = xor_def
            .instantiate(InstanceId::new(1), &pin_ids, Point::new(100.0, 50.0))
            .expect("Instansiasi harus berhasil");

        assert_eq!(instance.id.get(), 1);
        assert_eq!(instance.definition_id, "dlf.builtin.xor");
        assert_eq!(instance.pins.len(), 3);
        assert_eq!(instance.position.x, 100.0);
        assert_eq!(instance.position.y, 50.0);

        let pin_a = instance.get_pin_by_declaration("a").unwrap();
        assert_eq!(pin_a.id.get(), 10);
        assert_eq!(pin_a.direction, PinDirection::Input);

        let pin_y = instance.get_pin_by_declaration("y").unwrap();
        assert_eq!(pin_y.id.get(), 12);
        assert_eq!(pin_y.direction, PinDirection::Output);
    }

    #[test]
    fn test_instantiation_pin_count_mismatch() {
        let registry = ComponentRegistry::with_builtins();
        let and_def = registry.get("dlf.builtin.and").unwrap();

        let pin_ids = [PinId::new(10)]; // Kurang (harus 3)
        let res = and_def.instantiate(InstanceId::new(1), &pin_ids, Point::ZERO);
        assert!(res.is_err());
    }
}
