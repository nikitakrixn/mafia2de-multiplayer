//! Тестовые данные для демонстрации UI.

use super::state;

pub fn populate() {
    state::set_connection(true, "127.0.0.1:7788");

    state::add_player(1, "Vito".into(), 25, true);
    state::add_player(2, "Joe".into(), 45, false);
    state::add_player(3, "Henry".into(), 78, false);
    state::add_player(4, "Eddie".into(), 120, false);
    state::add_player(5, "Tommy".into(), 35, false);

    state::add_system_msg("Сервер запущен");
    state::add_chat_msg("Joe", "Привет всем!");
    state::add_chat_msg("Vito", "Привет Joe!");
    state::add_chat_msg("Henry", "Кто-нибудь хочет ограбить банк?");
    state::add_system_msg("Eddie присоединился к игре");
    state::add_chat_msg("Eddie", "Всем привет!");
    state::add_chat_msg("Tommy", "Поехали на гонки!");

    common::logger::info("[demo] test data loaded");
}

pub fn clear() {
    for id in 1..=5 {
        state::remove_player(id);
    }
    state::set_connection(false, "Отключен");
    common::logger::info("[demo] data cleared");
}

pub fn simulate_pings() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    for id in 1..=5 {
        let ping = 20 + ((seed.wrapping_mul(id).wrapping_mul(1103515245)) % 130);
        state::update_ping(id, ping);
    }
}