// Горячие клавиши клиента

use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::overlay::egui_input::just_pressed;
use crate::state;
use crate::utils;
use common::logger;

const INPUT_POLL_MS: u64 = 16;

pub fn log_keybinds() {
    logger::info("  Горячие клавиши:");
    logger::info("    DELETE  — Выгрузить клиент");
    logger::info("    F9      — Показать/скрыть overlay");
    logger::info("    F10     — Показать/скрыть отладку (FPS/позиция)");
}

pub fn run() {
    logger::debug("[input] запущен");

    loop {
        if state::is_shutting_down() {
            break;
        }

        std::thread::sleep(Duration::from_millis(INPUT_POLL_MS));

        // Обрабатываем ввод только если окно в фокусе
        if !utils::is_window_focused() {
            continue;
        }

        // DELETE — выгрузка клиента
        if just_pressed(VK_DELETE) {
            logger::info("[input] запрошено завершение");
            state::shutdown();
            break;
        }

        // F9 — показать/скрыть overlay
        if just_pressed(VK_F9) {
            crate::overlay::toggle_visibility();
        }

        // F10 — показать/скрыть отладку
        if just_pressed(VK_F10) {
            crate::overlay::state::toggle_debug();
        }
    }
}
