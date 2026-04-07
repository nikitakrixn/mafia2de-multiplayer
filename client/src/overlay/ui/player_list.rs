//! Список игроков онлайн — без unicode символов.

use egui::{Align2, RichText, ScrollArea, Vec2};

use crate::overlay::state::Snapshot;
use crate::overlay::theme::{colors, sizes};

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("player_list")
        .anchor(Align2::RIGHT_TOP, Vec2::new(-14.0, 30.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .min_width(sizes::PLAYER_LIST_WIDTH)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("ИГРОКИ")
                        .size(12.0)
                        .color(colors::GOLD_DIM)
                        .extra_letter_spacing(2.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}", snap.players.len()))
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY)
                            .monospace(),
                    );
                });
            });

            ui.add(egui::Separator::default().spacing(6.0));

            if snap.players.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Нет игроков")
                            .color(colors::TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.add_space(8.0);
                });
                return;
            }

            ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for player in &snap.players {
                        ui.horizontal(|ui| {
                            // Индикатор: >> для локального, пробел для остальных
                            let (name_text, name_color) = if player.is_local {
                                (format!(">> {}", player.name), colors::GOLD_BRIGHT)
                            } else {
                                (format!("   {}", player.name), colors::TEXT_PRIMARY)
                            };

                            ui.label(
                                RichText::new(name_text)
                                    .color(name_color)
                                    .size(12.0),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let ping_color = match player.ping {
                                        0..=50 => colors::GREEN,
                                        51..=100 => colors::YELLOW,
                                        _ => colors::RED,
                                    };
                                    ui.label(
                                        RichText::new(format!("{}ms", player.ping))
                                            .color(ping_color)
                                            .size(11.0)
                                            .monospace(),
                                    );
                                },
                            );
                        });
                        ui.add_space(1.0);
                    }
                });
        });
}