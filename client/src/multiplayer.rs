//! Orchestration layer мультиплеера.
//!
//! Единая точка координации на game thread:
//! - сбор и отправка локального snapshot'а
//! - обработка локальных событий → network
//! - применение входящих пакетов от сервера
//! - синхронизация UI состояния
//!
//! Не содержит transport логики — это зона `network.rs`.
//! Не содержит gameplay логики — это зоны `player_tracker`, `vehicle_tracker`, `player_events`.

use common::logger;

/// Инициализация multiplayer subsystem.
pub fn init() {
    logger::debug("[multiplayer] инициализирован");
}

/// Вызывается на game thread каждый tick.
///
/// Порядок:
/// 1. auto-disconnect при невалидной сессии
/// 2. обновление локального трекера (snapshot + события)
/// 3. обновление vehicle трекера
/// 4. обработка накопленных локальных событий → network queue
/// 5. применение входящих пакетов от сервера
pub fn on_main_thread_tick() {
    crate::network::auto_disconnect_if_session_invalid();

    crate::player_tracker::update_main_thread();
    crate::vehicle_tracker::update_main_thread();

    crate::player_events::process_pending();

    // Блокируем управление игроком когда открыт UI (чат, меню подключения)
    crate::overlay::multiplayer_ui::sync_player_controls();

    crate::network::poll_main_thread();
}
