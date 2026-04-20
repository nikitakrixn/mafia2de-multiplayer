//! Lua console — dev-инструмент (F4).

use egui::{Align2, RichText, ScrollArea, TextEdit, Vec2};

use crate::overlay::state::{self, ConsoleEntry, ConsoleResult, Snapshot};
use crate::overlay::theme::{self, colors, sizes};

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("lua_console")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            theme::panel_frame()
                .fill(colors::CONSOLE_BG)
                .inner_margin(egui::Margin::ZERO),
        )
        .fixed_size(egui::Vec2::new(sizes::CONSOLE_WIDTH, 0.0))
        .show(ctx, |ui| {
            draw_header(ui);

            egui::Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    draw_history(ui, snap);
                    ui.add_space(6.0);
                    ui.add(egui::Separator::default().spacing(2.0));
                    ui.add_space(6.0);
                    draw_input(ui, snap);
                });
        });
}

fn draw_header(ui: &mut egui::Ui) {
    egui::Frame::NONE
        .fill(colors::BG_TITLE)
        .corner_radius(egui::CornerRadius {
            nw: sizes::WINDOW_ROUNDING,
            ne: sizes::WINDOW_ROUNDING,
            sw: 0,
            se: 0,
        })
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::new(3.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, colors::GOLD);
                ui.add_space(2.0);
                ui.label(
                    RichText::new("LUA CONSOLE")
                        .color(colors::GOLD)
                        .size(13.0)
                        .strong()
                        .extra_letter_spacing(2.5),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("dev tool")
                        .color(colors::TEXT_MUTED)
                        .size(10.0)
                        .italics(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("CLEAR")
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY)
                                    .extra_letter_spacing(1.5),
                            )
                            .fill(colors::BG_WIDGET)
                            .stroke(egui::Stroke::new(1.0, colors::BORDER))
                            .min_size(egui::Vec2::new(56.0, 20.0)),
                        )
                        .clicked()
                    {
                        state::clear_console();
                    }
                });
            });
        });
}

fn draw_history(ui: &mut egui::Ui, snap: &Snapshot) {
    ScrollArea::vertical()
        .id_salt("lua_history")
        .stick_to_bottom(true)
        .max_height(320.0)
        .min_scrolled_height(320.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            if snap.console_entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("История пуста — введите Lua-код ниже")
                            .color(colors::TEXT_MUTED)
                            .size(11.0)
                            .italics(),
                    );
                });
                return;
            }
            for entry in &snap.console_entries {
                draw_entry(ui, entry);
            }
        });
}

fn draw_entry(ui: &mut egui::Ui, entry: &ConsoleEntry) {
    egui::Frame::NONE
        .fill(colors::BG_ROW_ALT)
        .corner_radius(egui::CornerRadius::same(2))
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(
                    RichText::new(&entry.time)
                        .color(colors::CHAT_TIME)
                        .size(10.0)
                        .monospace(),
                );
                ui.label(
                    RichText::new(">")
                        .color(colors::GOLD_DIM)
                        .monospace()
                        .strong(),
                );
                ui.label(
                    RichText::new(&entry.code)
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_PRIMARY),
                );
            });

            match &entry.result {
                ConsoleResult::Queued => result_line(ui, "...", colors::YELLOW),
                ConsoleResult::Ok => result_line(ui, "OK", colors::GREEN),
                ConsoleResult::Error(e) => {
                    result_line(ui, &format!("ERR: {e}"), colors::RED)
                }
            }
        });
}

fn result_line(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.add_space(14.0);
        ui.label(
            RichText::new(text)
                .monospace()
                .size(11.5)
                .color(color),
        );
    });
}

fn draw_input(ui: &mut egui::Ui, snap: &Snapshot) {
    let mut input = snap.console_input.clone();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(">")
                .color(colors::GOLD)
                .monospace()
                .size(14.0)
                .strong(),
        );

        let response = ui.add(
            TextEdit::singleline(&mut input)
                .desired_width(ui.available_width() - 70.0)
                .hint_text("Lua code...")
                .font(egui::FontId::monospace(12.5))
                .margin(egui::Margin::symmetric(8, 5)),
        );

        state::save_console_input(&input);
        response.request_focus();

        let run = ui
            .add_sized(
                [62.0, 26.0],
                egui::Button::new(
                    RichText::new("RUN")
                        .size(11.0)
                        .color(colors::GOLD)
                        .strong()
                        .extra_letter_spacing(1.5),
                )
                .fill(colors::BG_ACTIVE)
                .stroke(egui::Stroke::new(1.0, colors::BORDER_ACTIVE)),
            )
            .clicked();

        let enter = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if (run || enter) && !input.trim().is_empty() {
            let code = input.trim().to_string();
            state::save_console_input("");
            state::submit_console_command(&code);
        }
    });
}
