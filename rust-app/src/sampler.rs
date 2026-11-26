//! Background sampling thread. The UI thread never blocks on system calls: it only
//! reads the latest `State` behind a mutex.
//!
//! Efficiency notes:
//! * Every sysinfo container (`Disks`, `Components`, `Networks`) is created once and
//!   refreshed in place – enumeration of `/sys`, `/proc/net` and mounts only happens
//!   every 30–60 s (hot-plug detection), not on every sample.
//! * Per-application rows are aggregated through borrowed process names, so we only
//!   allocate one `String` per application instead of one per process and per tick.
//! * Disk I/O is sampled as deltas of the cumulative per-process counters, which is
//!   both correct and stable; no extra I/O is performed for it.
//! * Battery, GPU process usage, temperatures and disk lists are refreshed on slower
//!   cadences because they change slowly.

use crate::gpu::{self, GpuInfo};
use eframe::egui;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Components, CpuRefreshKind, Disks, Networks, Pid, ProcessesToUpdate, System};

pub const HIST: usize = 90;

#[derive(Clone, Default)]
pub struct AppRow {
    pub name: String,
    pub icon_path: Option<std::path::PathBuf>,
    pub procs: usize,
    pub cpu: f32,
    pub mem: u64,
    pub gpu: Option<f32>,
    /// Per-application network throughput is not exposed by a portable API; kept for platform back-ends.
    pub net: Option<f64>,
    pub disk_read: f64,
    pub disk_write: f64,
}

#[derive(Clone, Default)]
pub struct DiskRow {
    /// Friendly drive name, shown by platform-specific UI extensions.
    #[allow(dead_code)]
    pub name: String,
    pub mount: String,
    pub fs: String,
    pub kind: String,
    pub total: u64,
    pub avail: u64,
    pub removable: bool,
}

#[derive(Clone, Default)]
pub struct BatteryInfo {
    pub pct: f32,
    pub state: String,
    pub health: f32,
    pub minutes: Option<f32>,
    pub cycles: Option<u32>,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub cpu: f32,
    pub cores: Vec<f32>,
    pub freq_mhz: u64,
    pub mem_used: u64,
    pub mem_total: u64,
    pub swap_used: u64,
    #[allow(dead_code)]
    pub swap_total: u64,
    pub gpus: Vec<GpuInfo>,
    pub disks: Vec<DiskRow>,
    pub disk_read_bps: f64,
    pub disk_write_bps: f64,
    pub net_rx_bps: f64,
    pub net_tx_bps: f64,
    pub apps: Vec<AppRow>,
    pub battery: Option<BatteryInfo>,
    pub uptime: u64,
    pub temps: Vec<(String, f32)>,
    pub load: (f64, f64, f64),
}

#[derive(Default)]
pub struct History {
    pub cpu: VecDeque<f32>,
    pub gpu: VecDeque<f32>,
    pub ram: VecDeque<f32>,
    pub vram: VecDeque<f32>,
    pub disk_r: VecDeque<f32>,
    pub disk_w: VecDeque<f32>,
    pub net_rx: VecDeque<f32>,
    pub net_tx: VecDeque<f32>,
}

#[inline]
fn push(q: &mut VecDeque<f32>, v: f32) {
    if q.len() >= HIST {
        q.pop_front();
    }
    q.push_back(v);
}

#[derive(Default)]
pub struct State {
    pub snap: Snapshot,
    pub hist: History,
    pub ready: bool,
    /// Incremented for every published sample; lets the UI cheaply detect new data.
    pub seq: u64,
    /// How long the last sampling pass took (µs) – used by the built-in benchmark.
    pub sample_us: u64,
}

