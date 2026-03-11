//! Фоновый мониторинг состояния игры.
//!
//! Работает в отдельном потоке, запускается из initialize().
//! Каждые MONITOR_INTERVAL_SECS секунд проверяет:
//! - переходы между состояниями сессии (меню → загрузка → игра)
//! - появление и готовность игрока
//! - текущий баланс (для debug-лога)
//!
//! При первом появлении готового игрока снимает полный snapshot
//! (инвентарь, тип entity, деньги) — это помогает убедиться
//! что инжект и реверс работают корректно.

use std::time::Duration;

use common::logger;
use sdk::game::Player;

use crate::{
    runtime,
    state::{self, GameSessionState},
};

/// Интервал между проверками (секунды).
///
/// 5 секунд — достаточно для мониторинга, не засоряет лог.
/// Для более частых проверок есть PlayerTracker на main thread.
const MONITOR_INTERVAL_SECS: u64 = 5;

/// Точка входа фонового мониторинга.
///
/// Вызывается из `std::thread::spawn(monitor::run)` в initialize().
/// Работает до получения сигнала shutdown.
pub fn run() {
    logger::debug("[monitor] запущен");

    let mut last_state = state::get();
    let mut snapshot_done = false;

    loop {
        if runtime::is_shutting_down() {
            logger::debug("[monitor] остановка");
            break;
        }

        std::thread::sleep(Duration::from_secs(MONITOR_INTERVAL_SECS));

        // Обновляем состояние сессии через polling fallback.
        // Основные переходы приходят через FireEventById hook,
        // но некоторые могут быть пропущены — polling страхует.
        let current = state::refresh_from_runtime();

        // Обработка смены состояния
        if current != last_state {
            on_state_changed(current, &mut snapshot_done);
            last_state = current;
        }

        // Периодический debug-лог текущего состояния
        log_periodic_status(current, &mut snapshot_done);

        // ShuttingDown — выходим после последней итерации
        if current == GameSessionState::ShuttingDown {
            break;
        }
    }
}

/// Обработка перехода между состояниями.
///
/// Здесь логируем важные моменты: вход в игру, выход в меню,
/// загрузка. При входе в игру снимаем snapshot игрока.
fn on_state_changed(new_state: GameSessionState, snapshot_done: &mut bool) {
    match new_state {
        GameSessionState::InGame => {
            // Попробовать снять snapshot сразу при переходе в InGame.
            // Если игрок ещё не готов — snapshot_done останется false,
            // и мы попробуем снова в log_periodic_status.
            if let Some(player) = Player::get_active() {
                logger::info("[monitor] вошли в игру");
                log_player_snapshot(&player);
                *snapshot_done = true;
            }
        }
        GameSessionState::FrontendMenu => {
            logger::info("[monitor] главное меню");
            *snapshot_done = false;
        }
        GameSessionState::Loading => {
            logger::info("[monitor] загрузка");
            *snapshot_done = false;
        }
        GameSessionState::Paused => {
            logger::info("[monitor] пауза");
        }
        GameSessionState::Boot => {
            logger::info("[monitor] начальная загрузка");
            *snapshot_done = false;
        }
        GameSessionState::ShuttingDown => {
            // Ничего не делаем — выйдем из цикла
        }
    }
}

/// Периодический debug-лог текущего состояния.
///
/// В InGame показывает баланс — полезно видеть в логе
/// что клиент жив и данные читаются. Если snapshot ещё
/// не снят — пробуем снять (бывает что игрок появляется
/// чуть позже перехода в InGame).
fn log_periodic_status(state: GameSessionState, snapshot_done: &mut bool) {
    match state {
        GameSessionState::InGame => {
            if let Some(player) = Player::get_active() {
                // Отложенный snapshot — игрок мог появиться позже
                if !*snapshot_done && player.is_ready() {
                    log_player_snapshot(&player);
                    *snapshot_done = true;
                }

                // Краткий debug: баланс (не засоряет лог)
                let money = player
                    .get_money_formatted()
                    .unwrap_or_else(|| "кошелёк не готов".into());
                logger::debug(&format!("[monitor] в игре | {money}"));
            } else {
                logger::debug("[monitor] в игре, но указатель на игрока отсутствует");
            }
        }
        GameSessionState::FrontendMenu => {
            logger::debug("[monitor] меню");
        }
        GameSessionState::Loading => {
            logger::debug("[monitor] загрузка");
        }
        GameSessionState::Paused => {
            logger::debug("[monitor] пауза");
        }
        GameSessionState::Boot => {
            logger::debug("[monitor] boot");
        }
        GameSessionState::ShuttingDown => {}
    }
}

/// Снять и залогировать полный snapshot игрока.
///
/// Выводит: адрес C_Human, состояние инвентаря, тип entity,
/// количество слотов, текущий баланс. Помогает убедиться
/// что вся цепочка указателей работает.
fn log_player_snapshot(player: &Player) {
    player.log_debug_info();

    if player.is_wallet_ready() {
        match player.get_money_cents() {
            Some(c) => logger::info(&format!(
                "[monitor] баланс: {} центов = $ {}.{:02}",
                c, c / 100, (c % 100).abs(),
            )),
            None => logger::info("[monitor] баланс: не удалось прочитать"),
        }
    } else {
        logger::info("[monitor] кошелёк ещё не инициализирован");
    }
}