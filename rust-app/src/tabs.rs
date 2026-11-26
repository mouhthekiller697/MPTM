//! One drawing function (plus a little state) per tab.

use crate::disks::{self, PhysDisk};
use crate::sampler::{AppRow, DiskRow, State};
use crate::settings::Settings;
use crate::specs::{self, Specs};
use crate::startup;
use crate::storage::{self, ScanHandle};
use crate::widgets::*;
use eframe::egui::{self, pos2, vec2, Color32, Rect, RichText, Rounding, Sense, Ui};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const C_CPU: Color32 = Color32::from_rgb(56, 189, 248);
const C_GPU: Color32 = Color32::from_rgb(167, 139, 250);
const C_RAM: Color32 = Color32::from_rgb(52, 211, 153);
const C_VRAM: Color32 = Color32::from_rgb(244, 114, 182);
const C_DISK: Color32 = Color32::from_rgb(251, 191, 36);
const C_NET: Color32 = Color32::from_rgb(34, 211, 238);
const PALETTE: [Color32; 8] = [
    Color32::from_rgb(56, 189, 248),
    Color32::from_rgb(52, 211, 153),
    Color32::from_rgb(244, 114, 182),
    Color32::from_rgb(251, 191, 36),
    Color32::from_rgb(167, 139, 250),
    Color32::from_rgb(251, 113, 133),
    Color32::from_rgb(34, 211, 238),
    Color32::from_rgb(249, 115, 22),
];

fn peak(q: &[&std::collections::VecDeque<f32>], floor: f32) -> f32 {
    q.iter().flat_map(|d| d.iter().copied()).fold(floor, f32::max)
}

fn stacked(ui: &mut Ui, s: &Settings, parts: &[(f32, Color32)], height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, Rounding::same((s.rounding * 0.5).min(10.0)), s.pal().panel2);
    let mut x = rect.left();
    for (f, c) in parts {
        let w = (rect.width() * f).max(2.0).min(rect.right() - x);
        if w <= 0.0 {
            break;
        }
        p.rect_filled(Rect::from_min_size(pos2(x, rect.top()), vec2(w, height)), Rounding::ZERO, *c);
        x += w + 1.0;
    }
}

fn primary_disk(d: &[DiskRow]) -> Option<&DiskRow> {
    d.iter()
        .find(|x| x.mount == "/" || x.mount.starts_with("C:"))
        .or_else(|| d.iter().max_by_key(|x| x.total))
}

fn metric(ui: &mut Ui, s: &Settings, title: &str, frac: f32, label: &str, sub: &str, series: &[(&std::collections::VecDeque<f32>, Color32)], max: f32, color: Color32) {
    card(ui, s, title, |ui| {
        ui.horizontal(|ui| {
            ring(ui, s, frac, color, 78.0, label);
            ui.vertical(|ui| {
                ui.label(mute(s, sub));
                spark(ui, s, series, max, 54.0);
            });
        });
    });
}

// ───────────────────────────── General ─────────────────────────────

