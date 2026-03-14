//! Обработка lifecycle-событий (упрощённая версия для devtools).

use sdk::addresses::constants::game_events as ev;
use common::logger;

fn should_log(event_id: i32) -> bool {
    matches!(event_id,
        ev::MISSION_BEFORE_OPEN | ev::MISSION_AFTER_OPEN
        | ev::LOADING_PROCESS_STARTED | ev::LOADING_PROCESS_FINISHED
        | ev::GAME_INIT | ev::GAME_DONE
        | ev::GAME_PAUSED | ev::GAME_UNPAUSED
        | ev::SHUTDOWN
    )
}

pub fn process_fired_event(event_id: i32) {
    // FOV re-apply на lifecycle переходах
    match event_id {
        ev::MISSION_AFTER_OPEN | ev::GAME_INIT
        | ev::LOADING_PROCESS_FINISHED | ev::GAME_UNPAUSED => {
            crate::tools::camera_state::request_apply();
        }
        _ => {}
    }

    if should_log(event_id) {
        let name = crate::events::event_name(event_id);
        logger::info(&format!("[events] {name} ({event_id})"));
    }

    crate::state::on_event(event_id);
}

pub fn event_name(event_id: i32) -> &'static str {
    match event_id {
        ev::SYSTEM_INIT => "System Init",
        ev::GAME_TICK_ALWAYS => "Game Tick Always",
        ev::GAME_RENDER => "Game Render",
        ev::MISSION_BEFORE_OPEN => "Mission Before Open",
        ev::MISSION_AFTER_OPEN => "Mission After Open",
        ev::LOADING_PROCESS_STARTED => "Loading Started",
        ev::LOADING_PROCESS_FINISHED => "Loading Finished",
        ev::GAME_INIT => "Game Init",
        ev::GAME_DONE => "Game Done",
        ev::GAME_PAUSED => "Game Paused",
        ev::GAME_UNPAUSED => "Game Unpaused",
        ev::SHUTDOWN => "Shutdown",
        _ => "Other",
    }
}