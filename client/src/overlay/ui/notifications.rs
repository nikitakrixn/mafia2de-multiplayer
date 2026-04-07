//! Уведомления с fade-out. Дедупликация в state::notify().

use egui::{Align2, Color32, RichText, Vec2};

use crate::overlay::state::Snapshot;
use crate::overlay::theme::{colors, sizes};

const FADE_START_SECS: f32 = 2.5;
const TOTAL_DURATION_SECS: f32 = 4.0;
const NOTIFICATION_SPACING: f32 = 48.0;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    if snap.notifications.is_empty() {
        return;
    }

    for (i, notif) in snap.notifications.iter().enumerate() {
        let age = notif.created.elapsed().as_secs_f32();
        if age >= TOTAL_DURATION_SECS {
            continue;
        }

        let alpha = if age > FADE_START_SECS {
            1.0 - (age - FADE_START_SECS) / (TOTAL_DURATION_SECS - FADE_START_SECS)
        } else {
            1.0
        };

        let offset_y = 40.0 + i as f32 * NOTIFICATION_SPACING;

        egui::Area::new(egui::Id::new("notif").with(i))
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, offset_y))
            .interactable(false)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let bg = apply_alpha(colors::NOTIFY_BG, alpha);
                let text_color = apply_alpha(colors::GOLD_BRIGHT, alpha);

                egui::Frame::NONE
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(20, 10))
                    .corner_radius(egui::CornerRadius::same(sizes::ROUNDING))
                    .stroke(egui::Stroke::new(1.0, apply_alpha(colors::BORDER, alpha)))
                    .show(ui, |ui| {
                        ui.set_min_width(250.0);
                        ui.label(
                            RichText::new(&notif.text)
                                .color(text_color)
                                .size(13.0),
                        );
                    });
            });
    }
}

fn apply_alpha(color: Color32, alpha: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * alpha) as u8)
}