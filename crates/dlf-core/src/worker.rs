//! # Simulation Worker & Snapshot Protocol
//!
//! Mengimplementasikan siklus hidup thread latar belakang (*background simulation worker*)
//! yang terisolasi secara mutlak dari UI Godot.
//!
//! Kontrak arsitektur (D04, D05, D08):
//! - Worker adalah satu-satunya pemilik *mutable runtime state*.
//! - UI hanya dapat berkomunikasi via antrean perintah bertipe (*typed command queue*).
//! - Worker menerbitkan *immutable output snapshot* ke penyangga terkunci/ring-buffer,
//!   yang dipoles (*polled*) oleh main thread Godot pada waktu yang aman.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Status siklus hidup simulation worker sesuai Tabel 3.27 proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkerStatus {
    /// Worker dibuat dan belum memuat artefak.
    Idle,
    /// Artefak dimuat dan siklus berhenti pada batas siklus.
    Paused,
    /// Menjalankan siklus dalam batch di background thread.
    Running,
    /// Terjadi kegagalan runtime eksekusi.
    Error,
    /// Worker dihentikan dan runtime state dilepaskan.
    Stopped,
}

/// Perintah bertipe yang dikirim dari UI ke worker sesuai Tabel 3.28 proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerCommand {
    /// Memulai simulasi dari keadaan Idle.
    Start,
    /// Menjalankan siklus terus-menerus dalam batch.
    Run,
    /// Menjeda siklus pada batas siklus berikutnya.
    Pause,
    /// Menjalankan sejumlah N siklus lalu berhenti pada Paused.
    Step(u64),
    /// Mengembalikan state ke state awal dan cycle counter ke 0.
    Reset,
    /// Menghentikan worker dan membuang runtime state.
    Stop,
    /// Mengubah nilai masukan runtime pada batas siklus.
    SetInput {
        /// ID binding masukan.
        binding_id: u64,
        /// Nilai sinyal masukan baru.
        value: u64,
    },
    /// Mengatur ukuran batch siklus sebelum memeriksa perintah/snapshot.
    SetBatchSize(usize),
}

/// Snapshot keluaran yang diterbitkan worker ke UI sesuai Tabel 3.29 proposal.
///
/// Objek ini bersifat *immutable* setelah dibuat dan aman dikonsumsi
/// oleh main thread tanpa risiko data race.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputSnapshot {
    /// Nomor urut snapshot yang naik secara monoton.
    pub sequence: u64,
    /// Total siklus logis yang telah disimulasikan sejak Start/Reset.
    pub cycle_count: u64,
    /// Status worker saat snapshot dibuat.
    pub status: WorkerStatus,
    /// Nilai sinyal keluaran (ID binding -> nilai).
    pub outputs: Vec<(u64, u64)>,
    /// Estimasi kecepatan simulasi dalam siklus per detik (CPS).
    pub speed_cps: f64,
    /// Pesan diagnostik runtime jika ada.
    pub diagnostics: Vec<String>,
}

/// Controller sisi klien (UI / main thread) untuk mengontrol simulation worker.
pub struct SimulationWorkerController {
    command_tx: Sender<WorkerCommand>,
    latest_snapshot: Arc<Mutex<Option<OutputSnapshot>>>,
    is_shutdown: Arc<AtomicBool>,
    worker_thread: Option<JoinHandle<()>>,
}

impl SimulationWorkerController {
    /// Membuat controller baru dan memunculkan background worker thread.
    pub fn new() -> Self {
        let (command_tx, command_rx) = channel::<WorkerCommand>();
        let latest_snapshot = Arc::new(Mutex::new(None));
        let is_shutdown = Arc::new(AtomicBool::new(false));

        let snapshot_sink = Arc::clone(&latest_snapshot);
        let shutdown_flag = Arc::clone(&is_shutdown);

        let worker_thread = thread::Builder::new()
            .name("dlf-simulation-worker".into())
            .spawn(move || {
                Self::run_worker_loop(command_rx, snapshot_sink, shutdown_flag);
            })
            .expect("Gagal memunculkan simulation worker thread");

        Self {
            command_tx,
            latest_snapshot,
            is_shutdown,
            worker_thread: Some(worker_thread),
        }
    }

