//! # Alokasi Slot Memori Kontigu dan Pemetaan Simbolik
//!
//! Modul ini mengimplementasikan `SignalSlotAllocator` yang memetakan setiap `NetId`
//! kanonis pada `Netlist` ke indeks slot memori datar (`usize`), sekaligus membangun
//! `InputBinding`, `OutputBinding`, `const_pool`, dan `SourceMap` sesuai Tabel 3.26 proposal.

use crate::connectivity::{NetId, NetTerminal, Netlist};
use crate::ir::{SignalBinding, SourceMap};
use crate::model::circuit::CircuitDefinition;
use crate::model::pin::PinDirection;
use std::collections::BTreeMap;

/// Kumpulan tata letak memori dan pemetaan simbolik hasil alokasi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocatedLayout {
    /// Jumlah total slot sinyal yang dialokasikan.
    pub slot_count: usize,
    /// Lebar bit untuk masing-masing slot sinyal.
    pub slot_widths: Vec<u8>,
    /// Kumpulan konstanta 4-nilai terdeduplikasi `(val, unk)`.
    pub const_pool: Vec<(u64, u64)>,
    /// Daftar pemetaan port masukan eksternal ke slot runtime.
    pub input_bindings: Vec<SignalBinding>,
    /// Daftar pemetaan port keluaran eksternal ke slot runtime.
    pub output_bindings: Vec<SignalBinding>,
    /// Peta balik untuk diagnostik dan inspeksi probe.
    pub source_map: SourceMap,
}

/// Pengelola alokasi indeks slot sinyal kontigu dan pemetaan simbolik untuk kompilator IR.
#[derive(Debug, Default)]
pub struct SignalSlotAllocator {
    net_to_slot: BTreeMap<NetId, usize>,
    slot_widths: Vec<u8>,
    const_pool: Vec<(u64, u64)>,
    const_dedup: BTreeMap<(u64, u64), usize>,
    input_bindings: Vec<SignalBinding>,
    output_bindings: Vec<SignalBinding>,
    source_map: SourceMap,
}

impl SignalSlotAllocator {
    /// Membuat alokator kosong baru.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mengalokasikan slot sinyal deterministik untuk seluruh `Net` pada `Netlist`
    /// serta membentuk `InputBinding`, `OutputBinding`, dan `SourceMap` dari `CircuitDefinition`.
    pub fn allocate_from_netlist(circuit: &CircuitDefinition, netlist: &Netlist) -> Self {
        let mut allocator = Self::new();

        // 1. Alokasikan slot secara deterministik berdasarkan urutan terurut NetId
        for (&net_id, net) in &netlist.nets {
            let slot = allocator.slot_widths.len();
            allocator.slot_widths.push(net.width.get());
            allocator.net_to_slot.insert(net_id, slot);

            if let Some(primary_terminal) = net.drivers.first().or_else(|| net.loads.first()) {
                allocator
                    .source_map
                    .insert_slot(slot, net_id, primary_terminal.to_source_ref());
            }
        }

        // 2. Bangun InputBinding & OutputBinding untuk setiap PortDefinition
        for port in &circuit.ports {
            let terminal = NetTerminal::Port { id: port.id };
            if let Some(&net_id) = netlist.terminal_to_net.get(&terminal)
                && let Some(&slot) = allocator.net_to_slot.get(&net_id)
            {
                let binding = SignalBinding::new(port.id, &port.name, slot, port.width.get());
                match port.direction {
                    PinDirection::Input => allocator.input_bindings.push(binding),
                    PinDirection::Output => allocator.output_bindings.push(binding),
                    PinDirection::Inout => {
                        // Port bidireksional didaftarkan pada kedua arah pengamatan
                        allocator.input_bindings.push(binding.clone());
                        allocator.output_bindings.push(binding);
                    }
                }
            }
        }

        allocator
    }

    /// Mengambil indeks slot untuk `NetId` tertentu jika sudah dialokasikan.
    #[inline]
    pub fn slot_of_net(&self, net_id: NetId) -> Option<usize> {
        self.net_to_slot.get(&net_id).copied()
    }

    /// Mengambil indeks slot untuk `NetTerminal` tertentu melalui pencarian pada `Netlist`.
    #[inline]
    pub fn slot_of_terminal(&self, netlist: &Netlist, terminal: &NetTerminal) -> Option<usize> {
        netlist
            .terminal_to_net
            .get(terminal)
            .and_then(|net_id| self.slot_of_net(*net_id))
    }

    /// Mengalokasikan slot sementara (*scratch slot*) tambahan untuk ekspresi antara.
    pub fn alloc_temp_slot(&mut self, width: u8) -> usize {
        let slot = self.slot_widths.len();
        self.slot_widths.push(width.clamp(1, 64));
        slot
    }

