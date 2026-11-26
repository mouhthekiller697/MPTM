#![allow(dead_code)]
//! Headless render benchmark for the live tabs:
//!   cargo run --release --example ui_bench
//! It runs egui's layout + tessellation without a GPU, 120 frames per tab.

#[path = "../src/util.rs"]
mod util;
#[path = "../src/gpu.rs"]
mod gpu;
#[path = "../src/sampler.rs"]
mod sampler;
#[path = "../src/settings.rs"]
mod settings;
#[path = "../src/widgets.rs"]
mod widgets;
#[path = "../src/disks.rs"]
mod disks;
#[path = "../src/storage.rs"]
mod storage;
#[path = "../src/specs.rs"]
mod specs;
#[path = "../src/startup.rs"]
mod startup;
#[path = "../src/tabs.rs"]
mod tabs;

use eframe::egui::{self, vec2, Pos2, Rect};
use std::collections::VecDeque;
use std::time::Instant;

fn hist(v: f32) -> VecDeque<f32> {
    (0..sampler::HIST).map(|i| (v + (i as f32 * 0.4).sin() * 8.0).clamp(0.0, 100.0)).collect()
}

fn state() -> sampler::State {
    let apps: Vec<sampler::AppRow> = (0..60)
        .map(|i| sampler::AppRow {
            name: format!("application-{i:02}"),
            procs: 1 + i % 4,
            cpu: (i as f32 * 1.7) % 40.0,
            mem: 40_000_000 + (i as u64) * 31_000_000,
            gpu: if i % 3 == 0 { Some((i % 30) as f32) } else { None },
            net: None,
            disk_read: (i as f64) * 1024.0,
            disk_write: (i as f64) * 512.0,
        })
        .collect();
    let mut h = sampler::History::default();
    h.cpu = hist(24.0);
    h.gpu = hist(30.0);
    h.ram = hist(55.0);
    h.vram = hist(42.0);
    h.disk_r = hist(20.0);
    h.disk_w = hist(9.0);
    h.net_rx = hist(32.0);
    h.net_tx = hist(11.0);
    let snap = sampler::Snapshot {
        cpu: 24.0,
        cores: (0..16).map(|i| (i * 7 % 100) as f32).collect(),
        freq_mhz: 4200,
        mem_used: 17_000_000_000,
        mem_total: 32_000_000_000,
        swap_used: 0,
        swap_total: 0,
        gpu: Some(gpu::GpuInfo {
            name: "Benchmark GPU".into(),
            util: 30.0,
            vram_used: 5_000_000_000,
            vram_total: 12_000_000_000,
            temp: Some(61.0),
            power_w: Some(190.0),
            driver: "000.00".into(),
        }),
        disks: vec![sampler::DiskRow {
            name: "nvme0n1".into(),
            mount: "/".into(),
            fs: "ext4".into(),
            kind: "SSD".into(),
            total: 931_000_000_000,
            avail: 300_000_000_000,
            removable: false,
        }],
        disk_read_bps: 22_000_000.0,
        disk_write_bps: 4_000_000.0,
        net_rx_bps: 1_400_000.0,
        net_tx_bps: 220_000.0,
        apps,
        battery: None,
        uptime: 98_000,
        temps: vec![("CPU package".into(), 61.0)],
        load: (0.8, 1.1, 0.9),
    };
    sampler::State { snap, hist: h, ready: true, seq: 1, sample_us: 1500 }
}

fn main() {
    let ctx = egui::Context::default();
    let s = settings::Settings::default();
    s.apply(&ctx, true);
    let st = state();
    let mut input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 900.0))),
        ..Default::default()
    };
    input.time = Some(0.0);
    let mut apps_ui = tabs::AppsUi::default();
    let mut disks_ui = tabs::DisksUi::default();
    let mut storage_ui = tabs::StorageUi::default();
    let mut specs_ui = tabs::SpecsUi::default();
    let mut startup_ui = tabs::StartupUi::default();

    let mut run = |name: &str, mut draw: Box<dyn FnMut(&mut egui::Ui)>| {
        let mut samples = vec![];
        for frame in 0..120 {
            input.time = Some(frame as f64 / 60.0);
            let t = Instant::now();
            let _ = ctx.run(input.clone(), |ctx| {
                egui::SidePanel::left("nav").exact_width(210.0).show(ctx, |ui| {
                    ui.label("Ferrum Monitor");
                });
                egui::CentralPanel::default().show(ctx, |ui| draw(ui));
            });
            samples.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = samples[samples.len() / 2];
        let p95 = samples[samples.len() * 95 / 100];
        println!("{name:<12} median {med:.3} ms   p95 {p95:.3} ms");
    };

    run("general", Box::new(|ui| tabs::general(ui, &s, &st)));
    run("applications", Box::new(|ui| tabs::apps(ui, &s, &st, &mut apps_ui)));
    run("disks", Box::new(|ui| tabs::disks_tab(ui, &s, &st, &mut disks_ui)));
    run("storage", Box::new(|ui| tabs::storage_tab(ui, &s, &st, &mut storage_ui)));
    run("specs", Box::new(|ui| tabs::specs_tab(ui, &s, &st, &mut specs_ui)));
    run("startup", Box::new(|ui| tabs::startup_tab(ui, &s, &mut startup_ui)));
}
