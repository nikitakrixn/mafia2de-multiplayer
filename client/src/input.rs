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
    logger::info("    F2      — Меню подключения к серверу");
    logger::info("    F3      — Список игроков онлайн");
    logger::info("    T       — Открыть чат");
    logger::info("    F5      — Spawn mock remote player");
    logger::info("    F6      — Move mock remote player (по кругу)");
    logger::info("    F7      — Send mock remote event (Shot)");
    logger::info("    F8      — Despawn mock remote player");
    logger::info("    F11     — Загрузить демо-данные (тест UI)");
    logger::info("    F12     — Очистить демо-данные");
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

        // F11 — загрузить демо-данные для тестирования UI
        if just_pressed(VK_F11) {
            logger::info("[input] загрузка демо-данных мультиплеера");
            crate::overlay::multiplayer_demo::populate_demo_data();
        }

        // F12 — очистить демо-данные
        if just_pressed(VK_F12) {
            logger::info("[input] очистка демо-данных мультиплеера");
            crate::overlay::multiplayer_demo::clear_demo_data();
        }

        // F5 — заспавнить mock remote player
        if just_pressed(VK_F5) {
            crate::multiplayer_test::spawn_mock_remote();
        }

        // F6 — сдвинуть mock remote player по кругу
        if just_pressed(VK_F6) {
            crate::multiplayer_test::step_mock_remote();
        }

        // F7 — отправить mock remote event
        if just_pressed(VK_F7) {
            crate::multiplayer_test::send_mock_remote_event();
        }

        // F8 — удалить mock remote player
        if just_pressed(VK_F8) {
            crate::multiplayer_test::despawn_mock_remote();
        }
    }
}
