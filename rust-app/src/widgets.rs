//! Small reusable painter-based widgets: cards, sparklines, rings and bars.

use crate::settings::Settings;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Frame, Margin, RichText, Rounding, Sense, Shape, Stroke, Ui};
use std::collections::VecDeque;

pub fn card<R>(ui: &mut Ui, s: &Settings, title: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    let p = s.pal();
    Frame::none()
        .fill(p.panel)
        .stroke(Stroke::new(1.0_f32, p.line))
        .rounding(Rounding::same(s.rounding))
        .inner_margin(Margin::same(s.pad()))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if !title.is_empty() {
                ui.label(RichText::new(title.to_uppercase()).small().strong().color(p.mute));
                ui.add_space(4.0);
            }
            add(ui)
        })
        .inner
}

pub fn mute(s: &Settings, text: impl Into<String>) -> RichText {
    RichText::new(text.into()).color(s.pal().mute)
}

pub fn usage_color(frac: f32, base: Color32) -> Color32 {
    if frac > 0.9 {
        Color32::from_rgb(244, 63, 94)
    } else if frac > 0.75 {
        Color32::from_rgb(245, 158, 11)
    } else {
        base
    }
}

pub fn bar(ui: &mut Ui, s: &Settings, frac: f32, color: Color32, height: f32) {
    let frac = frac.clamp(0.0, 1.0);
    let w = ui.available_width().max(20.0);
    let (rect, _) = ui.allocate_exact_size(vec2(w, height), Sense::hover());
    let r = Rounding::same(height / 2.0);
    ui.painter().rect_filled(rect, r, s.pal().panel2);
    let mut fill = rect;
    fill.set_width((rect.width() * frac).max(height.min(rect.width())).min(rect.width()));
    if frac > 0.0 {
        ui.painter().rect_filled(fill, r, usage_color(frac, color));
    }
}

pub fn ring(ui: &mut Ui, s: &Settings, frac: f32, color: Color32, size: f32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let center = rect.center();
    let radius = size / 2.0 - 6.0;
    let p = ui.painter();
    p.circle_stroke(center, radius, Stroke::new(6.0_f32, s.pal().line));
    let frac = frac.clamp(0.0, 1.0);
    if frac > 0.002 {
        let steps = (96.0 * frac).ceil().max(2.0) as usize;
        let pts: Vec<_> = (0..=steps)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * frac * (i as f32 / steps as f32);
                pos2(center.x + radius * a.cos(), center.y + radius * a.sin())
            })
            .collect();
        p.add(Shape::line(pts, Stroke::new(6.0_f32, usage_color(frac, color))));
    }
    p.text(center, Align2::CENTER_CENTER, label, FontId::proportional(size * 0.24), s.pal().fg);
}

/// Line chart for up to two series sharing the same scale.
pub fn spark(ui: &mut Ui, s: &Settings, series: &[(&VecDeque<f32>, Color32)], max: f32, height: f32) {
    let w = ui.available_width().max(40.0);
    let (rect, _) = ui.allocate_exact_size(vec2(w, height), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, Rounding::same((s.rounding * 0.5).min(8.0)), s.pal().panel2);
    for g in 1..4 {
        let y = rect.top() + rect.height() * g as f32 / 4.0;
        p.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], Stroke::new(1.0_f32, s.pal().line.gamma_multiply(0.5)));
    }
    let max = max.max(1e-3);
    let n_max = crate::sampler::HIST;
    for (idx, (data, color)) in series.iter().enumerate() {
        let n = data.len();
        if n < 2 {
            continue;
        }
        let pts: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let x = rect.left() + rect.width() * ((n_max - n + i) as f32 / (n_max - 1) as f32);
                let y = rect.bottom() - 2.0 - (v / max).clamp(0.0, 1.0) * (rect.height() - 4.0);
                pos2(x, y)
            })
            .collect();
        if idx == 0 {
            // One textured-free mesh for the whole area fill. The naive version
            // allocated a Vec<Pos2> per history point on every frame.
            let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 38);
            let mut mesh = egui::Mesh::default();
            for pt in &pts {
                mesh.colored_vertex(*pt, fill);
                mesh.colored_vertex(pos2(pt.x, rect.bottom()), fill);
            }
            for i in 0..pts.len().saturating_sub(1) {
                let a = (i * 2) as u32;
                mesh.add_triangle(a, a + 2, a + 1);
                mesh.add_triangle(a + 1, a + 2, a + 3);
            }
            p.add(Shape::mesh(mesh));
        }
        p.add(Shape::line(pts, Stroke::new(2.0_f32, *color)));
    }
}

pub fn bytes(b: u64) -> String {
    const U: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{} B", b) } else { format!("{:.1} {}", v, U[i]) }
}

pub fn rate(bps: f64) -> String {
    format!("{}/s", bytes(bps.max(0.0) as u64))
}

pub fn duration(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 { format!("{d}d {h}h {m:02}m") } else { format!("{h}h {m:02}m") }
}
