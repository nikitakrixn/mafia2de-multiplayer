//! Список игроков онлайн (F3).

use egui::{Align2, RichText, ScrollArea, Vec2};

use crate::overlay::state::{PlayerEntry, Snapshot};
use crate::overlay::theme::{self, colors, sizes};

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("player_list")
        .anchor(Align2::RIGHT_TOP, Vec2::new(-12.0, 40.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(theme::panel_frame().inner_margin(egui::Margin::ZERO))
        .fixed_size(egui::Vec2::new(sizes::PLAYER_LIST_WIDTH, 0.0))
        .show(ctx, |ui| {
            let count = snap.players.len();
            theme::header_bar(ui, "ИГРОКИ", Some(&format!("{count} онлайн")));

            egui::Frame::NONE
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    if snap.players.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("Нет игроков")
                                    .color(colors::TEXT_MUTED)
                                    .size(12.0),
                            );
                            ui.add_space(12.0);
                        });
                        return;
                    }

                    ScrollArea::vertical()
                        .id_salt("player_list_scroll")
                        .max_height(420.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (i, p) in snap.players.iter().enumerate() {
                                draw_row(ui, p, i);
                            }
                        });
                });
        });
}

fn draw_row(ui: &mut egui::Ui, p: &PlayerEntry, idx: usize) {
    let (bg, stroke) = if p.is_local {
        (
            colors::BG_ROW_LOCAL,
            egui::Stroke::new(1.0, colors::BORDER_ACTIVE),
        )
    } else if idx % 2 == 1 {
        (colors::BG_ROW_ALT, egui::Stroke::NONE)
    } else {
        (egui::Color32::TRANSPARENT, egui::Stroke::NONE)
    };

    egui::Frame::NONE
        .fill(bg)
        .stroke(stroke)
        .inner_margin(egui::Margin::symmetric(6, 3))
        .corner_radius(egui::CornerRadius::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                theme::status_dot(
                    ui,
                    if p.is_local { colors::GOLD } else { colors::TEXT_MUTED },
                    6.0,
                );
                ui.add_space(2.0);

                let name_color = if p.is_local {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
                };
                let mut name = RichText::new(&p.name).color(name_color).size(12.5);
                if p.is_local {
                    name = name.strong();
                }
                ui.label(name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let pc = ping_color(p.ping);
                    ui.label(
                        RichText::new(format!("{}ms", p.ping))
                            .color(pc)
                            .size(11.0)
                            .monospace(),
                    );
                    ui.add_space(4.0);
                    theme::status_dot(ui, pc, 5.0);
                });
            });
        });
}

fn ping_color(ping: u32) -> egui::Color32 {
    match ping {
        0..=50 => colors::GREEN,
        51..=100 => colors::YELLOW,
        _ => colors::RED,
    }
}