pub fn general(ui: &mut Ui, s: &Settings, st: &State) {
    let sn = &st.snap;
    let h = &st.hist;
    if !st.ready {
        ui.label("Collecting first sample…");
        return;
    }
    let ram = if sn.mem_total > 0 { sn.mem_used as f32 / sn.mem_total as f32 } else { 0.0 };
    let temp = sn.temps.iter().map(|t| t.1).fold(f32::MIN, f32::max);
    let cpu_sub = format!("{} threads · {:.2} GHz{}", sn.cores.len(), sn.freq_mhz as f32 / 1000.0, if temp > 0.0 { format!(" · {temp:.0}°C") } else { String::new() });

    ui.columns(3, |c| {
        metric(&mut c[0], s, "CPU", sn.cpu / 100.0, &format!("{:.0}%", sn.cpu), &cpu_sub, &[(&h.cpu, C_CPU)], 100.0, C_CPU);
        match &sn.gpu {
            Some(g) => {
                let sub = format!("{}{}{}", g.name, g.temp.map(|t| format!(" · {t:.0}°C")).unwrap_or_default(), g.power_w.map(|w| format!(" · {w:.0} W")).unwrap_or_default());
                metric(&mut c[1], s, "GPU", g.util / 100.0, &format!("{:.0}%", g.util), &sub, &[(&h.gpu, C_GPU)], 100.0, C_GPU);
            }
            None => card(&mut c[1], s, "GPU", |ui| {
                ui.label(mute(s, "No supported GPU telemetry found."));
                ui.label(mute(s, "NVIDIA needs nvidia-smi; AMD is read from sysfs on Linux."));
            }),
        }
        metric(&mut c[2], s, "Memory (RAM)", ram, &format!("{:.0}%", ram * 100.0), &format!("{} / {}  ·  swap {}", bytes(sn.mem_used), bytes(sn.mem_total), bytes(sn.swap_used)), &[(&h.ram, C_RAM)], 100.0, C_RAM);
    });
    ui.add_space(8.0);

    ui.columns(3, |c| {
        match &sn.gpu {
            Some(g) if g.vram_total > 0 => {
                let f = g.vram_used as f32 / g.vram_total as f32;
                metric(&mut c[0], s, "GPU memory (VRAM)", f, &format!("{:.0}%", f * 100.0), &format!("{} / {}", bytes(g.vram_used), bytes(g.vram_total)), &[(&h.vram, C_VRAM)], 100.0, C_VRAM);
            }
            _ => card(&mut c[0], s, "GPU memory (VRAM)", |ui| {
                ui.label(mute(s, "Not available."));
            }),
        }
        card(&mut c[1], s, "Disk", |ui| {
            if let Some(d) = primary_disk(&sn.disks) {
                let used = d.total.saturating_sub(d.avail);
                let f = if d.total > 0 { used as f32 / d.total as f32 } else { 0.0 };
                ui.label(RichText::new(format!("{} used of {}", bytes(used), bytes(d.total))).strong());
                ui.label(mute(s, format!("{} · {} · {}", d.mount, d.kind, d.fs)));
                bar(ui, s, f, C_DISK, 10.0);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.colored_label(C_CPU, format!("▼ read {}", rate(sn.disk_read_bps)));
                ui.colored_label(C_VRAM, format!("▲ write {}", rate(sn.disk_write_bps)));
            });
            let m = peak(&[&h.disk_r, &h.disk_w], 1_000_000.0);
            spark(ui, s, &[(&h.disk_r, C_CPU), (&h.disk_w, C_VRAM)], m, 54.0);
        });
        card(&mut c[2], s, "Network", |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(C_NET, format!("▼ {}", rate(sn.net_rx_bps)));
                ui.colored_label(C_VRAM, format!("▲ {}", rate(sn.net_tx_bps)));
            });
            let m = peak(&[&h.net_rx, &h.net_tx], 100_000.0);
            spark(ui, s, &[(&h.net_rx, C_NET), (&h.net_tx, C_VRAM)], m, 78.0);
        });
    });
    ui.add_space(8.0);

    ui.columns(2, |c| {
        card(&mut c[0], s, "Battery", |ui| match &sn.battery {
            Some(b) => {
                ui.horizontal(|ui| {
                    ring(ui, s, b.pct / 100.0, C_RAM, 84.0, &format!("{:.0}%", b.pct));
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&b.state).strong());
                        ui.label(mute(s, format!("Health {:.0}%", b.health)));
                        if let Some(m) = b.minutes {
                            ui.label(mute(s, format!("Time left {}", duration((m * 60.0) as u64))));
                        }
                        if let Some(cy) = b.cycles {
                            ui.label(mute(s, format!("{cy} charge cycles")));
                        }
                    });
                });
            }
            None => {
                ui.label(mute(s, "No battery detected (desktop PC)."));
            }
        });
        card(&mut c[1], s, "System", |ui| {
            ui.label(format!("Uptime  {}", duration(sn.uptime)));
            ui.label(mute(s, format!("Load average  {:.2}  {:.2}  {:.2}", sn.load.0, sn.load.1, sn.load.2)));
            for (n, t) in sn.temps.iter().take(4) {
                ui.label(mute(s, format!("{n}: {t:.0}°C")));
            }
        });
    });
    ui.add_space(8.0);

    card(ui, s, "Per-thread CPU load", |ui| {
        ui.horizontal_wrapped(|ui| {
            for (i, c) in sn.cores.iter().enumerate() {
                ui.vertical(|ui| {
                    let (rect, resp) = ui.allocate_exact_size(vec2(24.0, 52.0), Sense::hover());
                    let p = ui.painter_at(rect);
                    p.rect_filled(rect, Rounding::same(4.0), s.pal().panel2);
                    let fh = rect.height() * (c / 100.0).clamp(0.0, 1.0);
                    p.rect_filled(Rect::from_min_max(pos2(rect.left(), rect.bottom() - fh), rect.right_bottom()), Rounding::same(4.0), s.accent());
                    resp.on_hover_text(format!("Thread {i}: {c:.0}%"));
                    ui.label(RichText::new(format!("{i}")).small().color(s.pal().mute));
                });
            }
        });
    });
}

