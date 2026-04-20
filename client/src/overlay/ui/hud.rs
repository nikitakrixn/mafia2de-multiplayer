//! HUD — компактная диагностическая панель (FPS, пинг, координаты, состояние),
//! бейдж подключения и плашка с версией.

use egui::{Align2, RichText, Vec2};

use crate::overlay::state::{ConnectionInfo, Snapshot};
use crate::overlay::theme::{self, colors};

const PANEL_ID: &str = "hud_debug";
const BADGE_ID: &str = "hud_conn_badge";
const VERSION_ID: &str = "hud_version";

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Area::new(egui::Id::new(PANEL_ID))
        .anchor(Align2::LEFT_TOP, Vec2::new(12.0, 12.0))
        .interactable(false)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            theme::overlay_frame(colors::HUD_BG).show(ui, |ui| {
                ui.set_min_width(120.0);
                ui.spacing_mut().item_spacing.y = 2.0;

                let fps_color = match snap.fps as u32 {
                    60.. => colors::GREEN,
                    30..=59 => colors::YELLOW,
                    _ => colors::RED,
                };
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{:>3.0}", snap.fps))
                            .monospace()
                            .size(13.0)
                            .color(fps_color)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("FPS")
                            .size(10.0)
                            .color(colors::TEXT_MUTED)
                            .monospace(),
                    );
                });

                if snap.connection.connected && snap.local_ping > 0 {
                    let ping_color = match snap.local_ping {
                        0..=50 => colors::GREEN,
                        51..=100 => colors::YELLOW,
                        _ => colors::RED,
                    };
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{:>3}", snap.local_ping))
                                .monospace()
                                .size(13.0)
                                .color(ping_color)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("ms")
                                .size(10.0)
                                .color(colors::TEXT_MUTED)
                                .monospace(),
                        );
                    });
                }

                ui.add(egui::Separator::default().spacing(2.0));

                for (label, value) in [
                    ("X", snap.pos[0]),
                    ("Y", snap.pos[1]),
                    ("Z", snap.pos[2]),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(label)
                                .size(10.0)
                                .monospace()
                                .color(colors::TEXT_MUTED),
                        );
                        ui.label(
                            RichText::new(format!("{value:>8.1}"))
                                .size(11.0)
                                .monospace()
                                .color(colors::TEXT_SECONDARY),
                        );
                    });
                }

                ui.add(egui::Separator::default().spacing(2.0));

                ui.label(
                    RichText::new(snap.game_state)
                        .size(10.5)
                        .color(colors::GOLD_DIM)
                        .extra_letter_spacing(1.5),
                );
            });
        });
}

pub fn draw_connection_badge(ctx: &egui::Context, conn: &ConnectionInfo) {
    egui::Area::new(egui::Id::new(BADGE_ID))
        .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-12.0, -12.0))
        .interactable(false)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            theme::overlay_frame(colors::HUD_BG).show(ui, |ui| {
                ui.horizontal(|ui| {
                    theme::status_dot(ui, colors::GREEN, 8.0);
                    ui.label(
                        RichText::new("ONLINE")
                            .size(11.0)
                            .color(colors::GREEN)
                            .strong()
                            .extra_letter_spacing(1.5),
                    );
                    ui.add(egui::Separator::default().vertical().spacing(4.0));
                    ui.label(
                        RichText::new(&conn.status)
                            .size(11.0)
                            .color(colors::TEXT_SECONDARY)
                            .monospace(),
                    );
                });
            });
        });
}

pub fn draw_version(ctx: &egui::Context) {
    egui::Area::new(egui::Id::new(VERSION_ID))
        .anchor(Align2::RIGHT_TOP, Vec2::new(-12.0, 12.0))
        .interactable(false)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            ui.label(
                RichText::new(concat!("M2:DE MP v", env!("CARGO_PKG_VERSION")))
                    .size(10.0)
                    .monospace()
                    .color(colors::TEXT_MUTED),
            );
        });
}
