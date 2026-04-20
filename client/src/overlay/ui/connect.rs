//! Окно подключения к серверу (F2).

use egui::{Align2, RichText, TextEdit, Vec2};

use crate::overlay::state::{self, Snapshot};
use crate::overlay::theme::{self, colors, sizes};

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("connect")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(theme::panel_frame().inner_margin(egui::Margin::ZERO))
        .fixed_size(egui::Vec2::new(sizes::CONNECT_WIDTH, 0.0))
        .show(ctx, |ui| {
            draw_brand(ui);
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(20, 14))
                .show(ui, |ui| {
                    draw_form(ui, snap);
                    ui.add_space(10.0);
                    draw_status(ui, snap);
                    ui.add_space(10.0);
                    draw_actions(ui, snap);
                    ui.add_space(8.0);
                    draw_hotkeys(ui);
                });
        });
}

fn draw_brand(ui: &mut egui::Ui) {
    egui::Frame::NONE
        .fill(colors::BG_TITLE)
        .corner_radius(egui::CornerRadius {
            nw: sizes::WINDOW_ROUNDING,
            ne: sizes::WINDOW_ROUNDING,
            sw: 0,
            se: 0,
        })
        .inner_margin(egui::Margin::symmetric(20, 14))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("MAFIA II")
                        .size(24.0)
                        .color(colors::GOLD)
                        .strong()
                        .extra_letter_spacing(2.0),
                );
                ui.label(
                    RichText::new("MULTIPLAYER")
                        .size(11.0)
                        .color(colors::GOLD_DIM)
                        .extra_letter_spacing(5.0),
                );
            });
        });
}

fn draw_form(ui: &mut egui::Ui, snap: &Snapshot) {
    let mut conn = snap.connection.clone();

    egui::Grid::new("connect_fields")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .min_col_width(60.0)
        .show(ui, |ui| {
            field_label(ui, "Адрес");
            ui.add(
                TextEdit::singleline(&mut conn.ip)
                    .desired_width(ui.available_width())
                    .hint_text("127.0.0.1")
                    .margin(egui::Margin::symmetric(8, 5)),
            );
            ui.end_row();

            field_label(ui, "Порт");
            ui.add(
                TextEdit::singleline(&mut conn.port)
                    .desired_width(ui.available_width())
                    .hint_text("7788")
                    .margin(egui::Margin::symmetric(8, 5)),
            );
            ui.end_row();

            field_label(ui, "Ник");
            ui.add(
                TextEdit::singleline(&mut conn.nickname)
                    .desired_width(ui.available_width())
                    .hint_text("Player")
                    .char_limit(24)
                    .margin(egui::Margin::symmetric(8, 5)),
            );
            ui.end_row();
        });

    if let Ok(mut c) = state::CONNECTION.lock() {
        c.ip.clone_from(&conn.ip);
        c.port.clone_from(&conn.port);
        c.nickname.clone_from(&conn.nickname);
    }
}

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(
            RichText::new(text)
                .color(colors::TEXT_SECONDARY)
                .size(12.0),
        );
    });
}

fn draw_status(ui: &mut egui::Ui, snap: &Snapshot) {
    let conn = &snap.connection;
    let (label, color) = if conn.connected {
        ("ONLINE", colors::GREEN)
    } else if conn.status.contains("одключение") {
        ("CONNECTING", colors::YELLOW)
    } else {
        ("OFFLINE", colors::RED)
    };

    egui::Frame::NONE
        .fill(colors::BG_WIDGET)
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .corner_radius(egui::CornerRadius::same(sizes::ROUNDING))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                theme::status_dot(ui, color, 8.0);
                ui.label(
                    RichText::new(label)
                        .color(color)
                        .size(11.0)
                        .strong()
                        .extra_letter_spacing(1.5),
                );
                ui.add(egui::Separator::default().vertical().spacing(4.0));
                ui.label(
                    RichText::new(&conn.status)
                        .color(colors::TEXT_SECONDARY)
                        .size(11.0),
                );
            });
        });
}

fn draw_actions(ui: &mut egui::Ui, snap: &Snapshot) {
    let conn = &snap.connection;

    ui.horizontal(|ui| {
        let primary_label;
        let primary_color;
        let primary_active: bool;
        if conn.connected {
            primary_label = "ОТКЛЮЧИТЬСЯ";
            primary_color = colors::RED;
            primary_active = true;
        } else {
            primary_label = "ПОДКЛЮЧИТЬСЯ";
            primary_color = colors::GOLD;
            primary_active = true;
        }

        let primary = ui.add_sized(
            [ui.available_width() - 96.0, 32.0],
            egui::Button::new(
                RichText::new(primary_label)
                    .size(12.5)
                    .color(primary_color)
                    .strong()
                    .extra_letter_spacing(1.5),
            )
            .fill(colors::BG_WIDGET)
            .stroke(egui::Stroke::new(1.0, colors::BORDER_STRONG)),
        );

        if primary_active && primary.clicked() {
            if conn.connected {
                crate::network::disconnect();
                state::close_connect();
            } else {
                let port: u16 = conn.port.parse().unwrap_or(protocol::DEFAULT_PORT);
                crate::network::connect(&conn.ip, port, &conn.nickname);
                state::close_connect();
            }
        }

        let close = ui.add_sized(
            [88.0, 32.0],
            egui::Button::new(
                RichText::new("ЗАКРЫТЬ")
                    .size(12.0)
                    .color(colors::TEXT_SECONDARY),
            )
            .fill(colors::BG_WIDGET)
            .stroke(egui::Stroke::new(1.0, colors::BORDER)),
        );
        if close.clicked() {
            state::close_connect();
        }
    });
}

fn draw_hotkeys(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (key, label) in [
            ("F2", "меню"),
            ("F3", "игроки"),
            ("F4", "консоль"),
            ("T", "чат"),
            ("TAB", "скорборд"),
        ] {
            theme::hotkey_chip(ui, key);
            ui.label(
                RichText::new(label)
                    .size(10.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.add_space(6.0);
        }
    });
}
