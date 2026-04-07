//! HUD — FPS, координаты, бейдж подключения, версия.

use egui::{Align2, FontId, Pos2};

use crate::overlay::state::{ConnectionInfo, Snapshot};
use crate::overlay::theme::colors;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    let painter = ctx.layer_painter(egui::LayerId::background());

    let x = 14.0;
    let mut y = 14.0;

    let fps_color = match snap.fps as u32 {
        60.. => colors::GREEN,
        30..=59 => colors::YELLOW,
        _ => colors::RED,
    };

    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        format!("{:.0} FPS", snap.fps),
        FontId::monospace(13.0),
        fps_color,
    );
    y += 18.0;

    let labels = ["X", "Y", "Z"];
    for (i, label) in labels.iter().enumerate() {
        painter.text(
            Pos2::new(x, y),
            Align2::LEFT_TOP,
            format!("{label}: {:.1}", snap.pos[i]),
            FontId::monospace(11.0),
            colors::TEXT_MUTED,
        );
        y += 14.0;
    }

    y += 2.0;
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        snap.game_state,
        FontId::monospace(11.0),
        colors::TEXT_MUTED,
    );
}

/// Бейдж — рисуем текст, потом кружок слева от него.
pub fn draw_connection_badge(ctx: &egui::Context, conn: &ConnectionInfo) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen = ctx.input(|i| i.screen_rect());

    let x = screen.max.x - 14.0;
    let y = screen.max.y - 20.0;

    // Сначала статус текст (правый край)
    let status_rect = painter.text(
        Pos2::new(x, y),
        Align2::RIGHT_TOP,
        &conn.status,
        FontId::proportional(11.0),
        colors::TEXT_SECONDARY,
    );

    // "ONLINE" левее статуса
    let online_rect = painter.text(
        Pos2::new(status_rect.min.x - 8.0, y),
        Align2::RIGHT_TOP,
        "ONLINE",
        FontId::proportional(11.0),
        colors::GREEN,
    );

    // Кружок левее "ONLINE"
    painter.circle_filled(
        Pos2::new(online_rect.min.x - 8.0, online_rect.center().y),
        4.0,
        colors::GREEN,
    );
}

pub fn draw_version(ctx: &egui::Context) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen = ctx.input(|i| i.screen_rect());

    painter.text(
        Pos2::new(screen.max.x - 8.0, 8.0),
        Align2::RIGHT_TOP,
        "M2:MP v0.1.0",
        FontId::monospace(10.0),
        colors::TEXT_MUTED,
    );
}