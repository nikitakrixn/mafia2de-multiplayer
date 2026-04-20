//! Отрисовка собственного курсора поверх игры.

use egui::{Color32, Pos2, Stroke, Vec2};

use crate::overlay::theme::colors;

pub fn draw(ctx: &egui::Context) {
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("m2mp_cursor"),
    ));

    let scale = 14.0;
    let tip = pos;
    let bottom_left = Pos2::new(pos.x + scale * 0.05, pos.y + scale);
    let mid = Pos2::new(pos.x + scale * 0.45, pos.y + scale * 0.78);
    let right = Pos2::new(pos.x + scale * 0.78, pos.y + scale * 0.45);

    let shadow_offset = Vec2::new(1.0, 1.5);
    let shadow_pts = vec![
        tip + shadow_offset,
        right + shadow_offset,
        mid + shadow_offset,
        bottom_left + shadow_offset,
    ];
    painter.add(egui::Shape::convex_polygon(
        shadow_pts,
        Color32::from_black_alpha(150),
        Stroke::NONE,
    ));

    let fill_pts = vec![tip, right, mid, bottom_left];
    painter.add(egui::Shape::convex_polygon(
        fill_pts,
        colors::TEXT_PRIMARY,
        Stroke::new(1.2, Color32::from_black_alpha(220)),
    ));
}