    /// Mendaftarkan atau mengambil indeks konstanta 4-nilai `(val, unk)` yang terdeduplikasi
    /// di dalam `const_pool`.
    pub fn intern_const(&mut self, val: u64, unk: u64) -> usize {
        if let Some(&idx) = self.const_dedup.get(&(val, unk)) {
            return idx;
        }
        let idx = self.const_pool.len();
        self.const_pool.push((val, unk));
        self.const_dedup.insert((val, unk), idx);
        idx
    }

    /// Mengakses `SourceMap` secara *mutable* untuk mencatat pemetaan operasi IR.
    #[inline]
    pub fn source_map_mut(&mut self) -> &mut SourceMap {
        &mut self.source_map
    }

    /// Jumlah total slot yang telah dialokasikan hingga saat ini.
    #[inline]
    pub fn slot_count(&self) -> usize {
        self.slot_widths.len()
    }

    /// Mengambil daftar lebar bit per slot.
    #[inline]
    pub fn slot_widths(&self) -> &[u8] {
        &self.slot_widths
    }

    /// Mengambil daftar `InputBinding`.
    #[inline]
    pub fn input_bindings(&self) -> &[SignalBinding] {
        &self.input_bindings
    }

    /// Mengambil daftar `OutputBinding`.
    #[inline]
    pub fn output_bindings(&self) -> &[SignalBinding] {
        &self.output_bindings
    }

    /// Mengkonsumsi alokator menjadi `AllocatedLayout` akhir untuk dimasukkan ke `CompiledArtifact`.
    pub fn into_layout(self) -> AllocatedLayout {
        AllocatedLayout {
            slot_count: self.slot_widths.len(),
            slot_widths: self.slot_widths,
            const_pool: self.const_pool,
            input_bindings: self.input_bindings,
            output_bindings: self.output_bindings,
            source_map: self.source_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::circuit::PortDefinition;
    use crate::model::component::ComponentRegistry;
    use crate::model::geometry::Point;
    use crate::model::id::{CircuitId, InstanceId, PinId, WireId};
    use crate::model::pin::SignalWidth;
    use crate::model::wiring::{Endpoint, WireSegment};

    #[test]
    fn test_allocator_maps_nets_bindings_and_consts() {
        let registry = ComponentRegistry::with_builtins();
        let mut circuit = CircuitDefinition::new(CircuitId::new(1), "AllocTest");

        let port_a = PortDefinition::new(
            PinId::new(10),
            "A",
            PinDirection::Input,
            SignalWidth::new(4).unwrap(),
            Point::ZERO,
        );
        let port_y = PortDefinition::new(
            PinId::new(11),
            "Y",
            PinDirection::Output,
            SignalWidth::new(4).unwrap(),
            Point::new(100.0, 0.0),
        );
        circuit.add_port(port_a);
        circuit.add_port(port_y);

        let not_def = registry.get("dlf.builtin.not").unwrap();
        let mut not_inst = not_def
            .instantiate(
                InstanceId::new(100),
                &[PinId::new(20), PinId::new(21)],
                Point::new(50.0, 0.0),
            )
            .unwrap();
        not_inst.pins[0].width = SignalWidth::new(4).unwrap();
        not_inst.pins[1].width = SignalWidth::new(4).unwrap();
        circuit.add_component(not_inst);

        circuit.add_wire(WireSegment::new(
            WireId::new(200),
            Endpoint::Port { id: PinId::new(10) },
            Endpoint::Pin {
                instance_id: InstanceId::new(100),
                pin_id: PinId::new(20),
            },
        ));
        circuit.add_wire(WireSegment::new(
            WireId::new(201),
            Endpoint::Pin {
                instance_id: InstanceId::new(100),
                pin_id: PinId::new(21),
            },
            Endpoint::Port { id: PinId::new(11) },
        ));

        let netlist = Netlist::from_circuit(&circuit);
        let mut allocator = SignalSlotAllocator::allocate_from_netlist(&circuit, &netlist);

        assert_eq!(allocator.slot_count(), 2);
        assert_eq!(allocator.input_bindings().len(), 1);
        assert_eq!(allocator.input_bindings()[0].name, "A");
        assert_eq!(allocator.input_bindings()[0].width, 4);
        assert_eq!(allocator.output_bindings().len(), 1);
        assert_eq!(allocator.output_bindings()[0].name, "Y");

        let temp_slot = allocator.alloc_temp_slot(8);
        assert_eq!(temp_slot, 2);
        assert_eq!(allocator.slot_count(), 3);

        // Deduplikasi const_pool
        let c0 = allocator.intern_const(0xA, 0);
        let c1 = allocator.intern_const(0xA, 0);
        let c2 = allocator.intern_const(0xF, 0);
        assert_eq!(c0, c1);
        assert_ne!(c0, c2);

        let layout = allocator.into_layout();
        assert_eq!(layout.const_pool.len(), 2);
        assert_eq!(layout.source_map.slot_to_net.len(), 2);
    }
}
