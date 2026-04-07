//! Скорборд — TAB удерживать.

use egui::{Align2, RichText, Vec2};

use crate::overlay::state::Snapshot;
use crate::overlay::theme::colors;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("scoreboard")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .min_width(480.0)
        .show(ctx, |ui| {
            // Заголовок
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("SCOREBOARD")
                        .size(16.0)
                        .color(colors::GOLD)
                        .extra_letter_spacing(3.0)
                        .strong(),
                );
            });

            ui.add_space(4.0);

            // Статус сервера
            ui.vertical_centered(|ui| {
                let status = if snap.connection.connected {
                    format!("{} | {} игроков", snap.connection.status, snap.players.len())
                } else {
                    "Не подключен".to_string()
                };
                ui.label(
                    RichText::new(status)
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
            });

            ui.add_space(6.0);
            ui.add(egui::Separator::default().spacing(4.0));

            if snap.players.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("Нет игроков")
                            .color(colors::TEXT_MUTED)
                            .size(13.0),
                    );
                    ui.add_space(16.0);
                });
                return;
            }

            // Заголовок таблицы
            ui.add_space(4.0);
            egui::Grid::new("scoreboard_header")
                .num_columns(3)
                .min_col_width(60.0)
                .spacing([20.0, 2.0])
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("ID")
                            .size(10.0)
                            .color(colors::TEXT_MUTED)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("ИМЯ")
                            .size(10.0)
                            .color(colors::TEXT_MUTED)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("ПИНГ")
                            .size(10.0)
                            .color(colors::TEXT_MUTED)
                            .strong(),
                    );
                    ui.end_row();
                });

            ui.add(egui::Separator::default().spacing(2.0));

            // Строки игроков
            egui::Grid::new("scoreboard_rows")
                .num_columns(3)
                .min_col_width(60.0)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    for player in &snap.players {
                        let name_color = if player.is_local {
                            colors::GOLD_BRIGHT
                        } else {
                            colors::TEXT_PRIMARY
                        };

                        let ping_color = match player.ping {
                            0..=50 => colors::GREEN,
                            51..=100 => colors::YELLOW,
                            _ => colors::RED,
                        };

                        ui.label(
                            RichText::new(format!("#{}", player.id))
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY)
                                .monospace(),
                        );

                        let name_display = if player.is_local {
                            format!("{} (вы)", player.name)
                        } else {
                            player.name.clone()
                        };
                        ui.label(
                            RichText::new(name_display)
                                .size(12.0)
                                .color(name_color),
                        );

                        ui.label(
                            RichText::new(format!("{}ms", player.ping))
                                .size(12.0)
                                .color(ping_color)
                                .monospace(),
                        );
                        ui.end_row();
                    }
                });

            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("[TAB]")
                        .size(10.0)
                        .color(colors::TEXT_MUTED),
                );
            });
        });
}