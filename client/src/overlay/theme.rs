//! Тема в стилистике Mafia II.

use egui::{Color32, CornerRadius, FontId, RichText, Shadow, Stroke, Style, TextStyle};

#[allow(dead_code)]
pub mod colors {
    use egui::Color32;

    pub const BG_DARK: Color32 = Color32::from_rgba_premultiplied(12, 12, 16, 230);
    pub const BG_PANEL: Color32 = Color32::from_rgba_premultiplied(20, 20, 28, 230);
    pub const BG_PANEL_SOLID: Color32 = Color32::from_rgba_premultiplied(18, 18, 24, 245);
    pub const BG_WIDGET: Color32 = Color32::from_rgba_premultiplied(35, 35, 45, 220);
    pub const BG_HOVER: Color32 = Color32::from_rgba_premultiplied(50, 50, 65, 230);
    pub const BG_ACTIVE: Color32 = Color32::from_rgba_premultiplied(60, 55, 40, 230);
    pub const BG_TITLE: Color32 = Color32::from_rgba_premultiplied(28, 24, 16, 235);
    pub const BG_ROW_ALT: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 6);
    /// Тёмно-янтарная подложка строки локального игрока.
    /// Намеренно тёмная (не золотая): текст поверх неё будет белым/светлым,
    /// иначе золото-на-золоте теряет контраст.
    pub const BG_ROW_LOCAL: Color32 = Color32::from_rgba_premultiplied(60, 45, 10, 200);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(225, 220, 205);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(150, 145, 135);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(95, 90, 85);

    pub const GOLD: Color32 = Color32::from_rgb(200, 170, 100);
    pub const GOLD_BRIGHT: Color32 = Color32::from_rgb(232, 198, 112);
    pub const GOLD_DIM: Color32 = Color32::from_rgb(150, 125, 70);

    pub const GREEN: Color32 = Color32::from_rgb(110, 190, 100);
    pub const YELLOW: Color32 = Color32::from_rgb(220, 190, 80);
    pub const RED: Color32 = Color32::from_rgb(210, 85, 75);
    pub const BLUE: Color32 = Color32::from_rgb(90, 150, 210);

    pub const CHAT_AUTHOR: Color32 = Color32::from_rgb(195, 170, 115);
    pub const CHAT_SYSTEM: Color32 = Color32::from_rgb(210, 185, 90);
    pub const CHAT_TIME: Color32 = Color32::from_rgb(95, 90, 80);

    pub const BORDER: Color32 = Color32::from_rgba_premultiplied(80, 70, 50, 140);
    pub const BORDER_STRONG: Color32 = Color32::from_rgba_premultiplied(120, 100, 60, 180);
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgba_premultiplied(200, 170, 100, 170);
    pub const NOTIFY_BG: Color32 = Color32::from_rgba_premultiplied(15, 15, 20, 230);
    pub const CHAT_BG_PASSIVE: Color32 = Color32::from_rgba_premultiplied(8, 8, 12, 130);
    pub const CHAT_BG_ACTIVE: Color32 = Color32::from_rgba_premultiplied(15, 15, 20, 235);
    pub const SCOREBOARD_BG: Color32 = Color32::from_rgba_premultiplied(10, 10, 14, 235);
    pub const CONSOLE_BG: Color32 = Color32::from_rgba_premultiplied(8, 8, 12, 240);
    pub const HUD_BG: Color32 = Color32::from_rgba_premultiplied(8, 8, 12, 170);
}

#[allow(dead_code)]
pub mod sizes {
    pub const ROUNDING: u8 = 4;
    pub const WINDOW_ROUNDING: u8 = 6;
    pub const BORDER_WIDTH: f32 = 1.0;
    pub const ITEM_SPACING: f32 = 4.0;
    pub const BUTTON_PADDING_X: f32 = 16.0;
    pub const BUTTON_PADDING_Y: f32 = 6.0;
    pub const CHAT_WIDTH: f32 = 460.0;
    pub const CHAT_HEIGHT: f32 = 320.0;
    pub const PLAYER_LIST_WIDTH: f32 = 280.0;
    pub const CONNECT_WIDTH: f32 = 400.0;
    pub const SCOREBOARD_WIDTH: f32 = 540.0;
    pub const CONSOLE_WIDTH: f32 = 640.0;
    pub const CONSOLE_HEIGHT: f32 = 420.0;
    pub const TITLE_BAR_HEIGHT: f32 = 24.0;
}

pub fn apply(ctx: &egui::Context) {
    apply_style(ctx);
}

/// Полупрозрачная "панель" для overlay-блоков: фон, рамка, скруглённые углы, отступы.
pub fn panel_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(colors::BG_PANEL_SOLID)
        .stroke(Stroke::new(sizes::BORDER_WIDTH, colors::BORDER))
        .corner_radius(CornerRadius::same(sizes::WINDOW_ROUNDING))
        .inner_margin(egui::Margin::same(10))
        .shadow(Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(110),
        })
}

/// Лёгкая overlay-панель без сильной тени (HUD, чат-passive).
pub fn overlay_frame(fill: Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(sizes::BORDER_WIDTH, colors::BORDER))
        .corner_radius(CornerRadius::same(sizes::ROUNDING))
        .inner_margin(egui::Margin::symmetric(10, 8))
}

/// Шапка окна с золотой акцентной полоской слева.
pub fn header_bar(ui: &mut egui::Ui, title: &str, hint: Option<&str>) {
    let h = sizes::TITLE_BAR_HEIGHT;
    egui::Frame::NONE
        .fill(colors::BG_TITLE)
        .corner_radius(CornerRadius {
            nw: sizes::WINDOW_ROUNDING,
            ne: sizes::WINDOW_ROUNDING,
            sw: 0,
            se: 0,
        })
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.set_min_height(h);
            ui.horizontal_centered(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::new(3.0, h - 8.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, colors::GOLD);
                ui.add_space(2.0);
                ui.label(
                    RichText::new(title)
                        .size(11.5)
                        .color(colors::GOLD)
                        .extra_letter_spacing(2.5)
                        .strong(),
                );
                if let Some(hint) = hint {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(hint)
                                .size(10.0)
                                .color(colors::TEXT_MUTED),
                        );
                    });
                }
            });
        });
}

/// Цветная "точка" статуса (ping, online/offline, индикатор локального игрока).
pub fn status_dot(ui: &mut egui::Ui, color: Color32, size: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        egui::Vec2::new(size, size),
        egui::Sense::hover(),
    );
    ui.painter().circle_filled(rect.center(), size * 0.5, color);
    resp
}

/// "Чип" с горячей клавишей: тонкая рамка, моно-шрифт, тёмный фон.
pub fn hotkey_chip(ui: &mut egui::Ui, key: &str) {
    egui::Frame::NONE
        .fill(colors::BG_WIDGET)
        .stroke(Stroke::new(1.0, colors::BORDER))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(egui::Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(
                RichText::new(key)
                    .size(10.0)
                    .color(colors::GOLD_DIM)
                    .monospace(),
            );
        });
}

/// Применить альфу поверх премультиплицированного цвета.
pub fn fade(color: Color32, alpha: f32) -> Color32 {
    let a = alpha.clamp(0.0, 1.0);
    let [r, g, b, ca] = color.to_array();
    Color32::from_rgba_premultiplied(
        (r as f32 * a) as u8,
        (g as f32 * a) as u8,
        (b as f32 * a) as u8,
        (ca as f32 * a) as u8,
    )
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
    style.spacing.interact_size = egui::Vec2::new(40.0, 22.0);
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.floating = true;

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