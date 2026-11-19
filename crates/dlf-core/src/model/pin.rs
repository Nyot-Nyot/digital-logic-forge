//! # Kontrak Pin, Arah Sinyal, dan Lebar Bus (Pin Contract & Bus Width)
//!
//! Modul ini mendefinisikan antarmuka dan kontrak pin komponen digital sesuai Bab 3.10.2 proposal.
//! Mencakup arah pin (`PinDirection`), peran logika (`PinRole`), lebar bus aman (`SignalWidth` 1..=64),
//! deklarasi pin pada pustaka komponen (`PinDeclaration`), serta instansiasi pin konkret (`PinInstance`).

use super::id::PinId;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Batas minimum lebar bus sinyal (1-bit / skalar).
pub const MIN_SIGNAL_WIDTH: u8 = 1;

/// Batas maksimum lebar bus sinyal yang didukung dalam 1 machine word 64-bit (Bab 3.10.4).
pub const MAX_SIGNAL_WIDTH: u8 = 64;

/// Lebar bus sinyal digital yang divalidasi dengan aman dalam rentang 1 hingga 64 bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SignalWidth(u8);

impl SignalWidth {
    /// Sinyal standar 1-bit (skalar).
    pub const SINGLE: Self = Self(1);

    /// Membuat instance `SignalWidth` baru setelah memvalidasi rentang 1..=64.
    ///
    /// # Errors
    /// Mengembalikan `CoreError::ValidationError` jika lebar di luar rentang `1..=64`.
    pub fn new(width: u8) -> Result<Self, CoreError> {
        if (MIN_SIGNAL_WIDTH..=MAX_SIGNAL_WIDTH).contains(&width) {
            Ok(Self(width))
        } else {
            Err(CoreError::ValidationError(format!(
                "Lebar sinyal tidak valid: {} bit (harus di antara {} dan {} bit)",
                width, MIN_SIGNAL_WIDTH, MAX_SIGNAL_WIDTH
            )))
        }
    }

    /// Mengembalikan nilai lebar bit mentah (`u8`).
    #[inline]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Menghasilkan bitmask `u64` sesuai dengan lebar sinyal.
    ///
    /// Contoh:
    /// - 1 bit  => `0x0000_0000_0000_0001`
    /// - 8 bit  => `0x0000_0000_0000_00FF`
    /// - 64 bit => `0xFFFF_FFFF_FFFF_FFFF`
    #[inline]
    pub const fn mask(self) -> u64 {
        if self.0 >= 64 {
            u64::MAX
        } else {
            (1u64 << self.0) - 1
        }
    }
}

impl Display for SignalWidth {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} bit", self.0)
    }
}

impl TryFrom<u8> for SignalWidth {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SignalWidth> for u8 {
    #[inline]
    fn from(width: SignalWidth) -> Self {
        width.0
    }
}

/// Arah aliran sinyal pada terminal pin komponen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PinDirection {
    /// Masukan sinyal (konsumen nilai).
    Input,
    /// Keluaran sinyal (penghasil/penggerak nilai tunggal).
    Output,
    /// Dua arah / bidirectional (hanya untuk bus yang diresolusi).
    Inout,
}

impl Display for PinDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "Input"),
            Self::Output => write!(f, "Output"),
            Self::Inout => write!(f, "Inout"),
        }
    }
}

/// Peran semantik fungsional dari suatu pin pada logika rangkaian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PinRole {
    /// Sinyal data atau logika umum.
    Signal,
    /// Sinyal pemicu tepi waktu (*active clock edge*).
    Clock,
    /// Sinyal inisialisasi / reset keadaan sekuensial.
    Reset,
    /// Sinyal aktivasi / enable operasional.
    Enable,
    /// Bus alamat untuk memori atau multiplexer.
    Address,
    /// Bus data untuk memori atau register.
    Data,
}

impl Display for PinRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signal => write!(f, "Signal"),
            Self::Clock => write!(f, "Clock"),
            Self::Reset => write!(f, "Reset"),
            Self::Enable => write!(f, "Enable"),
            Self::Address => write!(f, "Address"),
            Self::Data => write!(f, "Data"),
        }
    }
}

