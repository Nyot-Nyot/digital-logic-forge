//! # Representasi Nilai Logika Empat-Nilai (0, 1, X, Z)
//!
//! Modul ini mendefinisikan `LogicValue` sebagai wadah antarmuka untuk nilai sinyal
//! hingga 64-bit dengan dukungan empat keadaan logika sesuai `[SPIKE-004]`:
//! - `unk = 0, val = 0` → `0` (Low)
//! - `unk = 0, val = 1` → `1` (High)
//! - `unk = 1, val = 0` → `X` (Unknown / Konflik)
//! - `unk = 1, val = 1` → `Z` (High-Impedance / Mengambang)

use crate::ir::width_mask;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Nilai sinyal digital hingga 64-bit dengan pengodean dual-word `(val, unk)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LogicValue {
    /// Kata nilai biner (*value word*).
    pub val: u64,
    /// Kata penanda ketidakpastian (*unknown/high-z mask*).
    pub unk: u64,
}

impl LogicValue {
    /// Nilai `0` terdefinisi penuh.
    pub const ZERO: Self = Self { val: 0, unk: 0 };

    /// Nilai `X` (unknown) untuk seluruh 64 bit.
    pub const ALL_X: Self = Self {
        val: 0,
        unk: u64::MAX,
    };

    /// Membuat `LogicValue` dua-nilai (0/1) yang dipotong sesuai `width` (1..=64).
    #[inline]
    pub const fn from_u64(value: u64, width: u8) -> Self {
        let mask = width_mask(width);
        Self {
            val: value & mask,
            unk: 0,
        }
    }

    /// Membuat `LogicValue` empat-nilai dari pasangan `(val, unk)` dengan *width masking*.
    #[inline]
    pub const fn from_parts(val: u64, unk: u64, width: u8) -> Self {
        let mask = width_mask(width);
        Self {
            val: val & mask,
            unk: unk & mask,
        }
    }

    /// Membuat `LogicValue` bernilai `X` pada seluruh bit hingga `width`.
    #[inline]
    pub const fn all_x(width: u8) -> Self {
        Self {
            val: 0,
            unk: width_mask(width),
        }
    }

    /// Memeriksa apakah seluruh bit dalam lebar `width` diketahui (tidak ada bit `X` atau `Z`).
    #[inline]
    pub const fn is_fully_known(&self, width: u8) -> bool {
        (self.unk & width_mask(width)) == 0
    }
}

impl Display for LogicValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.unk == 0 {
            write!(f, "0x{:X}", self.val)
        } else {
            write!(f, "LogicValue(val=0x{:X}, unk=0x{:X})", self.val, self.unk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_value_masking() {
        let v = LogicValue::from_u64(0xFF, 4);
        assert_eq!(v.val, 0x0F);
        assert_eq!(v.unk, 0);
        assert!(v.is_fully_known(4));

        let x = LogicValue::all_x(4);
        assert_eq!(x.unk, 0x0F);
        assert!(!x.is_fully_known(4));
    }
}