// ───────────────────────────── Applications ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Cpu,
    Mem,
    Gpu,
    Net,
    Disk,
}

pub struct AppsUi {
    pub sort: SortKey,
    pub desc: bool,
    pub filter: String,
}

impl Default for AppsUi {
    fn default() -> Self {
        Self { sort: SortKey::Cpu, desc: true, filter: String::new() }
    }
}


/// Case-insensitive substring test without allocating (used for the app filter).
fn contains_ci(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let (h, n) = (hay.as_bytes(), needle.as_bytes());
    h.len() >= n.len() && h.windows(n.len()).any(|w| w.eq_ignore_ascii_case(n))
}

/// Case-insensitive name ordering without allocating per comparison.
fn cmp_name_ci(a: &str, b: &str) -> Ordering {
    a.bytes().map(|c| c.to_ascii_lowercase()).cmp(b.bytes().map(|c| c.to_ascii_lowercase()))
}

fn sort_header(ui: &mut Ui, state: &mut AppsUi, k: SortKey, label: &str) {
    let arrow = if state.sort == k { if state.desc { " ▼" } else { " ▲" } } else { "" };
    if ui.selectable_label(state.sort == k, format!("{label}{arrow}")).clicked() {
        if state.sort == k {
            state.desc = !state.desc;
        } else {
            state.sort = k;
            state.desc = k != SortKey::Name;
        }
    }
}