/// Deklarasi kontrak pin statis pada definisi komponen (`ComponentDefinition`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinDeclaration {
    /// Pengenal unik internal pin dalam definisi komponen (misalnya `"a"`, `"b"`, `"y"`).
    pub id: String,
    /// Nama tampilan visual untuk editor / inspector.
    pub display_name: String,
    /// Arah aliran sinyal pin.
    pub direction: PinDirection,
    /// Lebar sinyal bawaan sebelum parameterisasi.
    pub default_width: SignalWidth,
    /// Peran fungsional logika pin.
    pub role: PinRole,
    /// Menandakan apakah pin masukan wajib disambungkan (tidak boleh dibiarkan mengambang).
    pub is_mandatory: bool,
}

impl PinDeclaration {
    /// Membuat deklarasi pin baru.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        direction: PinDirection,
        default_width: SignalWidth,
        role: PinRole,
        is_mandatory: bool,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            direction,
            default_width,
            role,
            is_mandatory,
        }
    }
}

/// Instansiasi pin konkret yang melekat pada suatu komponen di canvas (`ComponentInstance`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinInstance {
    /// Stable ID 64-bit unik dari pin konkret ini.
    pub id: PinId,
    /// ID deklarasi pin yang dirujuk dari `PinDeclaration`.
    pub declaration_id: String,
    /// Arah aliran sinyal pin.
    pub direction: PinDirection,
    /// Lebar sinyal pin yang telah terkonfigurasi.
    pub width: SignalWidth,
    /// Peran semantik fungsional pin.
    pub role: PinRole,
}

impl PinInstance {
    /// Membuat instance pin konkret baru.
    pub fn new(
        id: PinId,
        declaration_id: impl Into<String>,
        direction: PinDirection,
        width: SignalWidth,
        role: PinRole,
    ) -> Self {
        Self {
            id,
            declaration_id: declaration_id.into(),
            direction,
            width,
            role,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_width_valid_range() {
        assert!(SignalWidth::new(0).is_err());
        assert!(SignalWidth::new(1).is_ok());
        assert!(SignalWidth::new(8).is_ok());
        assert!(SignalWidth::new(64).is_ok());
        assert!(SignalWidth::new(65).is_err());
    }

    #[test]
    fn test_signal_width_mask() {
        let w1 = SignalWidth::new(1).unwrap();
        assert_eq!(w1.mask(), 0x1);

        let w4 = SignalWidth::new(4).unwrap();
        assert_eq!(w4.mask(), 0xF);

        let w8 = SignalWidth::new(8).unwrap();
        assert_eq!(w8.mask(), 0xFF);

        let w16 = SignalWidth::new(16).unwrap();
        assert_eq!(w16.mask(), 0xFFFF);

        let w64 = SignalWidth::new(64).unwrap();
        assert_eq!(w64.mask(), u64::MAX);
    }

    #[test]
    fn test_pin_declaration_and_instance() {
        let decl = PinDeclaration::new(
            "din",
            "Data In",
            PinDirection::Input,
            SignalWidth::new(8).unwrap(),
            PinRole::Data,
            true,
        );

        assert_eq!(decl.id, "din");
        assert_eq!(decl.direction, PinDirection::Input);
        assert!(decl.is_mandatory);

        let inst = PinInstance::new(
            PinId::new(42),
            "din",
            PinDirection::Input,
            SignalWidth::new(8).unwrap(),
            PinRole::Data,
        );

        assert_eq!(inst.id.get(), 42);
        assert_eq!(inst.declaration_id, "din");
        assert_eq!(inst.width.get(), 8);
    }

    #[test]
    fn test_serde_pin_instance_roundtrip() {
        let inst = PinInstance::new(
            PinId::new(100),
            "clk",
            PinDirection::Input,
            SignalWidth::SINGLE,
            PinRole::Clock,
        );

        let json = serde_json::to_string(&inst).expect("Gagal serialize PinInstance");
        let decoded: PinInstance =
            serde_json::from_str(&json).expect("Gagal deserialize PinInstance");

        assert_eq!(inst, decoded);
    }
}
