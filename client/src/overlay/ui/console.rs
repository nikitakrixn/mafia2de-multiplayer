//! Lua console — нормальный dev-инструмент (F4)

use egui::{Align2, RichText, ScrollArea, TextEdit, Vec2};

use crate::overlay::state::{self, ConsoleResult, Snapshot};
use crate::overlay::theme::colors;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("lua_console")
        .fixed_size(Vec2::new(600.0, 400.0))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::NONE
                .fill(colors::CONSOLE_BG)
                .inner_margin(egui::Margin::same(10))
                .corner_radius(egui::CornerRadius::same(6))
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("LUA CONSOLE")
                        .color(colors::GOLD)
                        .size(14.0)
                        .strong(),
                );

                ui.add_space(10.0);

                ui.label(
                    RichText::new("dev tool")
                        .color(colors::TEXT_MUTED)
                        .size(10.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("CLEAR")
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY),
                            )
                            .fill(colors::BG_WIDGET),
                        )
                        .clicked()
                    {
                        state::clear_console();
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();

            ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(300.0)
                .show(ui, |ui| {
                    for entry in &snap.console_entries {
                        draw_entry(ui, entry);
                        ui.add_space(2.0);
                    }
                });

            ui.separator();

            let mut input = snap.console_input.clone();

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(">")
                        .color(colors::GOLD)
                        .monospace(),
                );

                let response = ui.add(
                    TextEdit::singleline(&mut input)
                        .desired_width(460.0)
                        .hint_text("Lua code...")
                        .font(egui::FontId::monospace(12.0)),
                );

                state::save_console_input(&input);
                response.request_focus();

                let run = ui
                    .add(
                        egui::Button::new(
                            RichText::new("RUN")
                                .size(11.0)
                                .color(colors::GOLD),
                        )
                        .fill(colors::BG_ACTIVE),
                    )
                    .clicked();

                let enter =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if (run || enter) && !input.trim().is_empty() {
                    let code = input.trim().to_string();
                    state::save_console_input("");
                    state::submit_console_command(&code);
                }
            });
        });
}

fn draw_entry(ui: &mut egui::Ui, entry: &state::ConsoleEntry) {
    ui.vertical(|ui| {
        // Команда
        ui.label(
            RichText::new(format!("> {}", entry.code))
                .monospace()
                .color(colors::TEXT_PRIMARY),
        );

        // Результат
        match &entry.result {
            ConsoleResult::Queued => {
                ui.label(
                    RichText::new("  ...")
                        .monospace()
                        .color(colors::YELLOW),
                );
            }
            ConsoleResult::Ok => {
                ui.label(
                    RichText::new("  OK")
                        .monospace()
                        .color(colors::GREEN),
                );
            }
            ConsoleResult::Error(e) => {
                ui.label(
                    RichText::new(format!("  ERR: {}", e))
                        .monospace()
                        .color(colors::RED),
                );
            }
        }
    });
}