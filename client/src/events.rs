//! Обработка lifecycle-событий игры.
//!
//! Этот модуль работает поверх hook на `GameCallbackManager::FireEventById`
//! и переводит сырой `event_id` в:
/// - логируемые события
/// - изменение session state
/// - изменение app focus state

use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use sdk::addresses::constants::game_events as ev;

/// Текущее состояние фокуса окна игры.
///
/// Используется для определения alt-tab / возврата в игру.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppFocusState {
    Active,
    Inactive,
}

/// true = окно игры активно, false = игра потеряла фокус.
static APP_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn app_focus_state() -> AppFocusState {
    if APP_ACTIVE.load(Ordering::Acquire) {
        AppFocusState::Active
    } else {
        AppFocusState::Inactive
    }
}

/// Обновляет focus state и пишет лог только если состояние реально изменилось.
fn set_app_focus(active: bool) {
    let old = APP_ACTIVE.swap(active, Ordering::AcqRel);
    if old != active {
        logger::info(&format!(
            "[app] focus -> {}",
            if active { "Active" } else { "Inactive" }
        ));
    }
}

/// Человекочитаемое имя event_id.
pub fn event_name(event_id: i32) -> &'static str {
    match event_id {
        ev::SYSTEM_INIT => "System Init",
        ev::SYSTEM_DONE => "System Done",
        ev::GAME_TICK => "Game Tick",
        ev::GAME_TICK_PAUSED => "Game Tick Paused",
        ev::GAME_TICK_ALWAYS => "Game Tick Always",
        ev::GAME_RENDER => "Game Render",
        ev::MISSION_QUIT => "Mission Quit",
        ev::MISSION_BEFORE_OPEN => "Mission Before Open",
        ev::MISSION_AFTER_OPEN => "Mission After Open",
        ev::MISSION_BEFORE_CLOSE => "Mission Before Close",
        ev::MISSION_AFTER_CLOSE => "Mission After Close",
        ev::GAME_INIT => "Game Init",
        ev::GAME_DONE => "Game Done",
        ev::INVALIDATE_ENTITY => "Invalidate Entity",
        ev::INVALIDATE_FRAME => "Invalidate Frame",
        ev::WRITE_GAME_INFO => "Write Game Info",
        ev::READ_GAME_INFO => "Read Game Info",
        ev::GAME_RESTORE => "Game Restore",
        ev::NO_GAME_START => "No Game Start",
        ev::NO_GAME_END => "No Game End",
        ev::NO_GAME_TICK => "No Game Tick",
        ev::NO_GAME_RENDER => "No Game Render",
        ev::NO_GAME_AFTER_GAME_LOOP => "No Game After Game Loop",
        ev::COLLISIONS_LOADED => "Collisions Loaded",
        ev::APACK_FROM_SDS_LOADED => "APack From SDS Loaded",
        ev::REGISTER_GAME_SAVE_CB => "Register Game Save CB",
        ev::GAMEPARAMS_CHANGED => "GameParams Changed",
        ev::GAMEPARAMS_PRESAVE => "GameParams Presave",
        ev::APP_DEACTIVATE => "App Deactivate",
        ev::APP_ACTIVATE => "App Activate",
        ev::LOADING_PROCESS_STARTED => "Loading Process Started",
        ev::LOADING_PROCESS_FINISHED => "Loading Process Finished",
        ev::GAME_PAUSED => "Game Paused",
        ev::GAME_UNPAUSED => "Game Unpaused",
        ev::LOADING_FADE_FINISHED => "Loading Fade Finished",
        ev::SLOT_WAITING_TICK => "Slot Waiting Tick",
        ev::SLOT_WAITING_RENDER => "Slot Waiting Render",
        ev::SHUTDOWN => "Shutdown",
        ev::WEATHER_MANAGER_CREATED => "Weather Manager Created",
        _ => "Unknown Event",
    }
}

/// Возвращает true только для реально значимых lifecycle-событий.
fn should_log_event(event_id: i32) -> bool {
    matches!(
        event_id,
        ev::MISSION_BEFORE_OPEN
            | ev::MISSION_AFTER_OPEN
            | ev::MISSION_BEFORE_CLOSE
            | ev::MISSION_AFTER_CLOSE
            | ev::LOADING_PROCESS_STARTED
            | ev::LOADING_PROCESS_FINISHED
            | ev::LOADING_FADE_FINISHED
            | ev::GAME_PAUSED
            | ev::GAME_UNPAUSED
            | ev::APP_ACTIVATE
            | ev::APP_DEACTIVATE
            | ev::GAME_INIT
            | ev::GAME_DONE
            | ev::NO_GAME_START
            | ev::NO_GAME_END
            | ev::SHUTDOWN
    )
}

/// Основная обработка одного fired event.
///
/// Вызывается из hook на `FireEventById`.
pub fn process_fired_event(event_id: i32) {
    if should_log_event(event_id) {
        logger::info(&format!(
            "[events] {} ({})",
            event_name(event_id),
            event_id
        ));
    }

    // Обработка фокуса окна игры
    match event_id {
        ev::APP_ACTIVATE => set_app_focus(true),
        ev::APP_DEACTIVATE => set_app_focus(false),
        _ => {}
    }

    // Передаём lifecycle-событие в state machine
    crate::state::on_event(event_id);
}