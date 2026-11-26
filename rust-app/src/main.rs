//! MPSM (Multipurpose System Manager) – cross-platform system manager written in Rust.
//!
//! Tabs: General · Applications · Disk manager · Storage analyser · Specs · Startup · Appearance

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod disks;
mod gpu;
mod icons;
mod sampler;
mod settings;
mod specs;
mod startup;
mod storage;
mod tabs;
mod util;
mod widgets;

use eframe::egui::{self, vec2, Frame, Margin, RichText, Stroke};
use settings::Settings;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    General,
    Apps,
    Disks,
    Storage,
    Specs,
    Startup,
    Appearance,
}

impl Tab {
    const ALL: [Tab; 7] = [
        Tab::General,
        Tab::Apps,
        Tab::Disks,
        Tab::Storage,
        Tab::Specs,
        Tab::Startup,
        Tab::Appearance,
    ];
    fn label(self) -> &'static str {
        match self {
            Tab::General => "📊  General",
            Tab::Apps => "🧩  Applications",
            Tab::Disks => "💽  Disk manager",
            Tab::Storage => "🗂  Storage analyser",
            Tab::Specs => "🔧  Specs",
            Tab::Startup => "🚀  Startup",
            Tab::Appearance => "🎨  Appearance",
        }
    }
    fn title(self) -> &'static str {
        self.label()
            .trim_start_matches(|c: char| !c.is_alphabetic())
    }
}

struct App {
    tab: Tab,
    settings: Settings,
    applied: Option<Settings>,
    state: Arc<Mutex<sampler::State>>,
    refresh: Arc<AtomicU64>,
    apps_ui: tabs::AppsUi,
    disks_ui: tabs::DisksUi,
    storage_ui: tabs::StorageUi,
    specs_ui: tabs::SpecsUi,
    startup_ui: tabs::StartupUi,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "settings"))
            .unwrap_or_default();
        let refresh = Arc::new(AtomicU64::new(settings.refresh_ms));
        let state = sampler::spawn(cc.egui_ctx.clone(), refresh.clone());
        Self {
            tab: Tab::General,
            settings,
            applied: None,
            state,
            refresh,
            apps_ui: Default::default(),
            disks_ui: Default::default(),
            storage_ui: Default::default(),
            specs_ui: Default::default(),
            startup_ui: Default::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Re-apply theme / fonts only when something changed.
        if self.applied.as_ref() != Some(&self.settings) {
            let fonts_changed = match &self.applied {
                None => true,
                Some(a) => {
                    a.font != self.settings.font
                        || a.custom_font_path != self.settings.custom_font_path
                }
            };
            self.settings.apply(ctx, fonts_changed);
            self.refresh
                .store(self.settings.refresh_ms, Ordering::Relaxed);
            self.applied = Some(self.settings.clone());
        }
        // Zoom is applied after the pointer is released so a slider doesn't jitter while dragging.
        if !ctx.input(|i| i.pointer.any_down())
            && (ctx.zoom_factor() - self.settings.ui_scale).abs() > f32::EPSILON
        {
            ctx.set_zoom_factor(self.settings.ui_scale);
        }

        let s = self.settings.clone();
        let p = s.pal();
        let state = self.state.clone();

        egui::SidePanel::left("nav")
            .exact_width(210.0)
            .resizable(false)
            .frame(
                Frame::none()
                    .fill(p.panel)
                    .stroke(Stroke::new(1.0_f32, p.line))
                    .inner_margin(Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(RichText::new("MPSM").heading().strong().color(s.accent()));
                ui.label(RichText::new("system task manager").small().color(p.mute));
                ui.add_space(14.0);
                for t in Tab::ALL {
                    let w = ui.available_width();
                    if ui
                        .add_sized(
                            vec2(w, 36.0),
                            egui::SelectableLabel::new(self.tab == t, RichText::new(t.label())),
                        )
                        .clicked()
                    {
                        self.tab = t;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    let st = state.lock().unwrap();
                    ui.label(
                        RichText::new(format!("Uptime {}", widgets::duration(st.snap.uptime)))
                            .small()
                            .color(p.mute),
                    );
                    ui.label(
                        RichText::new(format!(
                            "CPU {:.0}% · RAM {}",
                            st.snap.cpu,
                            if st.snap.mem_total > 0 {
                                format!(
                                    "{:.0}%",
                                    st.snap.mem_used as f64 / st.snap.mem_total as f64 * 100.0
                                )
                            } else {
                                "–".into()
                            }
                        ))
                        .small()
                        .color(p.mute),
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(p.bg)
                    .inner_margin(Margin::same(if s.compact { 10.0 } else { 18.0 })),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new(self.tab.title()).heading().strong());
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let st = state.lock().unwrap();
                        match self.tab {
                            Tab::General => tabs::general(ui, &s, &st),
                            Tab::Apps => tabs::apps(ui, &s, &st, &mut self.apps_ui),
                            Tab::Disks => tabs::disks_tab(ui, &s, &st, &mut self.disks_ui),
                            Tab::Storage => tabs::storage_tab(ui, &s, &st, &mut self.storage_ui),
                            Tab::Specs => tabs::specs_tab(ui, &s, &st, &mut self.specs_ui),
                            Tab::Startup => tabs::startup_tab(ui, &s, &mut self.startup_ui),
                            Tab::Appearance => {
                                drop(st);
                                settings::ui(ui, &mut self.settings);
                            }
                        }
                    });
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", &self.settings);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MPSM — Multipurpose System Manager")
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([820.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MPSM — Multipurpose System Manager",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
