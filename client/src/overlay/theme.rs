//! Тема в стилистике Mafia II.

use egui::{Color32, CornerRadius, FontId, Shadow, Stroke, Style, TextStyle};

#[allow(dead_code)]
pub mod colors {
    use egui::Color32;

    pub const BG_DARK: Color32 = Color32::from_rgba_premultiplied(12, 12, 16, 230);
    pub const BG_PANEL: Color32 = Color32::from_rgba_premultiplied(20, 20, 28, 220);
    pub const BG_WIDGET: Color32 = Color32::from_rgba_premultiplied(35, 35, 45, 200);
    pub const BG_HOVER: Color32 = Color32::from_rgba_premultiplied(50, 50, 65, 220);
    pub const BG_ACTIVE: Color32 = Color32::from_rgba_premultiplied(60, 55, 40, 220);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 215, 200);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(140, 135, 125);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(90, 85, 80);

    pub const GOLD: Color32 = Color32::from_rgb(200, 170, 100);
    pub const GOLD_BRIGHT: Color32 = Color32::from_rgb(230, 195, 110);
    pub const GOLD_DIM: Color32 = Color32::from_rgb(150, 125, 70);

    pub const GREEN: Color32 = Color32::from_rgb(90, 180, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(200, 180, 80);
    pub const RED: Color32 = Color32::from_rgb(200, 80, 70);
    pub const BLUE: Color32 = Color32::from_rgb(80, 140, 200);

    pub const CHAT_AUTHOR: Color32 = Color32::from_rgb(180, 160, 110);
    pub const CHAT_SYSTEM: Color32 = Color32::from_rgb(200, 180, 80);
    pub const CHAT_TIME: Color32 = Color32::from_rgb(80, 75, 70);

    pub const BORDER: Color32 = Color32::from_rgba_premultiplied(80, 70, 50, 120);
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgba_premultiplied(200, 170, 100, 150);
    pub const NOTIFY_BG: Color32 = Color32::from_rgba_premultiplied(15, 15, 20, 220);
    pub const CHAT_BG_PASSIVE: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 90);
    pub const CHAT_BG_ACTIVE: Color32 = Color32::from_rgba_premultiplied(15, 15, 20, 210);
    pub const SCOREBOARD_BG: Color32 = Color32::from_rgba_premultiplied(10, 10, 14, 220);
    pub const CONSOLE_BG: Color32 = Color32::from_rgba_premultiplied(8, 8, 12, 235);
}

#[allow(dead_code)]
pub mod sizes {
    pub const ROUNDING: u8 = 4;
    pub const WINDOW_ROUNDING: u8 = 6;
    pub const BORDER_WIDTH: f32 = 1.0;
    pub const ITEM_SPACING: f32 = 4.0;
    pub const BUTTON_PADDING_X: f32 = 16.0;
    pub const BUTTON_PADDING_Y: f32 = 6.0;
    pub const CHAT_WIDTH: f32 = 420.0;
    pub const CHAT_HEIGHT: f32 = 320.0;
    pub const PLAYER_LIST_WIDTH: f32 = 260.0;
    pub const CONNECT_WIDTH: f32 = 380.0;
    pub const SCOREBOARD_WIDTH: f32 = 500.0;
    pub const CONSOLE_WIDTH: f32 = 600.0;
    pub const CONSOLE_HEIGHT: f32 = 400.0;
}

pub fn apply(ctx: &egui::Context) {
    apply_style(ctx);
}

//  Шрифты но пока не хочу трогать ибо ложиться игра
// fn apply_fonts(ctx: &egui::Context) {
//     use egui::{FontData, FontDefinitions, FontFamily};
//     let mut fonts = FontDefinitions::default();
//     fonts.font_data.insert("inter".into(), std::sync::Arc::new(
//         FontData::from_static(include_bytes!("../../assets/fonts/Inter-Medium.ttf"))
//     ));
//     fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "inter".into());
//     fonts.font_data.insert("jetbrains".into(), std::sync::Arc::new(
//         FontData::from_static(include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"))
//     ));
//     fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "jetbrains".into());
//     ctx.set_fonts(fonts);
// }

fn apply_style(ctx: &egui::Context) {
    let mut style = Style::default();

    style.text_styles = [
        (TextStyle::Small, FontId::proportional(11.0)),
        (TextStyle::Body, FontId::proportional(13.0)),
        (TextStyle::Button, FontId::proportional(13.0)),
        (TextStyle::Heading, FontId::proportional(18.0)),
        (TextStyle::Monospace, FontId::monospace(12.0)),
    ]
    .into();

    style.spacing.item_spacing = egui::Vec2::splat(sizes::ITEM_SPACING);
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.button_padding = egui::Vec2::new(sizes::BUTTON_PADDING_X, sizes::BUTTON_PADDING_Y);

    let v = &mut style.visuals;
    v.dark_mode = true;
    v.window_fill = colors::BG_PANEL;
    v.window_stroke = Stroke::new(sizes::BORDER_WIDTH, colors::BORDER);
    v.window_shadow = Shadow { offset: [0, 4], blur: 12, spread: 0, color: Color32::from_black_alpha(100) };
    v.window_corner_radius = CornerRadius::same(sizes::WINDOW_ROUNDING);
    v.panel_fill = colors::BG_PANEL;
    v.popup_shadow = Shadow { offset: [0, 2], blur: 8, spread: 0, color: Color32::from_black_alpha(80) };

    v.widgets.noninteractive.bg_fill = colors::BG_WIDGET;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_SECONDARY);
    v.widgets.noninteractive.corner_radius = CornerRadius::same(sizes::ROUNDING);

    v.widgets.inactive.bg_fill = colors::BG_WIDGET;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);
    v.widgets.inactive.corner_radius = CornerRadius::same(sizes::ROUNDING);
    v.widgets.inactive.bg_stroke = Stroke::new(sizes::BORDER_WIDTH, colors::BORDER);

    v.widgets.hovered.bg_fill = colors::BG_HOVER;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::GOLD_BRIGHT);
    v.widgets.hovered.corner_radius = CornerRadius::same(sizes::ROUNDING);
    v.widgets.hovered.bg_stroke = Stroke::new(sizes::BORDER_WIDTH, colors::GOLD_DIM);

    v.widgets.active.bg_fill = colors::BG_ACTIVE;
    v.widgets.active.fg_stroke = Stroke::new(1.0, colors::GOLD_BRIGHT);
    v.widgets.active.corner_radius = CornerRadius::same(sizes::ROUNDING);
    v.widgets.active.bg_stroke = Stroke::new(sizes::BORDER_WIDTH, colors::GOLD);

    v.widgets.open.bg_fill = colors::BG_HOVER;
    v.widgets.open.fg_stroke = Stroke::new(1.0, colors::GOLD);
    v.widgets.open.corner_radius = CornerRadius::same(sizes::ROUNDING);

    v.override_text_color = Some(colors::TEXT_PRIMARY);
    v.selection.bg_fill = Color32::from_rgba_premultiplied(200, 170, 100, 60);
    v.selection.stroke = Stroke::new(1.0, colors::GOLD);
    v.extreme_bg_color = colors::BG_DARK;
    v.faint_bg_color = Color32::from_rgba_premultiplied(30, 30, 40, 100);
    v.striped = true;

    ctx.set_style(style);
}