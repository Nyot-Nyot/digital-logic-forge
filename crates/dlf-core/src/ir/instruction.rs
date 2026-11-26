//! # Definisi Instruksi IR (Intermediate Representation)
//!
//! Modul ini mendefinisikan set instruksi ringkas (`Opcode`) dan rekaman operasi datar
//! (`CompiledOp`) sesuai Tabel 3.25 proposal serta keputusan `[SPIKE-004]`.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Penanda (*flags*) tambahan pada instruksi IR.
pub mod op_flags {
    /// Tidak ada penanda khusus.
    pub const NONE: u8 = 0;
    /// Menandai sinyal tujuan diamati oleh probe/output sehingga tidak boleh dihapus optimasi.
    pub const OBSERVABLE: u8 = 1 << 0;
}

/// Menghitung *bitmask* 64-bit untuk lebar sinyal `width` (1 hingga 64 bit).
///
/// Nilai di luar rentang `1..=64` ditangani secara aman: `0` menghasilkan `0`,
/// dan `>= 64` menghasilkan `u64::MAX`.
#[inline]
pub const fn width_mask(width: u8) -> u64 {
    if width == 0 {
        0
    } else if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    }
}

/// Jenis operasi primitif dalam IR terkompilasi DLF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Opcode {
    /// Penyangga (*buffer*): `dst = src_a`.
    Buf,
    /// Negasi bitwise (*NOT*): `dst = !src_a`.
    Not,
    /// Konjungsi bitwise (*AND*): `dst = src_a & src_b`.
    And,
    /// Disjungsi bitwise (*OR*): `dst = src_a | src_b`.
    Or,
    /// Disjungsi eksklusif bitwise (*XOR*): `dst = src_a ^ src_b`.
    Xor,
    /// Negasi AND (*NAND*): `dst = !(src_a & src_b)`.
    Nand,
    /// Negasi OR (*NOR*): `dst = !(src_a | src_b)`.
    Nor,
    /// Negasi XOR (*XNOR*): `dst = !(src_a ^ src_b)`.
    Xnor,
    /// Memuat nilai konstanta dari `const_pool[src_a]` ke `dst`.
    Const,
    /// Multiplexer 2-masukan: memilih `src_a` (jika `src_c == 0`) atau `src_b` (jika `src_c == 1`).
    Mux,
    /// Ekstraksi irisan bit (*bit slice*): `dst = (src_a >> src_b) & width_mask(width)`.
    /// Pada operasi ini, `src_b` menyimpan offset bit awal (*LSB offset*).
    Slice,
    /// Penggabungan dua bus (*concatenation*): `dst = (src_a & mask_a) | ((src_b & mask_b) << src_c)`.
    /// Pada operasi ini, `src_c` menyimpan lebar bit `src_a` (jumlah pergeseran untuk `src_b`).
    Concat,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Buf => "BUF",
            Self::Not => "NOT",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::Nand => "NAND",
            Self::Nor => "NOR",
            Self::Xnor => "XNOR",
            Self::Const => "CONST",
            Self::Mux => "MUX",
            Self::Slice => "SLICE",
            Self::Concat => "CONCAT",
        };
        write!(f, "{}", name)
    }
}

/// Satu rekaman instruksi IR pada array eksekusi terkompilasi (Tabel 3.25).
///
/// Seluruh referensi sinyal berupa indeks integer langsung (`usize`) ke array kontigu
/// tanpa pencarian berbasis nama atau identitas objek saat runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledOp {
    /// Jenis operasi yang dieksekusi.
    pub opcode: Opcode,
    /// Indeks slot sinyal tujuan (*destination*).
    pub dst: usize,
    /// Indeks slot sumber pertama (atau indeks `const_pool` untuk `Opcode::Const`).
    pub src_a: usize,
    /// Indeks slot sumber kedua (atau offset bit untuk `Opcode::Slice`).
    pub src_b: usize,
    /// Indeks slot sumber ketiga (misal sinyal *select* pada `Opcode::Mux`,
    /// atau lebar bit `src_a` pada `Opcode::Concat`).
    pub src_c: usize,
    /// Lebar bit operasi (valid dalam rentang `1..=64`).
    pub width: u8,
    /// Penanda tambahan (`op_flags`).
    pub flags: u8,
}

