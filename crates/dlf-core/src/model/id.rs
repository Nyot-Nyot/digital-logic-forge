//! # Pengenal Entitas Stabil (Stable ID Types)
//!
//! Modul ini mendefinisikan tipe-tipe pengenal (*identifiers*) berkekuatan tipe (*strongly-typed newtypes*)
//! berbasis bilangan bulat 64-bit tak bertanda (`u64`) sesuai Bab 3.10.1 proposal.
//!
//! Setiap entitas di dokumen rangkaian (*Authoring Document*) memiliki ID unik yang stabil
//! melintasi operasi perpindahan posisi, undo/redo, serta penyimpanan dan pemuatan berkas.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Makro pembantu untuk mendeklarasikan newtype ID 64-bit yang konsisten dan berdokumen.
macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[repr(transparent)]
        pub struct $name(pub u64);

        impl $name {
            /// Membuat instance baru dari nilai bilangan bulat mentah `u64`.
            #[inline]
            pub const fn new(val: u64) -> Self {
                Self(val)
            }

            /// Mengambil representasi nilai bilangan bulat mentah `u64`.
            #[inline]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<u64> for $name {
            #[inline]
            fn from(val: u64) -> Self {
                Self(val)
            }
        }

        impl From<$name> for u64 {
            #[inline]
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(
    CircuitId,
    "Pengenal unik untuk definisi rangkaian (`CircuitDefinition`)."
);
define_id!(
    InstanceId,
    "Pengenal unik untuk instansiasi komponen (`ComponentInstance`)."
);
define_id!(
    PinId,
    "Pengenal unik untuk terminal/pin komponen atau port rangkaian (`PinInstance`/`PortDefinition`)."
);
define_id!(
    WireId,
    "Pengenal unik untuk segmen kawat penghubung (`WireSegment`)."
);
define_id!(
    JunctionId,
    "Pengenal unik untuk titik sambung cabang kawat (`Junction`)."
);
define_id!(
    TunnelId,
    "Pengenal unik untuk entitas label tunnel nirkabel (`Tunnel`)."
);

/// Generator terpusat deterministik untuk pembuatan Stable ID 64-bit dalam satu proyek.
///
/// Generator ini memastikan bahwa tidak ada ID yang dipakai ulang dalam satu proyek yang sama
/// selama sesi penyuntingan maupun penambahan objek baru.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdGenerator {
    /// Nilai ID berikutnya yang akan dialokasikan.
    next_id: u64,
}

impl IdGenerator {
    /// Nilai ID awal default jika tidak ditentukan (ID dimulai dari 1, di mana 0 dicadangkan/invalid).
    pub const DEFAULT_START_ID: u64 = 1;

    /// Membuat instance generator baru dengan nilai awal tertentu.
    pub const fn new(start_id: u64) -> Self {
        Self { next_id: start_id }
    }

    /// Mengambil nilai ID berikutnya tanpa mengalokasikannya.
    pub const fn peek_next(&self) -> u64 {
        self.next_id
    }

    /// Memastikan generator bernilai sekurangnya `min_id` untuk mencegah tabrakan saat memuat berkas.
    pub fn bump_to_at_least(&mut self, min_id: u64) {
        if self.next_id < min_id {
            self.next_id = min_id;
        }
    }

    /// Mengalokasikan nilai ID mentah `u64` dan menaikkan penghitung internal secara deterministik.
    pub fn allocate_raw(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// Mengalokasikan `CircuitId` baru.
    pub fn allocate_circuit_id(&mut self) -> CircuitId {
        CircuitId::new(self.allocate_raw())
    }

    /// Mengalokasikan `InstanceId` baru.
    pub fn allocate_instance_id(&mut self) -> InstanceId {
        InstanceId::new(self.allocate_raw())
    }

    /// Mengalokasikan `PinId` baru.
    pub fn allocate_pin_id(&mut self) -> PinId {
        PinId::new(self.allocate_raw())
    }

    /// Mengalokasikan `WireId` baru.
    pub fn allocate_wire_id(&mut self) -> WireId {
        WireId::new(self.allocate_raw())
    }

    /// Mengalokasikan `JunctionId` baru.
    pub fn allocate_junction_id(&mut self) -> JunctionId {
        JunctionId::new(self.allocate_raw())
    }

    /// Mengalokasikan `TunnelId` baru.
    pub fn allocate_tunnel_id(&mut self) -> TunnelId {
        TunnelId::new(self.allocate_raw())
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new(Self::DEFAULT_START_ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_id_generator_monotonic_and_unique() {
        let mut generator = IdGenerator::new(1);
        let mut seen = HashSet::new();

        for _ in 0..100 {
            let id = generator.allocate_raw();
            assert!(seen.insert(id), "ID {} terduplikasi!", id);
        }

        assert_eq!(generator.peek_next(), 101);
    }

    #[test]
    fn test_typed_allocations() {
        let mut generator = IdGenerator::new(10);

        let c_id = generator.allocate_circuit_id();
        let i_id = generator.allocate_instance_id();
        let p_id = generator.allocate_pin_id();
        let w_id = generator.allocate_wire_id();
        let j_id = generator.allocate_junction_id();
        let t_id = generator.allocate_tunnel_id();

        assert_eq!(c_id.get(), 10);
        assert_eq!(i_id.get(), 11);
        assert_eq!(p_id.get(), 12);
        assert_eq!(w_id.get(), 13);
        assert_eq!(j_id.get(), 14);
        assert_eq!(t_id.get(), 15);
        assert_eq!(generator.peek_next(), 16);
    }

    #[test]
    fn test_bump_to_at_least() {
        let mut generator = IdGenerator::new(5);
        generator.bump_to_at_least(20);
        assert_eq!(generator.allocate_raw(), 20);

        // Memanggil bump dengan nilai lebih kecil tidak boleh memundurkan generator
        generator.bump_to_at_least(10);
        assert_eq!(generator.allocate_raw(), 21);
    }

    #[test]
    fn test_id_display_and_conversion() {
        let wire_id = WireId::new(42);
        assert_eq!(format!("{}", wire_id), "42");
        let raw: u64 = wire_id.into();
        assert_eq!(raw, 42);
        let from_raw = WireId::from(raw);
        assert_eq!(from_raw, wire_id);
    }
}
