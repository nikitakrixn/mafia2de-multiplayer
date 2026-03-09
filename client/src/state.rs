//! Состояние игровой сессии клиента.
//!
//! Работает в смешанном режиме:
//! - callback lifecycle события через FireEventById
//! - polling fallback через refresh_from_runtime()

use std::sync::atomic::{AtomicU8, Ordering};

use common::logger;
use sdk::{
    addresses::constants::game_events as ev,
    game::{self, Player},
};

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

static CURRENT_STATE: AtomicU8 = AtomicU8::new(GameSessionState::Boot as u8);

fn decode_state(value: u8) -> GameSessionState {
    match value {
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
    decode_state(CURRENT_STATE.load(Ordering::Acquire))
}

pub fn set(next: GameSessionState) -> GameSessionState {
    let prev_raw = CURRENT_STATE.swap(next as u8, Ordering::AcqRel);
    let prev = decode_state(prev_raw);

    if prev != next {
        logger::info(&format!(
            "[state] {} -> {}",
            prev.as_str(),
            next.as_str()
        ));
    }

    prev
}

pub fn mark_paused() {
    if matches!(get(), GameSessionState::InGame) {
        set(GameSessionState::Paused);
    }
}

pub fn mark_unpaused() {
    if matches!(get(), GameSessionState::Paused) {
        set(GameSessionState::InGame);
    }
}

pub fn mark_shutting_down() {
    set(GameSessionState::ShuttingDown);
}

/// Lifecycle event -> state transition.
pub fn on_event(event_id: i32) {
    match event_id {
        ev::LOADING_PROCESS_STARTED
        | ev::MISSION_BEFORE_OPEN
        | ev::MISSION_BEFORE_CLOSE => {
            set(GameSessionState::Loading);
        }

        ev::MISSION_AFTER_OPEN
        | ev::LOADING_PROCESS_FINISHED
        | ev::GAME_INIT => {
            let _ = refresh_from_runtime();
        }

        ev::MISSION_AFTER_CLOSE
        | ev::NO_GAME_START
        | ev::NO_GAME_END
        | ev::GAME_DONE => {
            set(GameSessionState::FrontendMenu);
        }

        ev::GAME_PAUSED => {
            mark_paused();
        }

        ev::GAME_UNPAUSED => {
            mark_unpaused();
        }

        ev::SHUTDOWN => {
            mark_shutting_down();
        }

        _ => {}
    }
}

/// Polling fallback на случай, если какое-то lifecycle-событие не пришло.
pub fn refresh_from_runtime() -> GameSessionState {
    let current = get();

    if matches!(current, GameSessionState::ShuttingDown) {
        return GameSessionState::ShuttingDown;
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
            GameSessionState::Paused => GameSessionState::FrontendMenu,
            GameSessionState::Boot => GameSessionState::Boot,
            _ => GameSessionState::FrontendMenu,
        }
    };

    set(next);
    next
}