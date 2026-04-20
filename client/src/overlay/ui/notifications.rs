//! Уведомления-тосты сверху по центру с fade-out и slide-in.
//! Дедупликация одинаковых сообщений происходит в `state::notify()`.

use egui::{Align2, RichText, Vec2};

use crate::overlay::state::Snapshot;
use crate::overlay::theme::{self, colors, sizes};

const FADE_START_SECS: f32 = 2.5;
const TOTAL_DURATION_SECS: f32 = 4.0;
const NOTIFICATION_SPACING: f32 = 44.0;
const SLIDE_IN_SECS: f32 = 0.18;

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

        let slide = (age / SLIDE_IN_SECS).clamp(0.0, 1.0);
        let slide_offset = (1.0 - slide) * -10.0;

        let offset_y = 28.0 + i as f32 * NOTIFICATION_SPACING + slide_offset;

        egui::Area::new(egui::Id::new("notif").with(i))
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, offset_y))
            .interactable(false)
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(theme::fade(colors::NOTIFY_BG, alpha))
                    .stroke(egui::Stroke::new(
                        1.0,
                        theme::fade(colors::BORDER_STRONG, alpha),
                    ))
                    .corner_radius(egui::CornerRadius::same(sizes::ROUNDING))
                    .inner_margin(egui::Margin::symmetric(20, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(260.0);
                        ui.horizontal(|ui| {
                            theme::status_dot(ui, theme::fade(colors::GOLD, alpha), 6.0);
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(&notif.text)
                                    .color(theme::fade(colors::GOLD_BRIGHT, alpha))
                                    .size(13.0),
                            );
                        });
                    });

                ctx.request_repaint();
            });
    }
}