fn f32cmp(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

pub fn apps(ui: &mut Ui, s: &Settings, st: &State, ui_state: &mut AppsUi) {
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.add(egui::TextEdit::singleline(&mut ui_state.filter).hint_text("Filter applications…").desired_width(260.0));
        ui.label(mute(s, "Processes with the same name are grouped into one application."));
    });
    ui.add_space(6.0);

    let mut rows: Vec<&AppRow> = st.snap.apps.iter().filter(|a| contains_ci(&a.name, &ui_state.filter)).collect();
    let (key, desc) = (ui_state.sort, ui_state.desc);
    rows.sort_by(|a, b| {
        let o = match key {
            SortKey::Name => cmp_name_ci(&a.name, &b.name),
            SortKey::Cpu => f32cmp(a.cpu, b.cpu),
            SortKey::Mem => a.mem.cmp(&b.mem),
            SortKey::Gpu => f32cmp(a.gpu.unwrap_or(0.0), b.gpu.unwrap_or(0.0)),
            SortKey::Net => a.net.unwrap_or(0.0).partial_cmp(&b.net.unwrap_or(0.0)).unwrap_or(Ordering::Equal),
            SortKey::Disk => (a.disk_read + a.disk_write).partial_cmp(&(b.disk_read + b.disk_write)).unwrap_or(Ordering::Equal),
        };
        if desc { o.reverse() } else { o }
    });

    let total_mem = st.snap.mem_total.max(1) as f32;
    card(ui, s, "Resource use per application", |ui| {
        egui::Grid::new("apps_grid").striped(true).num_columns(8).min_col_width(64.0).spacing([16.0, 6.0]).show(ui, |ui| {
            sort_header(ui, ui_state, SortKey::Name, "Application");
            ui.label(mute(s, "Procs"));
            sort_header(ui, ui_state, SortKey::Cpu, "CPU");
            sort_header(ui, ui_state, SortKey::Mem, "Memory");
            sort_header(ui, ui_state, SortKey::Gpu, "GPU");
            sort_header(ui, ui_state, SortKey::Net, "Network");
            sort_header(ui, ui_state, SortKey::Disk, "Disk (r + w)");
            ui.label(mute(s, "Disk read"));
            ui.end_row();

            for a in rows.iter().take(400) {
                ui.label(RichText::new(&a.name).strong());
                ui.label(mute(s, a.procs.to_string()));
                ui.label(RichText::new(format!("{:.1}%", a.cpu)).color(usage_color(a.cpu / 100.0 * 2.0, s.accent())));
                ui.label(RichText::new(bytes(a.mem)).color(usage_color(a.mem as f32 / total_mem * 3.0, s.accent())));
                match a.gpu {
                    Some(g) => ui.label(format!("{g:.0}%")),
                    None => ui.label(mute(s, "—")).on_hover_text("Per-application GPU data needs NVIDIA's nvidia-smi."),
                };
                match a.net {
                    Some(n) => ui.label(rate(n)),
                    None => ui.label(mute(s, "—")).on_hover_text("Per-application network accounting needs a platform back-end (eBPF / ETW / nettop)."),
                };
                ui.label(rate(a.disk_read + a.disk_write));
                ui.label(rate(a.disk_read));
                ui.end_row();
            }
        });
    });
}

// ───────────────────────────── Disk manager ─────────────────────────────

pub struct DisksUi {
    pub list: Option<Vec<PhysDisk>>,
    pub sel: usize,
    pub health: Arc<Mutex<HashMap<String, String>>>,
    pub msg: String,
}

impl Default for DisksUi {
    fn default() -> Self {
        Self { list: None, sel: 0, health: Arc::new(Mutex::new(HashMap::new())), msg: String::new() }
    }
}