    /// Mengirim perintah bertipe ke worker secara non-blocking.
    pub fn send_command(&self, cmd: WorkerCommand) -> Result<(), String> {
        self.command_tx
            .send(cmd)
            .map_err(|e| format!("Gagal mengirim perintah ke worker: {}", e))
    }

    /// Mengambil snapshot terbaru yang telah diterbitkan worker (non-blocking).
    pub fn poll_snapshot(&self) -> Option<OutputSnapshot> {
        let guard = self.latest_snapshot.lock().ok()?;
        guard.clone()
    }

    /// Menghentikan worker thread dan menunggu hingga selesai.
    pub fn shutdown(&mut self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        let _ = self.command_tx.send(WorkerCommand::Stop);
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
    }

    /// Loop internal yang berjalan di background worker thread murni Rust.
    fn run_worker_loop(
        command_rx: Receiver<WorkerCommand>,
        snapshot_sink: Arc<Mutex<Option<OutputSnapshot>>>,
        shutdown_flag: Arc<AtomicBool>,
    ) {
        let mut status = WorkerStatus::Idle;
        let mut cycle_count: u64 = 0;
        let mut sequence: u64 = 0;
        let mut batch_size: usize = 1000;
        let mut dummy_input_val: u64 = 0;

        let mut last_measure_time = Instant::now();
        let mut last_measure_cycles: u64 = 0;
        let mut current_cps: f64 = 0.0;

        let publish_snapshot =
            |seq: &mut u64,
             cyc: u64,
             st: WorkerStatus,
             cps: f64,
             val: u64,
             sink: &Arc<Mutex<Option<OutputSnapshot>>>| {
                *seq += 1;
                let snapshot = OutputSnapshot {
                    sequence: *seq,
                    cycle_count: cyc,
                    status: st,
                    outputs: vec![(1, val)], // Mock output binding untuk baseline M0
                    speed_cps: cps,
                    diagnostics: Vec::new(),
                };
                if let Ok(mut guard) = sink.lock() {
                    *guard = Some(snapshot);
                }
            };

        // Publikasikan snapshot awal Idle
        publish_snapshot(
            &mut sequence,
            cycle_count,
            status,
            current_cps,
            dummy_input_val,
            &snapshot_sink,
        );

        while !shutdown_flag.load(Ordering::Relaxed) {
            // 1. Proses semua antrean perintah yang tersedia
            loop {
                match command_rx.try_recv() {
                    Ok(cmd) => match cmd {
                        WorkerCommand::Start => {
                            status = WorkerStatus::Paused;
                            publish_snapshot(
                                &mut sequence,
                                cycle_count,
                                status,
                                current_cps,
                                dummy_input_val,
                                &snapshot_sink,
                            );
                        }
                        WorkerCommand::Run => {
                            if status == WorkerStatus::Paused {
                                status = WorkerStatus::Running;
                                last_measure_time = Instant::now();
                                last_measure_cycles = cycle_count;
                            }
                        }
                        WorkerCommand::Pause => {
                            if status == WorkerStatus::Running {
                                status = WorkerStatus::Paused;
                                publish_snapshot(
                                    &mut sequence,
                                    cycle_count,
                                    status,
                                    current_cps,
                                    dummy_input_val,
                                    &snapshot_sink,
                                );
                            }
                        }
                        WorkerCommand::Step(n) => {
                            if status == WorkerStatus::Paused {
                                // Eksekusi N siklus
                                cycle_count = cycle_count.saturating_add(n);
                                dummy_input_val = dummy_input_val.wrapping_add(n);
                                publish_snapshot(
                                    &mut sequence,
                                    cycle_count,
                                    status,
                                    current_cps,
                                    dummy_input_val,
                                    &snapshot_sink,
                                );
                            }
                        }
                        WorkerCommand::Reset => {
                            cycle_count = 0;
                            dummy_input_val = 0;
                            status = WorkerStatus::Paused;
                            publish_snapshot(
                                &mut sequence,
                                cycle_count,
                                status,
                                0.0,
                                dummy_input_val,
                                &snapshot_sink,
                            );
                        }
                        WorkerCommand::Stop => {
                            status = WorkerStatus::Stopped;
                            publish_snapshot(
                                &mut sequence,
                                cycle_count,
                                status,
                                0.0,
                                dummy_input_val,
                                &snapshot_sink,
                            );
                            return;
                        }
                        WorkerCommand::SetInput {
                            binding_id: _,
                            value,
                        } => {
                            dummy_input_val = value;
                        }
                        WorkerCommand::SetBatchSize(size) => {
                            batch_size = size.max(1);
                        }
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            // 2. Eksekusi batch siklus jika berstatus Running
            if status == WorkerStatus::Running {
                // Simulasi batch siklus
                cycle_count = cycle_count.saturating_add(batch_size as u64);
                dummy_input_val = dummy_input_val.wrapping_add(1);

                // Hitung CPS
                let elapsed = last_measure_time.elapsed();
                if elapsed >= Duration::from_millis(50) {
                    let cycles_delta = cycle_count.saturating_sub(last_measure_cycles);
                    current_cps = (cycles_delta as f64) / elapsed.as_secs_f64();
                    last_measure_time = Instant::now();
                    last_measure_cycles = cycle_count;

                    // Terbitkan snapshot terkini
                    publish_snapshot(
                        &mut sequence,
                        cycle_count,
                        status,
                        current_cps,
                        dummy_input_val,
                        &snapshot_sink,
                    );
                }

                // Berikan yielding kecil agar CPU tidak throttling
                thread::yield_now();
            } else {
                // Jika sedang tidak berjalan, tidur sebentar untuk menghemat CPU
                thread::sleep(Duration::from_millis(5));
            }
        }
    }
}

impl Default for SimulationWorkerController {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SimulationWorkerController {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_lifecycle_and_commands() {
        let mut controller = SimulationWorkerController::new();

        // Ambil snapshot awal
        thread::sleep(Duration::from_millis(20));
        let snap1 = controller.poll_snapshot().expect("Harus ada snapshot awal");
        assert_eq!(snap1.status, WorkerStatus::Idle);
        assert_eq!(snap1.cycle_count, 0);

        // Kirim perintah Start -> status Paused
        controller.send_command(WorkerCommand::Start).unwrap();
        thread::sleep(Duration::from_millis(20));
        let snap2 = controller.poll_snapshot().unwrap();
        assert_eq!(snap2.status, WorkerStatus::Paused);

        // Kirim perintah Step(5)
        controller.send_command(WorkerCommand::Step(5)).unwrap();
        thread::sleep(Duration::from_millis(20));
        let snap3 = controller.poll_snapshot().unwrap();
        assert_eq!(snap3.cycle_count, 5);

        // Kirim perintah Run
        controller.send_command(WorkerCommand::Run).unwrap();
        thread::sleep(Duration::from_millis(100));
        let snap4 = controller.poll_snapshot().unwrap();
        assert_eq!(snap4.status, WorkerStatus::Running);
        assert!(snap4.cycle_count > 5);

        // Kirim perintah Pause
        controller.send_command(WorkerCommand::Pause).unwrap();
        thread::sleep(Duration::from_millis(30));
        let snap5 = controller.poll_snapshot().unwrap();
        assert_eq!(snap5.status, WorkerStatus::Paused);
        let frozen_cycles = snap5.cycle_count;

        // Pastikan tidak ada siklus tambahan setelah dijeda
        thread::sleep(Duration::from_millis(30));
        let snap6 = controller.poll_snapshot().unwrap();
        assert_eq!(snap6.cycle_count, frozen_cycles);

        // Shutdown
        controller.shutdown();
    }
}
