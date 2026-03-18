// UI для оверлея — показываем FPS, позицию и уведомления
//
// Минималистичный интерфейс без лишних элементов

use egui::{Align2, Color32, FontId, Pos2, RichText, Vec2};

use super::state::Snapshot;

// Цвета
const TEXT_DIM: Color32 = Color32::from_rgb(160, 160, 160);
const ACCENT: Color32 = Color32::from_rgb(100, 180, 255);
const BG_OVERLAY: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 180);

// Главная функция — рисуем весь оверлей
pub fn draw_overlay(ctx: &egui::Context, snap: &Snapshot) {
    // Отладочная инфа (F10)
    if snap.show_debug {
        draw_debug(ctx, snap);
    }

    // Уведомление
    if !snap.notification.is_empty() {
        draw_notification(ctx, &snap.notification);
    }
}

// Отладочная информация в левом верхнем углу
fn draw_debug(ctx: &egui::Context, snap: &Snapshot) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    
    let mut y = 10.0;
    let x = 10.0;
    
    // FPS — цвет зависит от значения
    let fps_text = format!("FPS: {:.0}", snap.fps);
    let fps_color = if snap.fps > 50.0 {
        Color32::from_rgb(100, 255, 100)  // зелёный
    } else if snap.fps > 30.0 {
        Color32::from_rgb(255, 200, 100)  // жёлтый
    } else {
        Color32::from_rgb(255, 100, 100)  // красный
    };
    
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        fps_text,
        FontId::monospace(14.0),
        fps_color,
    );
    y += 18.0;
    
    // Позиция игрока
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        format!("X: {:.1}", snap.pos_x),
        FontId::monospace(12.0),
        TEXT_DIM,
    );
    y += 16.0;
    
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        format!("Y: {:.1}", snap.pos_y),
        FontId::monospace(12.0),
        TEXT_DIM,
    );
    y += 16.0;
    
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        format!("Z: {:.1}", snap.pos_z),
        FontId::monospace(12.0),
        TEXT_DIM,
    );
    y += 16.0;
    
    // Состояние игры
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        &snap.game_state,
        FontId::monospace(12.0),
        TEXT_DIM,
    );
}

// Уведомление по центру сверху
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
