use std::sync::atomic::{AtomicU8, Ordering};
use common::logger;
use sdk::addresses::constants::game_events as ev;
use sdk::game::{self, Player};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSessionState {
    Boot = 0,
    FrontendMenu = 1,
    Loading = 2,
    InGame = 3,
    Paused = 4,
    ShuttingDown = 5,
}

static CURRENT_STATE: AtomicU8 = AtomicU8::new(0);

fn decode(v: u8) -> GameSessionState {
    match v {
        0 => GameSessionState::Boot,
        1 => GameSessionState::FrontendMenu,
        2 => GameSessionState::Loading,
        3 => GameSessionState::InGame,
        4 => GameSessionState::Paused,
        5 => GameSessionState::ShuttingDown,
        _ => GameSessionState::Boot,
    }
}

impl GameSessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Boot => "Boot",
            Self::FrontendMenu => "FrontendMenu",
            Self::Loading => "Loading",
            Self::InGame => "InGame",
            Self::Paused => "Paused",
            Self::ShuttingDown => "ShuttingDown",
        }
    }
}

pub fn get() -> GameSessionState {
    decode(CURRENT_STATE.load(Ordering::Acquire))
}

pub fn set(next: GameSessionState) -> GameSessionState {
    let prev = decode(CURRENT_STATE.swap(next as u8, Ordering::AcqRel));
    if prev != next {
        logger::info(&format!("[state] {} -> {}", prev.as_str(), next.as_str()));
    }
    prev
}

pub fn mark_shutting_down() { set(GameSessionState::ShuttingDown); }

pub fn on_event(event_id: i32) {
    match event_id {
        ev::LOADING_PROCESS_STARTED | ev::MISSION_BEFORE_OPEN | ev::MISSION_BEFORE_CLOSE => {
            set(GameSessionState::Loading);
        }
        ev::MISSION_AFTER_OPEN | ev::LOADING_PROCESS_FINISHED | ev::GAME_INIT => {
            let _ = refresh_from_runtime();
        }
        ev::MISSION_AFTER_CLOSE | ev::NO_GAME_START | ev::NO_GAME_END | ev::GAME_DONE => {
            set(GameSessionState::FrontendMenu);
        }
        ev::GAME_PAUSED => {
            if matches!(get(), GameSessionState::InGame) { set(GameSessionState::Paused); }
        }
        ev::GAME_UNPAUSED => {
            if matches!(get(), GameSessionState::Paused) { set(GameSessionState::InGame); }
        }
        ev::SHUTDOWN => { mark_shutting_down(); }
        _ => {}
    }
}

pub fn refresh_from_runtime() -> GameSessionState {
    let current = get();
    if matches!(current, GameSessionState::ShuttingDown) {
        return current;
    }

    let next = if !game::is_game_initialized() {
        match current {
            GameSessionState::Boot => GameSessionState::Boot,
            _ => GameSessionState::FrontendMenu,
        }
    } else if let Some(player) = Player::get_active() {
        if player.is_ready() {
            match current {
                GameSessionState::Paused => GameSessionState::Paused,
                _ => GameSessionState::InGame,
            }
        } else {
            GameSessionState::Loading
        }
    } else {
        match current {
            GameSessionState::Loading => GameSessionState::Loading,
            GameSessionState::Boot => GameSessionState::Boot,
            _ => GameSessionState::FrontendMenu,
        }
    };

    set(next);
    next
}