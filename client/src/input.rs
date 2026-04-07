use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::overlay::input::just_pressed;
use crate::state;
use crate::utils;
use common::logger;

const INPUT_POLL_MS: u64 = 16;

pub fn log_keybinds() {
    logger::info("  Горячие клавиши:");
    logger::info("    DELETE  - Выгрузить клиент");
    logger::info("    F9      - Показать/скрыть overlay");
    logger::info("    F10     - Показать/скрыть отладку (FPS/позиция)");
    logger::info("    F2      - Меню подключения к серверу");
    logger::info("    F3      - Список игроков онлайн");
    logger::info("    F4      - Lua консоль");
    logger::info("    T       - Открыть чат");
    logger::info("    TAB     - Скорборд (удерживать)");
    logger::info("    F11     - Загрузить демо-данные");
    logger::info("    F12     - Очистить демо-данные");
}

pub fn run() {
    logger::debug("[input] запущен");

    loop {
        if state::is_shutting_down() {
            break;
        }

        std::thread::sleep(Duration::from_millis(INPUT_POLL_MS));

        if !utils::is_window_focused() {
            continue;
        }

        if just_pressed(VK_DELETE) {
            logger::info("[input] запрошено завершение");
            state::shutdown();
            break;
        }

        if just_pressed(VK_F9) {
            crate::overlay::toggle_visibility();
        }

        if just_pressed(VK_F10) {
            crate::overlay::state::toggle_debug();
        }

        if just_pressed(VK_F11) {
            logger::info("[input] загрузка демо-данных");
            crate::overlay::demo::populate();
        }

        if just_pressed(VK_F12) {
            logger::info("[input] очистка демо-данных");
            crate::overlay::demo::clear();
        }
    }
}