//! Демонстрация мультиплеер UI — тестовые данные для проверки интерфейса.
//!
//! Этот модуль можно вызвать из devtools для заполнения UI тестовыми данными.

use super::multiplayer_ui;

/// Заполнить UI тестовыми данными для демонстрации.
pub fn populate_demo_data() {
    // Симулируем подключение
    multiplayer_ui::set_connection_status(true, "Подключено к 127.0.0.1:7777".to_string());

    // Добавляем тестовых игроков
    multiplayer_ui::add_player(1, "Vito".to_string(), 25, true);
    multiplayer_ui::add_player(2, "Joe".to_string(), 45, false);
    multiplayer_ui::add_player(3, "Henry".to_string(), 78, false);
    multiplayer_ui::add_player(4, "Eddie".to_string(), 120, false);
    multiplayer_ui::add_player(5, "Tommy".to_string(), 35, false);

    // Добавляем тестовые сообщения в чат
    multiplayer_ui::add_system_message("Сервер запущен".to_string());
    multiplayer_ui::add_chat_message("Joe".to_string(), "Привет всем!".to_string());
    multiplayer_ui::add_chat_message("Vito".to_string(), "Привет Joe!".to_string());
    multiplayer_ui::add_chat_message(
        "Henry".to_string(),
        "Кто-нибудь хочет ограбить банк?".to_string(),
    );
    multiplayer_ui::add_system_message("Eddie присоединился к игре".to_string());
    multiplayer_ui::add_chat_message("Eddie".to_string(), "Всем привет!".to_string());
    multiplayer_ui::add_chat_message("Tommy".to_string(), "Поехали на гонки!".to_string());

    common::logger::info("[multiplayer-demo] тестовые данные загружены");
}

/// Очистить все данные UI.
pub fn clear_demo_data() {
    // Удаляем игроков по одному для демонстрации
    for id in 1..=5 {
        multiplayer_ui::remove_player(id);
    }

    multiplayer_ui::set_connection_status(false, "Отключен".to_string());

    common::logger::info("[multiplayer-demo] данные очищены");
}

/// Симулировать обновление пинга игроков.
pub fn simulate_ping_updates() {
    // Простая симуляция изменения пинга
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    for id in 1..=5 {
        // Простой псевдослучайный пинг на основе времени и ID
        let ping = 20 + ((seed.wrapping_mul(id).wrapping_mul(1103515245)) % 130);
        multiplayer_ui::update_player_ping(id, ping);
    }
}