pub fn disks_tab(ui: &mut Ui, s: &Settings, st: &State, u: &mut DisksUi) {
    if u.list.is_none() {
        u.list = Some(disks::list());
    }
    ui.horizontal(|ui| {
        if ui.button("⟳ Refresh").clicked() {
            u.list = Some(disks::list());
        }
        if ui.button("🛠 Open partition tool").clicked() {
            u.msg = if disks::open_partition_tool() {
                "Opened the system partition editor (it will ask for administrator rights).".into()
            } else {
                "No partition editor found – install GParted or GNOME Disks.".into()
            };
        }
        if !u.msg.is_empty() {
            ui.label(mute(s, u.msg.as_str()));
        }
    });
    ui.add_space(6.0);

    let list = u.list.clone().unwrap_or_default();
    if list.is_empty() {
        ui.label("No disks found.");
        return;
    }
    u.sel = u.sel.min(list.len() - 1);
    ui.horizontal_wrapped(|ui| {
        for (i, d) in list.iter().enumerate() {
            if ui.selectable_label(u.sel == i, format!("{} · {} · {}", d.dev, d.kind, bytes(d.size))).clicked() {
                u.sel = i;
            }
        }
    });
    ui.add_space(6.0);
    let d = &list[u.sel];

    card(ui, s, &format!("{} — partition layout {}", d.model, if d.table.is_empty() { String::new() } else { format!("({})", d.table) }), |ui| {
        let parts: Vec<(f32, Color32)> = d.parts.iter().enumerate().map(|(i, p)| (p.size as f32 / d.size.max(1) as f32, PALETTE[i % PALETTE.len()])).collect();
        stacked(ui, s, &parts, 34.0);
        ui.add_space(8.0);
        egui::Grid::new("parts").striped(true).num_columns(6).spacing([18.0, 6.0]).show(ui, |ui| {
            for h in ["", "Partition", "Filesystem", "Mount", "Size", "Used"] {
                ui.label(mute(s, h));
            }
            ui.end_row();
            for (i, p) in d.parts.iter().enumerate() {
                ui.colored_label(PALETTE[i % PALETTE.len()], "■");
                ui.label(&p.name);
                ui.label(mute(s, if p.fstype.is_empty() { "—" } else { p.fstype.as_str() }));
                ui.label(mute(s, if p.mount.is_empty() { "—" } else { p.mount.as_str() }));
                ui.label(bytes(p.size));
                match p.used {
                    Some(used) => {
                        ui.vertical(|ui| {
                            ui.set_width(160.0);
                            bar(ui, s, used as f32 / p.size.max(1) as f32, s.accent(), 8.0);
                            ui.label(RichText::new(format!("{} ({:.0}%)", bytes(used), used as f32 / p.size.max(1) as f32 * 100.0)).small().color(s.pal().mute));
                        });
                    }
                    None => {
                        ui.label(mute(s, "—"));
                    }
                }
                ui.end_row();
            }
        });
    });
    ui.add_space(8.0);

    card(ui, s, "Health (S.M.A.R.T.)", |ui| {
        let key = d.dev.clone();
        if ui.button("Check health").clicked() {
            let store = u.health.clone();
            let dev = key.clone();
            store.lock().unwrap().insert(key.clone(), "Reading…".into());
            std::thread::spawn(move || {
                let r = disks::health(&dev);
                store.lock().unwrap().insert(dev, r);
            });
        }
        match u.health.lock().unwrap().get(&key) {
            Some(txt) => {
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    ui.label(RichText::new(txt).monospace().small());
                });
            }
            None => {
                ui.label(mute(s, "Press “Check health” to query the drive (needs smartmontools / admin rights on Linux & macOS)."));
            }
        }
    });
    ui.add_space(8.0);

    card(ui, s, "Mounted volumes", |ui| {
        for v in &st.snap.disks {
            let used = v.total.saturating_sub(v.avail);
            ui.label(format!("{}  ({}, {}{})", v.mount, v.fs, v.kind, if v.removable { ", removable" } else { "" }));
            bar(ui, s, used as f32 / v.total.max(1) as f32, s.accent(), 8.0);
            ui.label(RichText::new(format!("{} used · {} free · {} total", bytes(used), bytes(v.avail), bytes(v.total))).small().color(s.pal().mute));
            ui.add_space(4.0);
        }
    });
}

// ───────────────────────────── Storage analyser ─────────────────────────────

#[derive(Default)]
pub struct StorageUi {
    pub path: String,
    pub handle: Option<ScanHandle>,
    pub stack: Vec<PathBuf>,
    pub apps_view: bool,
    pub apps: Arc<Mutex<Option<Vec<(String, u64)>>>>,
    pub apps_started: bool,
}

impl StorageUi {
    fn scan(&mut self, ctx: &egui::Context, path: PathBuf) {
        if let Some(h) = &self.handle {
            h.cancel();
        }
        self.path = path.display().to_string();
        self.handle = Some(storage::start(path, ctx.clone()));
    }
}

