//! User customisation: theme, accent colour, font, UI size, rounding.

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Theme {
    Midnight,
    Graphite,
    Daylight,
    Paper,
    Forest,
    Dusk,
}

pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub panel2: Color32,
    pub line: Color32,
    pub fg: Color32,
    pub mute: Color32,
}

const fn c(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Theme {
    pub const ALL: [Theme; 6] = [Theme::Midnight, Theme::Graphite, Theme::Daylight, Theme::Paper, Theme::Forest, Theme::Dusk];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Midnight => "Midnight",
            Theme::Graphite => "Graphite",
            Theme::Daylight => "Daylight",
            Theme::Paper => "Paper",
            Theme::Forest => "Forest",
            Theme::Dusk => "Dusk",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Midnight => Palette { dark: true, bg: c(11, 15, 23), panel: c(18, 24, 38), panel2: c(24, 32, 50), line: c(36, 48, 73), fg: c(230, 235, 245), mute: c(138, 151, 178) },
            Theme::Graphite => Palette { dark: true, bg: c(18, 18, 18), panel: c(27, 27, 29), panel2: c(36, 36, 39), line: c(51, 51, 56), fg: c(238, 238, 238), mute: c(154, 154, 162) },
            Theme::Daylight => Palette { dark: false, bg: c(238, 241, 246), panel: c(255, 255, 255), panel2: c(243, 245, 249), line: c(219, 224, 234), fg: c(26, 34, 51), mute: c(102, 113, 137) },
            Theme::Paper => Palette { dark: false, bg: c(246, 241, 231), panel: c(255, 250, 240), panel2: c(241, 234, 217), line: c(225, 216, 194), fg: c(58, 50, 38), mute: c(139, 127, 105) },
            Theme::Forest => Palette { dark: true, bg: c(12, 20, 17), panel: c(18, 29, 24), panel2: c(24, 39, 31), line: c(36, 58, 47), fg: c(227, 241, 233), mute: c(133, 163, 147) },
            Theme::Dusk => Palette { dark: true, bg: c(29, 27, 43), panel: c(38, 35, 55), panel2: c(47, 43, 68), line: c(64, 59, 94), fg: c(239, 234, 255), mute: c(167, 159, 201) },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum FontChoice {
    System,
    Monospace,
    Custom,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub accent: [u8; 3],
    pub font: FontChoice,
    pub custom_font_path: String,
    /// Whole-UI zoom factor (pixels-per-point multiplier).
    pub ui_scale: f32,
    /// Base text size in points.
    pub font_size: f32,
    pub rounding: f32,
    pub compact: bool,
    pub refresh_ms: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Midnight,
            accent: [56, 189, 248],
            font: FontChoice::System,
            custom_font_path: String::new(),
            ui_scale: 1.0,
            font_size: 14.0,
            rounding: 12.0,
            compact: false,
            refresh_ms: 1000,
        }
    }
}

impl Settings {
    pub fn pal(&self) -> Palette {
        self.theme.palette()
    }
    pub fn accent(&self) -> Color32 {
        Color32::from_rgb(self.accent[0], self.accent[1], self.accent[2])
    }
    pub fn pad(&self) -> f32 {
        if self.compact { 8.0 } else { 14.0 }
    }

    /// Re-apply every visual option to the egui context.
    pub fn apply(&self, ctx: &egui::Context, reload_fonts: bool) {
        let p = self.pal();
        let acc = self.accent();
        let lum = 0.299 * acc.r() as f32 + 0.587 * acc.g() as f32 + 0.114 * acc.b() as f32;
        let on_acc = if lum > 150.0 { Color32::BLACK } else { Color32::WHITE };
        let r = Rounding::same(self.rounding);

        let mut v = if p.dark { Visuals::dark() } else { Visuals::light() };
        v.panel_fill = p.bg;
        v.window_fill = p.panel;
        v.extreme_bg_color = p.panel2;
        v.faint_bg_color = p.panel2;
        v.hyperlink_color = acc;
        v.selection.bg_fill = acc;
        v.selection.stroke = Stroke::new(1.0_f32, on_acc);
        v.window_rounding = r;
        v.menu_rounding = r;

        let w = &mut v.widgets;
        w.noninteractive.bg_fill = p.panel;
        w.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
        w.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.fg);
        w.noninteractive.rounding = r;
        w.inactive.bg_fill = p.panel2;
        w.inactive.weak_bg_fill = p.panel2;
        w.inactive.bg_stroke = Stroke::new(1.0_f32, p.line);
        w.inactive.fg_stroke = Stroke::new(1.0_f32, p.fg);
        w.inactive.rounding = r;
        w.hovered.bg_fill = p.panel2;
        w.hovered.weak_bg_fill = p.panel2;
        w.hovered.bg_stroke = Stroke::new(1.0_f32, acc);
        w.hovered.fg_stroke = Stroke::new(1.5_f32, p.fg);
        w.hovered.rounding = r;
        w.active.bg_fill = acc;
        w.active.weak_bg_fill = acc;
        w.active.bg_stroke = Stroke::new(1.0_f32, acc);
        w.active.fg_stroke = Stroke::new(1.5_f32, on_acc);
        w.active.rounding = r;
        w.open.rounding = r;
        ctx.set_visuals(v);