impl CompiledOp {
    /// Membuat instruksi operasi uner (`Buf` atau `Not`).
    #[inline]
    pub const fn unary(opcode: Opcode, dst: usize, src_a: usize, width: u8) -> Self {
        Self {
            opcode,
            dst,
            src_a,
            src_b: 0,
            src_c: 0,
            width,
            flags: op_flags::NONE,
        }
    }

    /// Membuat instruksi operasi biner 2-masukan (`And`, `Or`, `Xor`, `Nand`, `Nor`, `Xnor`).
    #[inline]
    pub const fn binary(opcode: Opcode, dst: usize, src_a: usize, src_b: usize, width: u8) -> Self {
        Self {
            opcode,
            dst,
            src_a,
            src_b,
            src_c: 0,
            width,
            flags: op_flags::NONE,
        }
    }

    /// Membuat instruksi pemuatan konstanta (`Const`) dari indeks `const_index`.
    #[inline]
    pub const fn constant(dst: usize, const_index: usize, width: u8) -> Self {
        Self {
            opcode: Opcode::Const,
            dst,
            src_a: const_index,
            src_b: 0,
            src_c: 0,
            width,
            flags: op_flags::NONE,
        }
    }

    /// Membuat instruksi multiplexer 2-ke-1 (`Mux`).
    #[inline]
    pub const fn mux(dst: usize, d0: usize, d1: usize, sel: usize, width: u8) -> Self {
        Self {
            opcode: Opcode::Mux,
            dst,
            src_a: d0,
            src_b: d1,
            src_c: sel,
            width,
            flags: op_flags::NONE,
        }
    }

    /// Membuat instruksi pemotongan bus (`Slice`).
    #[inline]
    pub const fn slice(dst: usize, src: usize, lsb_offset: usize, width: u8) -> Self {
        Self {
            opcode: Opcode::Slice,
            dst,
            src_a: src,
            src_b: lsb_offset,
            src_c: 0,
            width,
            flags: op_flags::NONE,
        }
    }

    /// Membuat instruksi penggabungan dua bus (`Concat`).
    ///
    /// `low_src` ditempatkan pada bit rendah (`0..low_width`), sedangkan `high_src`
    /// digeser ke kiri sebesar `low_width`.
    #[inline]
    pub const fn concat(
        dst: usize,
        low_src: usize,
        high_src: usize,
        low_width: usize,
        total_width: u8,
    ) -> Self {
        Self {
            opcode: Opcode::Concat,
            dst,
            src_a: low_src,
            src_b: high_src,
            src_c: low_width,
            width: total_width,
            flags: op_flags::NONE,
        }
    }

    /// Menandai instruksi ini sebagai *observable* agar tidak dihapus oleh pass optimasi.
    #[inline]
    pub const fn with_observable(mut self) -> Self {
        self.flags |= op_flags::OBSERVABLE;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_mask_boundaries() {
        assert_eq!(width_mask(0), 0);
        assert_eq!(width_mask(1), 0b1);
        assert_eq!(width_mask(4), 0b1111);
        assert_eq!(width_mask(8), 0xFF);
        assert_eq!(width_mask(32), 0xFFFF_FFFF);
        assert_eq!(width_mask(63), (1u64 << 63) - 1);
        assert_eq!(width_mask(64), u64::MAX);
    }

    #[test]
    fn test_compiled_op_constructors() {
        let op_and = CompiledOp::binary(Opcode::And, 3, 0, 1, 8).with_observable();
        assert_eq!(op_and.opcode, Opcode::And);
        assert_eq!(op_and.dst, 3);
        assert_eq!(op_and.src_a, 0);
        assert_eq!(op_and.src_b, 1);
        assert_eq!(op_and.width, 8);
        assert_ne!(op_and.flags & op_flags::OBSERVABLE, 0);

        let op_mux = CompiledOp::mux(5, 1, 2, 4, 16);
        assert_eq!(op_mux.opcode, Opcode::Mux);
        assert_eq!(op_mux.src_a, 1);
        assert_eq!(op_mux.src_b, 2);
        assert_eq!(op_mux.src_c, 4);
        assert_eq!(op_mux.width, 16);
    }
}
