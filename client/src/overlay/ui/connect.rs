//! Окно подключения к серверу.

use egui::{Align2, RichText, TextEdit, Vec2};

use crate::overlay::state::{self, Snapshot};
use crate::overlay::theme::{colors, sizes};

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("MAFIA II ONLINE")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .min_width(sizes::CONNECT_WIDTH)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("MAFIA II").size(22.0).color(colors::GOLD).strong());
                ui.label(
                    RichText::new("MULTIPLAYER")
                        .size(11.0)
                        .color(colors::GOLD_DIM)
                        .extra_letter_spacing(4.0),
                );
                ui.add_space(12.0);
            });

            ui.add(egui::Separator::default().spacing(8.0));

            let mut conn = snap.connection.clone();

            egui::Grid::new("connect_fields")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Адрес").color(colors::TEXT_SECONDARY).size(12.0));
                    ui.add(
                        TextEdit::singleline(&mut conn.ip)
                            .desired_width(220.0)
                            .hint_text("127.0.0.1"),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Порт").color(colors::TEXT_SECONDARY).size(12.0));
                    ui.add(
                        TextEdit::singleline(&mut conn.port)
                            .desired_width(220.0)
                            .hint_text("7788"),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Ник").color(colors::TEXT_SECONDARY).size(12.0));
                    ui.add(
                        TextEdit::singleline(&mut conn.nickname)
                            .desired_width(220.0)
                            .hint_text("Player")
                            .char_limit(24),
                    );
                    ui.end_row();
                });

            if let Ok(mut c) = state::CONNECTION.lock() {
                c.ip.clone_from(&conn.ip);
                c.port.clone_from(&conn.port);
                c.nickname.clone_from(&conn.nickname);
            }

            ui.add_space(8.0);

            let (status_label, status_color) = if conn.connected {
                ("ONLINE", colors::GREEN)
            } else if conn.status.contains("одключение") {
                ("CONNECTING", colors::YELLOW)
            } else {
                ("OFFLINE", colors::RED)
            };

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, status_color);
                ui.label(RichText::new(status_label).color(status_color).size(11.0).strong());
                ui.label(RichText::new(&conn.status).color(colors::TEXT_SECONDARY).size(11.0));
            });

            ui.add_space(8.0);
            ui.add(egui::Separator::default().spacing(8.0));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if !conn.connected {
                    let btn = ui.add_sized(
                        [120.0, 30.0],
                        egui::Button::new(RichText::new("ПОДКЛЮЧИТЬСЯ").size(12.0).color(colors::GOLD)),
                    );
                    if btn.clicked() {
                        let port: u16 = conn.port.parse().unwrap_or(protocol::DEFAULT_PORT);
                        crate::network::connect(&conn.ip, port, &conn.nickname);
                        state::close_connect();
                    }
                } else {
                    let btn = ui.add_sized(
                        [120.0, 30.0],
                        egui::Button::new(RichText::new("ОТКЛЮЧИТЬСЯ").size(12.0).color(colors::RED)),
                    );
                    if btn.clicked() {
                        crate::network::disconnect();
                        state::close_connect();
                    }
                }

                let close = ui.add_sized(
                    [80.0, 30.0],
                    egui::Button::new(RichText::new("ЗАКРЫТЬ").size(12.0).color(colors::TEXT_SECONDARY)),
                );
                if close.clicked() {
                    state::close_connect();
                }
            });

            ui.add_space(6.0);
            ui.label(
                RichText::new("F2 меню  |  F3 игроки  |  T чат  |  TAB скорборд")
                    .size(10.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.add_space(4.0);
        });
}