        let mut fonts = FontDefinitions::default();
        let mut family = FontFamily::Proportional;
        match self.font {
            FontChoice::System => {}
            FontChoice::Monospace => family = FontFamily::Monospace,
            FontChoice::Custom => {
                if let Ok(bytes) = std::fs::read(&self.custom_font_path) {
                    fonts.font_data.insert("user_font".to_owned(), FontData::from_owned(bytes));
                    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "user_font".to_owned());
                }
            }
        }
        if reload_fonts {
            ctx.set_fonts(fonts);
        }

        let fs = self.font_size;
        let sp = if self.compact { 4.0 } else { 7.0 };
        ctx.style_mut(|st| {
            st.text_styles.insert(TextStyle::Heading, FontId::new(fs * 1.6, family.clone()));
            st.text_styles.insert(TextStyle::Body, FontId::new(fs, family.clone()));
            st.text_styles.insert(TextStyle::Button, FontId::new(fs, family.clone()));
            st.text_styles.insert(TextStyle::Small, FontId::new(fs * 0.8, family.clone()));
            st.text_styles.insert(TextStyle::Monospace, FontId::new(fs, FontFamily::Monospace));
            st.spacing.item_spacing = egui::vec2(sp + 2.0, sp);
            st.spacing.button_padding = egui::vec2(sp + 4.0, sp - 1.0);
        });
    }
}

pub fn ui(ui: &mut egui::Ui, s: &mut Settings) {
    use crate::widgets::card;
    let p = s.pal();
    let snapshot = s.clone();
    card(ui, &snapshot, "Theme", |ui| {
        ui.horizontal_wrapped(|ui| {
            for t in Theme::ALL {
                ui.selectable_value(&mut s.theme, t, t.name());
            }
        });
    });
    ui.add_space(8.0);
    card(ui, &snapshot, "Accent colour", |ui| {
        ui.horizontal_wrapped(|ui| {
            for col in [[56, 189, 248], [52, 211, 153], [244, 114, 182], [251, 191, 36], [167, 139, 250], [251, 113, 133], [249, 115, 22], [34, 211, 238]] {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
                ui.painter().circle_filled(rect.center(), 12.0, Color32::from_rgb(col[0], col[1], col[2]));
                if s.accent == col {
                    ui.painter().circle_stroke(rect.center(), 14.5, Stroke::new(2.0_f32, p.fg));
                }
                if resp.clicked() {
                    s.accent = col;
                }
            }
            ui.label("Custom:");
            ui.color_edit_button_srgb(&mut s.accent);
        });
    });
    ui.add_space(8.0);
    card(ui, &snapshot, "Font", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut s.font, FontChoice::System, "System sans");
            ui.selectable_value(&mut s.font, FontChoice::Monospace, "Monospace");
            ui.selectable_value(&mut s.font, FontChoice::Custom, "Custom .ttf / .otf");
        });
        if s.font == FontChoice::Custom {
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut s.custom_font_path);
            });
            if !std::path::Path::new(&s.custom_font_path).is_file() {
                ui.colored_label(Color32::from_rgb(245, 158, 11), "File not found – using default font.");
            }
        }
        ui.add(egui::Slider::new(&mut s.font_size, 10.0..=24.0).text("Text size"));
    });
    ui.add_space(8.0);
    card(ui, &snapshot, "Interface", |ui| {
        ui.add(egui::Slider::new(&mut s.ui_scale, 0.7..=2.0).text("UI size / ratio"));
        ui.add(egui::Slider::new(&mut s.rounding, 0.0..=24.0).text("Corner roundness"));
        ui.checkbox(&mut s.compact, "Compact layout");
        ui.horizontal(|ui| {
            ui.label("Refresh every");
            for (ms, label) in [(500u64, "0.5 s"), (1000, "1 s"), (2000, "2 s"), (5000, "5 s")] {
                ui.selectable_value(&mut s.refresh_ms, ms, label);
            }
        });
        if ui.button("Reset to defaults").clicked() {
            *s = Settings::default();
        }
    });
}
