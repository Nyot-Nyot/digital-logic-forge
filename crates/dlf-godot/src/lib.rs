//! # GDExtension Adapter untuk Digital Logic Forge
//!
//! Modul ini menyediakan jembatan komunikasi antara antarmuka Godot Engine
//! dan mesin kompilasi/simulasi inti Rust (`dlf-core`).
//!
//! Selaras dengan keputusan arsitektur D04 dan D05:
//! - Adapter ini hanya mengekspos API tingkat tinggi (*coarse-grained*).
//! - State simulasi tidak pernah dibagikan langsung ke SceneTree Godot.

use dlf_core::worker::{SimulationWorkerController, WorkerCommand};
use godot::prelude::*;

/// Penanda library ekstensi untuk GDExtension Godot 4.
struct DlfExtension;

#[gdextension]
unsafe impl ExtensionLibrary for DlfExtension {}

/// Facade GDExtension utama yang diekspos ke GDScript.
///
/// Bertindak sebagai satu-satunya gerbang komunikasi antara
/// lapisan presentasi Godot dan mesin inti `dlf-core`.
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct DlfFacade {
    base: Base<RefCounted>,
    worker: Option<SimulationWorkerController>,
}

#[godot_api]
impl IRefCounted for DlfFacade {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base, worker: None }
    }
}

#[godot_api]
impl DlfFacade {
    /// Mengembalikan status mesin inti dan versi runtime.
    ///
    /// Digunakan untuk smoke test pemanggilan dari GDScript ke Rust.
    #[func]
    pub fn ping(&self) -> GString {
        GString::from(&dlf_core::engine_status())
    }

    /// Mengembalikan versi semantik pustaka inti DLF.
    #[func]
    pub fn get_core_version(&self) -> GString {
        GString::from(dlf_core::CORE_VERSION)
    }

    /// Memulai worker simulasi di thread latar belakang terpisah.
    #[func]
    pub fn worker_start(&mut self) -> bool {
        let worker = SimulationWorkerController::new();
        if worker.send_command(WorkerCommand::Start).is_ok() {
            self.worker = Some(worker);
            true
        } else {
            false
        }
    }

    /// Menjalankan simulasi terus-menerus dalam batch.
    #[func]
    pub fn worker_run(&mut self) -> bool {
        if let Some(ref worker) = self.worker {
            worker.send_command(WorkerCommand::Run).is_ok()
        } else {
            false
        }
    }

    /// Menjeda simulasi pada batas siklus terdekat.
    #[func]
    pub fn worker_pause(&mut self) -> bool {
        if let Some(ref worker) = self.worker {
            worker.send_command(WorkerCommand::Pause).is_ok()
        } else {
            false
        }
    }

    /// Menjalankan N siklus simulasi secara bertahap (single/multi-step).
    #[func]
    pub fn worker_step(&mut self, cycles: i64) -> bool {
        if let Some(ref worker) = self.worker {
            worker
                .send_command(WorkerCommand::Step(cycles.max(1) as u64))
                .is_ok()
        } else {
            false
        }
    }

    /// Mengembalikan keadaan simulasi ke siklus awal.
    #[func]
    pub fn worker_reset(&mut self) -> bool {
        if let Some(ref worker) = self.worker {
            worker.send_command(WorkerCommand::Reset).is_ok()
        } else {
            false
        }
    }

    /// Menghentikan worker latar belakang dan membebaskan sumber daya.
    #[func]
    pub fn worker_stop(&mut self) {
        if let Some(mut worker) = self.worker.take() {
            worker.shutdown();
        }
    }

    /// Mengambil salinan snapshot keluaran simulasi terkini (non-blocking).
    ///
    /// Mengembalikan Dictionary kosong jika belum ada snapshot atau worker belum aktif.
    #[func]
    pub fn poll_snapshot(&self) -> Dictionary<Variant, Variant> {
        let mut result = Dictionary::<Variant, Variant>::new();
        if let Some(snap) = self.worker.as_ref().and_then(|w| w.poll_snapshot()) {
            result.set("sequence", snap.sequence as i64);
            result.set("cycle_count", snap.cycle_count as i64);
            result.set("status", format!("{:?}", snap.status));
            result.set("speed_cps", snap.speed_cps);

            let mut outputs = Dictionary::<Variant, Variant>::new();
            for (id, val) in snap.outputs {
                outputs.set(id as i64, val as i64);
            }
            result.set("outputs", &outputs);
        }
        result
    }
}