pub fn spawn(ctx: egui::Context, refresh_ms: Arc<AtomicU64>) -> Arc<Mutex<State>> {
    let shared = Arc::new(Mutex::new(State::default()));
    let out = shared.clone();

    thread::Builder::new()
        .name("sampler".into())
        .spawn(move || {
            let mut sys = System::new_all();
            let desktop_icons = crate::icons::DesktopIcons::discover();
            let mut disks = Disks::new_with_refreshed_list();
            let mut comps = Components::new_with_refreshed_list();
            let mut networks = Networks::new_with_refreshed_list();
            let battery_mgr = battery::Manager::new().ok();

            let mut last_net: Option<(u64, u64)> = None;
            // pid -> (cumulative read, cumulative written) from the previous sample
            let mut last_disk: HashMap<Pid, (u64, u64)> = HashMap::new();
            let mut next_disk: HashMap<Pid, (u64, u64)> =
                HashMap::with_capacity(last_disk.capacity());
            let mut last = Instant::now();
            let mut tick: u64 = 0;
            let mut gpu_info: Vec<GpuInfo> = Vec::new();
            let mut gpu_procs: HashMap<u32, f32> = HashMap::new();
            let mut disk_rows: Vec<DiskRow> = Vec::new();
            let mut temps: Vec<(String, f32)> = Vec::new();
            let mut battery: Option<BatteryInfo> = None;
            let mut load = (0.0f64, 0.0f64, 0.0f64);

            loop {
                let cycle_start = Instant::now();
                let now = Instant::now();
                let dt = now.duration_since(last).as_secs_f64().max(0.05);
                last = now;

                // ── cheap, every tick ────────────────────────────────────────────
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                sys.refresh_processes(ProcessesToUpdate::All, true);

                // ── slow-changing system data ───────────────────────────────────
                if tick % 5 == 0 {
                    if tick > 0 && tick % 60 != 0 {
                        disks.refresh();
                        for c in comps.iter_mut() {
                            c.refresh();
                        }
                    } else {
                        // full re-enumeration detects newly plugged / removed devices
                        disks.refresh_list();
                        comps.refresh_list();
                    }
                    disk_rows.clear();
                    disk_rows.extend(disks.iter().map(|d| DiskRow {
                        name: d.name().to_string_lossy().into_owned(),
                        mount: d.mount_point().to_string_lossy().into_owned(),
                        fs: d.file_system().to_string_lossy().into_owned(),
                        kind: d.kind().to_string(),
                        total: d.total_space(),
                        avail: d.available_space(),
                        removable: d.is_removable(),
                    }));
                    temps.clear();
                    temps.extend(
                        comps
                            .iter()
                            .filter(|c| c.temperature().is_finite())
                            .map(|c| (c.label().to_string(), c.temperature())),
                    );
                    if tick % 10 == 0 {
                        let la = System::load_average();
                        load = (la.one, la.five, la.fifteen);
                        sys.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency());
                    }
                }
                if tick % 10 == 0 {
                    battery = read_battery(&battery_mgr);
                }
                if tick % 2 == 0 {
                    gpu_info = gpu::info();
                    if gpu_info.is_empty() {
                        gpu_procs.clear();
                    }
                    // `nvidia-smi pmon` is an extra process spawn: 5× cheaper cadence.
                    if tick % 6 == 0 {
                        gpu_procs = gpu::process_usage();
                    }
                }

                // ── per-application aggregation ────────────────────────────────
                // Keys borrow from `sys`, so no name allocation happens per process.
                let mut per_app: HashMap<&str, AppRow> = HashMap::with_capacity(256);
                let (mut rd_total, mut wr_total) = (0u64, 0u64);
                next_disk.clear();
                for (pid, p) in sys.processes() {
                    let name = p.name().to_str().unwrap_or("unknown process");
                    let du = p.disk_usage();
                    let cur = (du.read_bytes, du.written_bytes);
                    let (prev_r, prev_w) = last_disk.get(pid).copied().unwrap_or(cur);
                    let dr = cur.0.saturating_sub(prev_r);
                    let dw = cur.1.saturating_sub(prev_w);
                    rd_total += dr;
                    wr_total += dw;
                    next_disk.insert(*pid, cur);

                    let e = per_app.entry(name).or_default();
                    e.procs += 1;
                    e.cpu += p.cpu_usage();
                    e.mem += p.memory();
                    if e.icon_path.is_none() {
                        e.icon_path = desktop_icons.for_process(name);
                    }
                    e.disk_read += dr as f64 / dt;
                    e.disk_write += dw as f64 / dt;
                    if let Some(g) = gpu_procs.get(&pid.as_u32()) {
                        *e.gpu.get_or_insert(0.0) += *g;
                    }
                }
                std::mem::swap(&mut last_disk, &mut next_disk);

                let ncpu = sys.cpus().len().max(1) as f32;
                let mut apps: Vec<AppRow> = Vec::with_capacity(per_app.len());
                for (name, mut row) in per_app {
                    row.name = name.to_string();
                    row.cpu /= ncpu;
                    apps.push(row);
                }
                apps.sort_unstable_by(|a, b| {
                    b.cpu
                        .partial_cmp(&a.cpu)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // ── network (cumulative counters -> rate) ─────────────────────
                if tick % 30 == 0 && tick > 0 {
                    networks.refresh_list();
                } else {
                    networks.refresh();
                }
                let (rx, tx) = networks.iter().fold((0u64, 0u64), |a, (_, n)| {
                    (a.0 + n.total_received(), a.1 + n.total_transmitted())
                });
                let (rx_bps, tx_bps) = match last_net {
                    Some((prx, ptx)) => (
                        rx.saturating_sub(prx) as f64 / dt,
                        tx.saturating_sub(ptx) as f64 / dt,
                    ),
                    None => (0.0, 0.0),
                };
                last_net = Some((rx, tx));

                let cpus = sys.cpus();
                let freq = if cpus.is_empty() {
                    0
                } else {
                    cpus.iter().map(|c| c.frequency()).sum::<u64>() / cpus.len() as u64
                };
                let snap = Snapshot {
                    cpu: sys.global_cpu_usage(),
                    cores: cpus.iter().map(|c| c.cpu_usage()).collect(),
                    freq_mhz: freq,
                    mem_used: sys.used_memory(),
                    mem_total: sys.total_memory(),
                    swap_used: sys.used_swap(),
                    swap_total: sys.total_swap(),
                    gpus: gpu_info.clone(),
                    disks: disk_rows.clone(),
                    disk_read_bps: rd_total as f64 / dt,
                    disk_write_bps: wr_total as f64 / dt,
                    net_rx_bps: rx_bps,
                    net_tx_bps: tx_bps,
                    apps,
                    battery: battery.clone(),
                    uptime: System::uptime(),
                    temps: temps.clone(),
                    load,
                };
                let sample_us = cycle_start.elapsed().as_micros() as u64;

                {
                    let mut st = out.lock().unwrap();
                    push(&mut st.hist.cpu, snap.cpu);
                    push(
                        &mut st.hist.ram,
                        if snap.mem_total > 0 {
                            snap.mem_used as f32 / snap.mem_total as f32 * 100.0
                        } else {
                            0.0
                        },
                    );
                    if !snap.gpus.is_empty() {
                        let count = snap.gpus.len() as f32;
                        push(
                            &mut st.hist.gpu,
                            snap.gpus.iter().map(|g| g.util).sum::<f32>() / count,
                        );
                        let total = snap.gpus.iter().map(|g| g.vram_total).sum::<u64>();
                        let used = snap.gpus.iter().map(|g| g.vram_used).sum::<u64>();
                        push(
                            &mut st.hist.vram,
                            if total > 0 {
                                used as f32 / total as f32 * 100.0
                            } else {
                                0.0
                            },
                        );
                    }
                    push(&mut st.hist.disk_r, snap.disk_read_bps as f32);
                    push(&mut st.hist.disk_w, snap.disk_write_bps as f32);
                    push(&mut st.hist.net_rx, snap.net_rx_bps as f32);
                    push(&mut st.hist.net_tx, snap.net_tx_bps as f32);
                    st.snap = snap;
                    st.ready = true;
                    st.seq = st.seq.wrapping_add(1);
                    st.sample_us = sample_us;
                }
                ctx.request_repaint();
                tick = tick.wrapping_add(1);

                let ms = refresh_ms.load(Ordering::Relaxed).max(250);
                let elapsed = cycle_start.elapsed();
                if let Some(rest) = Duration::from_millis(ms).checked_sub(elapsed) {
                    thread::sleep(rest);
                }
            }
        })
        .expect("failed to start sampler thread");

    shared
}

fn read_battery(mgr: &Option<battery::Manager>) -> Option<BatteryInfo> {
    use battery::units::{ratio::percent, time::minute};
    let mgr = mgr.as_ref()?;
    let b = mgr.batteries().ok()?.next()?.ok()?;
    Some(BatteryInfo {
        pct: b.state_of_charge().get::<percent>(),
        state: format!("{:?}", b.state()),
        health: b.state_of_health().get::<percent>(),
        minutes: b
            .time_to_empty()
            .or_else(|| b.time_to_full())
            .map(|t| t.get::<minute>()),
        cycles: b.cycle_count(),
    })
}