pub fn storage_tab(ui: &mut Ui, s: &Settings, st: &State, u: &mut StorageUi) {
    let ctx = ui.ctx().clone();
    if u.path.is_empty() {
        u.path = primary_disk(&st.snap.disks).map(|d| d.mount.clone()).unwrap_or_else(|| "/".into());
    }
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut u.apps_view, false, "📁 Folders");
        ui.selectable_value(&mut u.apps_view, true, "📦 Applications");
        ui.separator();
        if !u.apps_view {
            for d in &st.snap.disks {
                if ui.button(&d.mount).clicked() {
                    u.stack.clear();
                    u.scan(&ctx, PathBuf::from(&d.mount));
                }
            }
        }
    });
    ui.add_space(6.0);

    if u.apps_view {
        if !u.apps_started {
            u.apps_started = true;
            let slot = u.apps.clone();
            std::thread::spawn(move || {
                let v = storage::installed_apps();
                *slot.lock().unwrap() = Some(v);
            });
        }
        card(ui, s, "Storage used by installed applications", |ui| {
            let g = u.apps.lock().unwrap();
            match g.as_ref() {
                None => {
                    ui.label(mute(s, "Reading package database…"));
                }
                Some(v) if v.is_empty() => {
                    ui.label(mute(s, "No package manager data found on this system."));
                }
                Some(v) => {
                    let max = v[0].1.max(1) as f32;
                    egui::Grid::new("app_sizes").striped(true).num_columns(3).spacing([16.0, 5.0]).show(ui, |ui| {
                        for (name, size) in v.iter().take(200) {
                            ui.label(name);
                            ui.scope(|ui| {
                                ui.set_width(240.0);
                                bar(ui, s, *size as f32 / max, s.accent(), 8.0);
                            });
                            ui.label(bytes(*size));
                            ui.end_row();
                        }
                    });
                }
            }
        });
        return;
    }

    ui.horizontal(|ui| {
        if ui.add_enabled(u.stack.len() > 0, egui::Button::new("⬆ Up")).clicked() {
            if let Some(prev) = u.stack.pop() {
                u.scan(&ctx, prev);
            }
        }
        let input = ui.add(
            egui::TextEdit::singleline(&mut u.path)
                .desired_width(360.0)
                .hint_text("Folder to analyse")
                .cursor_at_end(false),
        );
        // Enter scans straight away, so a typed path needs only one action.
        let submit = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui.button("Scan").clicked() || submit {
            u.stack.clear();
            let p = PathBuf::from(u.path.clone());
            u.scan(&ctx, p);
        }
    });
    ui.add_space(6.0);

    let Some(handle) = &u.handle else {
        card(ui, s, "", |ui| {
            ui.label("Pick a disk above or type a folder path, then press Scan. Folders are measured in parallel.");
        });
        return;
    };
    let (entries, running, done, total, root) = {
        let g = handle.state.lock().unwrap();
        (g.view.clone(), g.running, g.done, g.total, g.root.clone())
    };
    let sum: u64 = entries.iter().map(|e| e.size).sum();
    let mut drill: Option<PathBuf> = None;

    card(ui, s, &format!("{} — {}", root.display(), bytes(sum)), |ui| {
        if running {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(mute(s, format!("Scanning… {done} / {total} items")));
            });
        }
        let parts: Vec<(f32, Color32)> = entries.iter().take(8).enumerate().map(|(i, e)| (e.size as f32 / sum.max(1) as f32, PALETTE[i])).collect();
        stacked(ui, s, &parts, 26.0);
        ui.add_space(8.0);
        let max = entries.first().map(|e| e.size).unwrap_or(1).max(1) as f32;
        egui::Grid::new("folder_sizes").striped(true).num_columns(4).spacing([16.0, 5.0]).show(ui, |ui| {
            for (i, e) in entries.iter().take(300).enumerate() {
                let col = PALETTE[i % PALETTE.len()];
                if e.is_dir {
                    if ui.link(format!("📁 {}", e.name)).clicked() {
                        drill = Some(e.path.clone());
                    }
                } else {
                    ui.label(format!("📄 {}", e.name));
                }
                ui.scope(|ui| {
                    ui.set_width(240.0);
                    bar(ui, s, e.size as f32 / max, col, 8.0);
                });
                ui.label(bytes(e.size));
                ui.label(mute(s, format!("{:.1}%", e.size as f32 / sum.max(1) as f32 * 100.0)));
                ui.end_row();
            }
        });
    });
    if let Some(p) = drill {
        u.stack.push(root);
        u.scan(&ctx, p);
    }
}

// ───────────────────────────── Specs ─────────────────────────────

#[derive(Default)]
pub struct SpecsUi {
    pub data: Option<Specs>,
    pub sub: usize,
}

