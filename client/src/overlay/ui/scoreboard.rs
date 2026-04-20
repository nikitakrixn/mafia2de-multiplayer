//! Скорборд (TAB удерживать) — таблица игроков.

use egui::{Align, Align2, Color32, Layout, RichText, Stroke, Vec2};

use crate::overlay::state::{PlayerEntry, Snapshot};
use crate::overlay::theme::{self, colors, sizes};

const COL_MARK: f32 = 14.0;
const COL_ID: f32 = 50.0;
const COL_PING: f32 = 90.0;
const ROW_HEIGHT: f32 = 24.0;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("scoreboard")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            theme::panel_frame()
                .fill(colors::SCOREBOARD_BG)
                .inner_margin(egui::Margin::ZERO),
        )
        .fixed_size(Vec2::new(sizes::SCOREBOARD_WIDTH, 0.0))
        .show(ctx, |ui| {
            draw_header(ui, snap);

            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    draw_table_header(ui);
                    ui.add(egui::Separator::default().spacing(2.0));
                    if snap.players.is_empty() {
                        empty_state(ui);
                    } else {
                        for (i, p) in snap.players.iter().enumerate() {
                            draw_row(ui, p, i);
                        }
                    }
                });

            draw_footer(ui);
        });
}

fn draw_header(ui: &mut egui::Ui, snap: &Snapshot) {
    egui::Frame::NONE
        .fill(colors::BG_TITLE)
        .corner_radius(egui::CornerRadius {
            nw: sizes::WINDOW_ROUNDING,
            ne: sizes::WINDOW_ROUNDING,
            sw: 0,
            se: 0,
        })
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("SCOREBOARD")
                        .size(17.0)
                        .color(colors::GOLD)
                        .extra_letter_spacing(5.0)
                        .strong(),
                );
                let status = if snap.connection.connected {
                    format!(
                        "{} · {} {}",
                        snap.connection.status,
                        snap.players.len(),
                        plural_players(snap.players.len()),
                    )
                } else {
                    "Не подключен".to_string()
                };
                ui.label(
                    RichText::new(status)
                        .size(11.5)
                        .color(colors::TEXT_SECONDARY),
                );
            });
        });
}

fn draw_table_header(ui: &mut egui::Ui) {
    let total_w = ui.available_width();
    let name_w = total_w - COL_MARK - COL_ID - COL_PING - 12.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        // Маркер
        ui.allocate_ui_with_layout(
            Vec2::new(COL_MARK, ROW_HEIGHT),
            Layout::left_to_right(Align::Center),
            |ui| { ui.add_space(0.0); },
        );
        // ID
        ui.allocate_ui_with_layout(
            Vec2::new(COL_ID, ROW_HEIGHT),
            Layout::left_to_right(Align::Center),
            |ui| header_label(ui, "ID"),
        );
        // ИМЯ
        ui.allocate_ui_with_layout(
            Vec2::new(name_w, ROW_HEIGHT),
            Layout::left_to_right(Align::Center),
            |ui| header_label(ui, "ИМЯ"),
        );
        // ПИНГ (правый край)
        ui.allocate_ui_with_layout(
            Vec2::new(COL_PING, ROW_HEIGHT),
            Layout::right_to_left(Align::Center),
            |ui| header_label(ui, "ПИНГ"),
        );
    });
}

fn header_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(10.5)
            .color(colors::TEXT_MUTED)
            .strong()
            .extra_letter_spacing(2.0),
    );
}

fn draw_row(ui: &mut egui::Ui, p: &PlayerEntry, idx: usize) {
    let bg = if p.is_local {
        colors::BG_ROW_LOCAL
    } else if idx % 2 == 1 {
        colors::BG_ROW_ALT
    } else {
        Color32::TRANSPARENT
    };

    let total_w = ui.available_width();
    let name_w = total_w - COL_MARK - COL_ID - COL_PING - 12.0 /* item_spacing */;

    let frame = egui::Frame::NONE
        .fill(bg)
        .stroke(if p.is_local {
            Stroke::new(1.0, colors::BORDER_ACTIVE)
        } else {
            Stroke::NONE
        })
        .corner_radius(egui::CornerRadius::same(2))
        .inner_margin(egui::Margin::symmetric(2, 0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            ui.allocate_ui_with_layout(
                Vec2::new(COL_MARK, ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    if p.is_local {
                        theme::status_dot(ui, colors::GOLD, 7.0);
                    }
                },
            );

            // ID
            ui.allocate_ui_with_layout(
                Vec2::new(COL_ID, ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(format!("#{}", p.id))
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY)
                            .monospace(),
                    );
                },
            );

            ui.allocate_ui_with_layout(
                Vec2::new(name_w, ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    let color = if p.is_local {
                        colors::TEXT_PRIMARY
                    } else {
                        colors::TEXT_SECONDARY
                    };
                    let mut name = RichText::new(&p.name).size(13.0).color(color);
                    if p.is_local {
                        name = name.strong();
                    }
                    ui.label(name);
                    if p.is_local {
                        ui.label(
                            RichText::new("(вы)")
                                .size(11.0)
                                .color(colors::GOLD)
                                .italics(),
                        );
                    }
                },
            );

            ui.allocate_ui_with_layout(
                Vec2::new(COL_PING, ROW_HEIGHT),
                Layout::right_to_left(Align::Center),
                |ui| {
                    let color = ping_color(p.ping);
                    ui.label(
                        RichText::new(format!("{}ms", p.ping))
                            .size(12.0)
                            .color(color)
                            .monospace(),
                    );
                    ui.add_space(4.0);
                    theme::status_dot(ui, color, 5.0);
                },
            );
        });
    });
}

fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.label(
            RichText::new("Нет игроков")
                .color(colors::TEXT_MUTED)
                .size(13.0),
        );
        ui.add_space(20.0);
    });
}

fn draw_footer(ui: &mut egui::Ui) {
    egui::Frame::NONE
        .fill(colors::BG_TITLE)
        .corner_radius(egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: sizes::WINDOW_ROUNDING,
            se: sizes::WINDOW_ROUNDING,
        })
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                theme::hotkey_chip(ui, "TAB");
                ui.label(
                    RichText::new("удерживать для отображения")
                        .size(10.0)
                        .color(colors::TEXT_MUTED),
                );
            });
        });
}

fn ping_color(ping: u32) -> Color32 {
    match ping {
        0..=50 => colors::GREEN,
        51..=100 => colors::YELLOW,
        _ => colors::RED,
    }
}

fn plural_players(n: usize) -> &'static str {
    let n10 = n % 10;
    let n100 = n % 100;
    if (11..=14).contains(&n100) {
        "игроков"
    } else if n10 == 1 {
        "игрок"
    } else if (2..=4).contains(&n10) {
        "игрока"
    } else {
        "игроков"
    }
}
