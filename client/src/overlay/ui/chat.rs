//! Чат — всегда видимые сообщения + открываемое поле ввода.
//!
//! Пассивный режим: последние сообщения с fade (без интерактива).
//! Активный режим (T): полная история + поле ввода.

use egui::{Align2, Color32, FontId, Pos2, RichText, ScrollArea, TextEdit, Vec2};

use crate::overlay::state::{self, ChatMsg, Snapshot};
use crate::overlay::theme::colors;

// Пассивный чат
const PASSIVE_MAX_MSGS: usize = 8;
const PASSIVE_VISIBLE_SECS: f32 = 10.0;
const PASSIVE_FADE_START: f32 = 7.0;
const MSG_LINE_HEIGHT: f32 = 16.0;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    if snap.chat_input_open {
        draw_active(ctx, snap);
    } else {
        draw_passive(ctx, snap);
    }
}

fn draw_passive(ctx: &egui::Context, snap: &Snapshot) {
    // Фильтруем только свежие
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

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("chat_passive"),
    ));

    let screen = ctx.screen_rect();
    let x = 14.0;
    let base_y = screen.max.y - 20.0;

    // Фон
    let total_h = recent.len() as f32 * MSG_LINE_HEIGHT + 12.0;
    let bg_rect = egui::Rect::from_min_size(
        Pos2::new(x - 6.0, base_y - total_h),
        Vec2::new(420.0, total_h + 6.0),
    );
    painter.rect_filled(bg_rect, 4.0, colors::CHAT_BG_PASSIVE);

    for (i, msg) in recent.iter().enumerate() {
        let age = msg.created.elapsed().as_secs_f32();
        let alpha = if age > PASSIVE_FADE_START {
            1.0 - (age - PASSIVE_FADE_START) / (PASSIVE_VISIBLE_SECS - PASSIVE_FADE_START)
        } else {
            1.0
        }
        .clamp(0.0, 1.0);

        let y = base_y - (recent.len() - i) as f32 * MSG_LINE_HEIGHT;
        let text = format_msg(msg);
        let color = msg_color(msg, alpha);

        painter.text(
            Pos2::new(x, y),
            egui::Align2::LEFT_TOP,
            text,
            FontId::proportional(12.0),
            color,
        );
    }
}

fn draw_active(ctx: &egui::Context, snap: &Snapshot) {
    egui::Window::new("chat_active")
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(14.0, -14.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .min_width(420.0)
        .default_height(300.0)
        .show(ctx, |ui| {
            // Заголовок
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("ЧАТ")
                        .size(11.0)
                        .color(colors::GOLD_DIM)
                        .extra_letter_spacing(2.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Enter - отправить, Esc - закрыть")
                            .size(10.0)
                            .color(colors::TEXT_MUTED),
                    );
                });
            });

            ui.add(egui::Separator::default().spacing(4.0));

            // История сообщений
            ScrollArea::vertical()
                .max_height(240.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if snap.chat_msgs.is_empty() {
                        ui.label(
                            RichText::new("Нет сообщений")
                                .color(colors::TEXT_MUTED)
                                .size(12.0),
                        );
                        return;
                    }

                    for msg in &snap.chat_msgs {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(&msg.time)
                                    .color(colors::CHAT_TIME)
                                    .size(10.0)
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

            ui.add(egui::Separator::default().spacing(4.0));

            // Поле ввода
            let mut input = snap.chat_input.clone();

            let response = ui.add(
                TextEdit::singleline(&mut input)
                    .desired_width(ui.available_width())
                    .hint_text("Сообщение...")
                    .font(FontId::proportional(12.0)),
            );

            state::save_chat_input(&input);
            response.request_focus();

            // Enter — отправить
            let enter = response.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if enter {
                let text = input.trim().to_string();
                if !text.is_empty() {
                    state::save_chat_input("");
                    let nick = state::get_nickname();
                    state::add_chat_msg(&nick, &text);
                    crate::network::send_chat_message(text);
                }
                state::close_chat_input();
            }
        });
}

fn format_msg(msg: &ChatMsg) -> String {
    if msg.system {
        format!("{} {}", msg.time, msg.text)
    } else {
        format!("{} {}: {}", msg.time, msg.author, msg.text)
    }
}

fn msg_color(msg: &ChatMsg, alpha: f32) -> Color32 {
    let base = if msg.system {
        colors::CHAT_SYSTEM
    } else {
        colors::TEXT_PRIMARY
    };
    apply_alpha(base, alpha)
}

fn apply_alpha(color: Color32, alpha: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * alpha) as u8)
}