pub fn specs_tab(ui: &mut Ui, s: &Settings, st: &State, u: &mut SpecsUi) {
    let data = u.data.get_or_insert_with(specs::collect).clone();
    let names = ["CPU", "Mainboard", "Memory", "GPU", "System"];
    ui.horizontal_wrapped(|ui| {
        for (i, n) in names.iter().enumerate() {
            ui.selectable_value(&mut u.sub, i, *n);
        }
    });
    ui.add_space(6.0);

    let rows: Vec<(String, String)> = match u.sub {
        0 => {
            let mut r = data.cpu.clone();
            r.push(("Current clock (avg)".into(), format!("{} MHz", st.snap.freq_mhz)));
            r.push(("Current load".into(), format!("{:.1}%", st.snap.cpu)));
            r
        }
        1 => data.board.clone(),
        2 => {
            let mut r = data.memory.clone();
            r.push(("In use".into(), bytes(st.snap.mem_used)));
            r.push(("Swap in use".into(), bytes(st.snap.swap_used)));
            r
        }
        3 => match &st.snap.gpu {
            Some(g) => {
                let mut r = vec![("Name".to_string(), g.name.clone()), ("Driver".into(), g.driver.clone()), ("VRAM".into(), bytes(g.vram_total))];
                if let Some(t) = g.temp {
                    r.push(("Temperature".into(), format!("{t:.0} °C")));
                }
                if let Some(w) = g.power_w {
                    r.push(("Power draw".into(), format!("{w:.0} W")));
                }
                r
            }
            None => vec![("GPU".into(), "No telemetry source available".into())],
        },
        _ => data.system.clone(),
    };

    card(ui, s, &format!("{} information", names[u.sub.min(4)]), |ui| {
        egui::Grid::new("specs_grid").striped(true).num_columns(2).spacing([30.0, 7.0]).show(ui, |ui| {
            for (k, v) in &rows {
                ui.label(mute(s, k));
                ui.label(RichText::new(v).strong());
                ui.end_row();
            }
        });
    });
    if u.sub == 0 {
        ui.add_space(8.0);
        card(ui, s, "Sensors", |ui| {
            if st.snap.temps.is_empty() {
                ui.label(mute(s, "No temperature sensors exposed."));
            }
            for (n, t) in &st.snap.temps {
                ui.label(format!("{n}: {t:.0} °C"));
            }
        });
    }
}

// ───────────────────────────── Startup ─────────────────────────────

#[derive(Default)]
pub struct StartupUi {
    pub items: Option<Vec<startup::Item>>,
    pub msg: String,
}

pub fn startup_tab(ui: &mut Ui, s: &Settings, u: &mut StartupUi) {
    if u.items.is_none() {
        u.items = Some(startup::list());
    }
    ui.horizontal(|ui| {
        if ui.button("⟳ Refresh").clicked() {
            u.items = Some(startup::list());
        }
        if !u.msg.is_empty() {
            ui.colored_label(Color32::from_rgb(245, 158, 11), u.msg.as_str());
        }
    });
    ui.add_space(6.0);
    let mut toggled: Option<(usize, bool)> = None;
    let items = u.items.clone().unwrap_or_default();
    let enabled = items.iter().filter(|i| i.enabled).count();

    card(ui, s, &format!("Startup applications — {enabled} of {} enabled", items.len()), |ui| {
        if items.is_empty() {
            ui.label(mute(s, "No startup entries found."));
        }
        egui::Grid::new("startup").striped(true).num_columns(4).spacing([18.0, 8.0]).show(ui, |ui| {
            for (i, it) in items.iter().enumerate() {
                ui.label(RichText::new(&it.name).strong());
                ui.label(mute(s, it.command.as_str())).on_hover_text(it.command.clone());
                ui.label(mute(s, &it.location));
                let mut on = it.enabled;
                if ui.add_enabled(it.can_toggle, egui::Checkbox::new(&mut on, "Enabled")).changed() {
                    toggled = Some((i, on));
                }
                ui.end_row();
            }
        });
    });
    if let Some((i, on)) = toggled {
        match startup::set_enabled(&items[i], on) {
            Ok(()) => {
                u.msg.clear();
                u.items = Some(startup::list());
            }
            Err(e) => u.msg = e,
        }
    }
}
