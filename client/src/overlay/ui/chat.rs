//! Чат — пассивная "лента" внизу экрана + полноценное окно при открытии (T).
//!
//! Пассивный режим: последние сообщения с плавным fade всего блока.
//! Активный режим: тайтл-бар, история со скроллом, поле ввода.

use egui::{Align2, FontId, RichText, ScrollArea, TextEdit, Vec2};

use crate::overlay::state::{self, ChatMsg, Snapshot};
use crate::overlay::theme::{self, colors, sizes};

const PASSIVE_MAX_MSGS: usize = 8;
const PASSIVE_VISIBLE_SECS: f32 = 10.0;
const PASSIVE_FADE_START: f32 = 7.0;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    if snap.chat_input_open {
        draw_active(ctx, snap);
    } else {
        draw_passive(ctx, snap);
    }
}

fn draw_passive(ctx: &egui::Context, snap: &Snapshot) {
    let recent: Vec<&ChatMsg> = snap
        .chat_msgs
        .iter()
        .filter(|m| m.created.elapsed().as_secs_f32() < PASSIVE_VISIBLE_SECS)
        .rev()
        .take(PASSIVE_MAX_MSGS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if recent.is_empty() {
        return;
    }

    let block_alpha = recent
        .iter()
        .map(|m| message_alpha(m))
        .fold(0.0_f32, f32::max);

    egui::Area::new(egui::Id::new("chat_passive"))
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(12.0, -12.0))
        .interactable(false)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            theme::overlay_frame(theme::fade(colors::CHAT_BG_PASSIVE, block_alpha))
                .show(ui, |ui| {
                    ui.set_max_width(sizes::CHAT_WIDTH);
                    ui.spacing_mut().item_spacing.y = 1.0;

                    for msg in &recent {
                        draw_passive_msg(ui, msg, block_alpha);
                    }
                });
        });
}

fn draw_passive_msg(ui: &mut egui::Ui, msg: &ChatMsg, block_alpha: f32) {
    let alpha = message_alpha(msg).min(block_alpha);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        ui.label(
            RichText::new(&msg.time)
                .size(10.5)
                .monospace()
                .color(theme::fade(colors::CHAT_TIME, alpha)),
        );

        if msg.system {
            ui.label(
                RichText::new(&msg.text)
                    .size(12.0)
                    .color(theme::fade(colors::CHAT_SYSTEM, alpha)),
            );
        } else {
            ui.label(
                RichText::new(format!("{}:", msg.author))
                    .size(12.0)
                    .strong()
                    .color(theme::fade(colors::CHAT_AUTHOR, alpha)),
            );
            ui.label(
                RichText::new(&msg.text)
                    .size(12.0)
                    .color(theme::fade(colors::TEXT_PRIMARY, alpha)),
            );
        }
    });
}

fn draw_active(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("chat_active")
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(12.0, -12.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(theme::panel_frame().inner_margin(egui::Margin::ZERO))
        .fixed_size(egui::Vec2::new(sizes::CHAT_WIDTH, 0.0))
        .show(ctx, |ui| {
            theme::header_bar(ui, "ЧАТЕРСЫ", Some("ENTER — отправить · ESC — закрыть"));

            egui::Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    draw_active_history(ui, snap);
                    ui.add_space(4.0);
                    ui.add(egui::Separator::default().spacing(2.0));
                    ui.add_space(4.0);
                    draw_active_input(ui, snap);
                });
        });
}

fn draw_active_history(ui: &mut egui::Ui, snap: &Snapshot) {
    ScrollArea::vertical()
        .id_salt("chat_history")
        .max_height(260.0)
        .min_scrolled_height(260.0)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;

            if snap.chat_msgs.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Нет сообщений")
                            .color(colors::TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.add_space(20.0);
                });
                return;
            }

            for msg in &snap.chat_msgs {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label(
                        RichText::new(&msg.time)
                            .color(colors::CHAT_TIME)
                            .size(10.5)
                            .monospace(),
                    );

                    if msg.system {
                        ui.label(
                            RichText::new(&msg.text)
                                .color(colors::CHAT_SYSTEM)
                                .size(12.0),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!("{}:", msg.author))
                                .color(colors::CHAT_AUTHOR)
                                .size(12.0)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(&msg.text)
                                .color(colors::TEXT_PRIMARY)
                                .size(12.0),
                        );
                    }
                });
            }
        });
}

fn draw_active_input(ui: &mut egui::Ui, snap: &Snapshot) {
    let mut input = snap.chat_input.clone();

    let response = ui.add(
        TextEdit::singleline(&mut input)
            .desired_width(ui.available_width())
            .hint_text("Сообщение...")
            .font(FontId::proportional(13.0))
            .margin(egui::Margin::symmetric(8, 6))
            .char_limit(255),
    );

    state::save_chat_input(&input);
    response.request_focus();

    let send = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

    if send {
        let text = input.trim().to_string();
        if !text.is_empty() {
            state::save_chat_input("");
            let nick = state::get_nickname();
            state::add_chat_msg(&nick, &text);
            crate::network::send_chat_message(text);
        }
        state::close_chat_input();
    }
}

fn message_alpha(msg: &ChatMsg) -> f32 {
    let age = msg.created.elapsed().as_secs_f32();
    if age >= PASSIVE_VISIBLE_SECS {
        0.0
    } else if age > PASSIVE_FADE_START {
        1.0 - (age - PASSIVE_FADE_START) / (PASSIVE_VISIBLE_SECS - PASSIVE_FADE_START)
    } else {
        1.0
    }
    .clamp(0.0, 1.0)
}
