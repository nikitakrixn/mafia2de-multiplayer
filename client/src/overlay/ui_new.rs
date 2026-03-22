//! UI-отрисовка overlay.
//!
//! Здесь рисуются:
//! - debug-информация слева сверху
//!   - FPS
//!   - координаты игрока
//!   - текущее состояние игры
//!
//! - уведомления сверху по центру
//! - мультиплеер UI (меню, чат, список игроков)
//!
//! Рисуем через `egui::Painter` напрямую:
//! без окон, без интерактива, с минимальной нагрузкой на кадр.

use egui::{Align2, Color32, FontId, Pos2, RichText, Vec2};

use super::multiplayer_ui;
use super::state::Snapshot;

// =============================================================================
//  Цветовая схема
// =============================================================================

/// Тусклый текст для второстепенной информации.
const TEXT_DIM: Color32 = Color32::from_rgb(160, 160, 160);

/// Акцентный цвет заголовков.
const ACCENT: Color32 = Color32::from_rgb(100, 180, 255);

/// Фон для уведомления.
const BG_OVERLAY: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 180);

// =============================================================================
//  Главный вход
// =============================================================================

/// Рисует весь overlay за кадр.
///
/// Вызывается из render-thread каждый кадр после сбора snapshot'а.
pub fn draw_overlay(ctx: &egui::Context, snap: &Snapshot) {
    // Обработка горячих клавиш для мультиплеера
    multiplayer_ui::handle_hotkeys();

    // Базовый debug-блок слева сверху
    if snap.show_debug {
        draw_debug(ctx, snap);
    }

    // Временное уведомление сверху по центру
    if !snap.notification.is_empty() {
        draw_notification(ctx, &snap.notification);
    }

    // Мультиплеер UI (меню, чат, список игроков)
    multiplayer_ui::draw_multiplayer_ui(ctx);
}

// =============================================================================
//  Debug-блок
// =============================================================================

/// Рисует компактный debug-блок:
/// - FPS
/// - координаты игрока
/// - состояние игры
fn draw_debug(ctx: &egui::Context, snap: &Snapshot) {
    let painter = ctx.layer_painter(egui::LayerId::background());

    let mut y = 10.0;
    let x = 10.0;

    // Цвет FPS зависит от производительности
    let fps_color = match snap.fps as u32 {
        51.. => Color32::from_rgb(100, 255, 100),    // хороший FPS
        31..=50 => Color32::from_rgb(255, 200, 100), // средний FPS
        _ => Color32::from_rgb(255, 100, 100),       // низкий FPS
    };

    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        format!("FPS: {:.0}", snap.fps),
        FontId::monospace(14.0),
        fps_color,
    );
    y += 18.0;

    // Координаты игрока
    for (label, value) in [("X", snap.pos_x), ("Y", snap.pos_y), ("Z", snap.pos_z)] {
        painter.text(
            Pos2::new(x, y),
            Align2::LEFT_TOP,
            format!("{label}: {value:.1}"),
            FontId::monospace(12.0),
            TEXT_DIM,
        );
        y += 16.0;
    }

    // Состояние игры
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        &snap.game_state,
        FontId::monospace(12.0),
        TEXT_DIM,
    );
}

// =============================================================================
//  Уведомление
// =============================================================================

/// Рисует временное уведомление по центру сверху.
fn draw_notification(ctx: &egui::Context, text: &str) {
    egui::Area::new(egui::Id::new("notification"))
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 50.0))
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(BG_OVERLAY)
                .inner_margin(egui::Margin::symmetric(20, 10))
                .corner_radius(egui::CornerRadius::same(8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(text)
                            .font(FontId::proportional(16.0))
                            .color(ACCENT),
                    );
                });
        });
}
