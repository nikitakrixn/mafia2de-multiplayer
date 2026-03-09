//! PlayerTracker — локальный трекер изменений игрока.
//!
//! Работает на главном игровом потоке из `Game Tick Always` hook.
//! Превращает snapshot'ы игрока в высокоуровневые события.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use common::logger;
use sdk::game::{Player, player::Vec3};

use crate::{
    player_events::{self, PlayerEvent},
    state::{self, GameSessionState},
};

const TRACK_INTERVAL_MS: u64 = 150;
const TELEPORT_DISTANCE: f32 = 40.0;
const MOVE_EPSILON: f32 = 0.10;
const STOP_TICKS_REQUIRED: u32 = 4;

#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub position: Option<Vec3>,
    pub money_cents: Option<i64>,
    pub controls_locked: Option<bool>,
    pub control_style: Option<String>,
}

#[derive(Debug)]
struct PlayerTracker {
    last_update: Option<Instant>,
    last_snapshot: Option<PlayerSnapshot>,
    moving: bool,
    still_ticks: u32,
}

static TRACKER: OnceLock<Mutex<PlayerTracker>> = OnceLock::new();

fn tracker() -> &'static Mutex<PlayerTracker> {
    TRACKER.get_or_init(|| {
        Mutex::new(PlayerTracker {
            last_update: None,
            last_snapshot: None,
            moving: false,
            still_ticks: 0,
        })
    })
}

pub fn init() {
    let _ = tracker();
}

pub fn update_main_thread() {
    if !matches!(state::get(), GameSessionState::InGame) {
        reset();
        return;
    }

    let Some(player) = Player::get_active() else {
        reset();
        return;
    };

    if !player.is_ready() {
        reset();
        return;
    }

    let mut guard = match tracker().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[tracker] mutex poisoned");
            return;
        }
    };

    if !guard.should_update() {
        return;
    }

    let snapshot = capture_snapshot(&player);
    guard.process_snapshot(snapshot);
}

fn reset() {
    let mut guard = match tracker().lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    guard.last_snapshot = None;
    guard.moving = false;
    guard.still_ticks = 0;
    guard.last_update = None;
}

fn capture_snapshot(player: &Player) -> PlayerSnapshot {
    PlayerSnapshot {
        position: player.get_position(),
        money_cents: player.get_money_cents(),
        controls_locked: player.are_controls_locked(),
        control_style: player.get_control_style_str(),
    }
}

fn distance(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

impl PlayerTracker {
    fn should_update(&mut self) -> bool {
        let now = Instant::now();

        match self.last_update {
            Some(last) if now.duration_since(last) < Duration::from_millis(TRACK_INTERVAL_MS) => {
                false
            }
            _ => {
                self.last_update = Some(now);
                true
            }
        }
    }

    fn process_snapshot(&mut self, current: PlayerSnapshot) {
        let Some(previous) = self.last_snapshot.as_ref() else {
            logger::debug("[tracker] initial snapshot captured");
            self.last_snapshot = Some(current);
            return;
        };

        if previous.money_cents != current.money_cents {
            if let (Some(old), Some(new)) = (previous.money_cents, current.money_cents) {
                player_events::push(PlayerEvent::MoneyChanged {
                    old_cents: old,
                    new_cents: new,
                    delta_cents: new - old,
                });
            }
        }

        if previous.controls_locked != current.controls_locked {
            if let Some(locked) = current.controls_locked {
                player_events::push(PlayerEvent::ControlsLockedChanged { locked });
            }
        }

        if previous.control_style != current.control_style {
            if let Some(style) = current.control_style.clone() {
                player_events::push(PlayerEvent::ControlStyleChanged { style });
            }
        }

        match (previous.position, current.position) {
            (Some(old_pos), Some(new_pos)) => {
                let dist = distance(old_pos, new_pos);

                if dist >= TELEPORT_DISTANCE {
                    player_events::push(PlayerEvent::Teleported {
                        from: old_pos,
                        to: new_pos,
                        distance: dist,
                    });
                    self.moving = false;
                    self.still_ticks = 0;
                } else if dist >= MOVE_EPSILON {
                    if !self.moving {
                        player_events::push(PlayerEvent::MovementStarted { pos: new_pos });
                    }
                    self.moving = true;
                    self.still_ticks = 0;
                } else if self.moving {
                    self.still_ticks += 1;
                    if self.still_ticks >= STOP_TICKS_REQUIRED {
                        self.moving = false;
                        self.still_ticks = 0;
                        player_events::push(PlayerEvent::MovementStopped { pos: new_pos });
                    }
                }
            }
            _ => {
                self.moving = false;
                self.still_ticks = 0;
            }
        }

        self.last_snapshot = Some(current);
    }
}