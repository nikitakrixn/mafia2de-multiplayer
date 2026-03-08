//! PlayerTracker — локальный трекер изменений игрока.
//!
//! Работает на главном игровом потоке из `Game Tick Always` hook.
//! Его задача — превращать сырые snapshot'ы игрока в полезные high-level события:
//! - деньги изменились
//! - игрок начал движение
//! - игрок остановился
//! - игрок телепортировался
//! - сменился lock controls
//! - сменился control style
//!
//! Это очень полезно для будущего multiplayer-клиента,
//! даже до реверса Human/Entity message system.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use common::logger;
use sdk::game::{Player, player::Vec3};

use crate::state::{self, GameSessionState};

/// Не обновляем tracker каждый тик, чтобы не спамить и не делать лишних чтений.
const TRACK_INTERVAL_MS: u64 = 150;

/// Если расстояние между snapshot'ами больше этого порога — считаем это телепортом.
const TELEPORT_DISTANCE: f32 = 40.0;

/// Если расстояние больше этого порога — считаем, что игрок движется.
const MOVE_EPSILON: f32 = 0.10;

/// Сколько подряд "почти неподвижных" snapshot'ов нужно,
/// чтобы объявить остановку движения.
const STOP_TICKS_REQUIRED: u32 = 4;

/// Один снимок состояния игрока.
#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub position: Option<Vec3>,
    pub money_cents: Option<i64>,
    pub controls_locked: Option<bool>,
    pub control_style: Option<String>,
}

/// Внутреннее состояние трекера.
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

/// Вызывается из main-thread service loop.
///
/// Безопасно звать часто: внутри стоит throttling по времени.
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

        // ─────────────────────────────────────────────────────────────
        // Деньги
        // ─────────────────────────────────────────────────────────────
        if previous.money_cents != current.money_cents {
            if let (Some(old), Some(new)) = (previous.money_cents, current.money_cents) {
                let delta = new - old;
                logger::info(&format!(
                    "[tracker] money changed: {} -> {} (delta: {})",
                    format_money(old),
                    format_money(new),
                    format_money_delta(delta),
                ));
            }
        }

        // ─────────────────────────────────────────────────────────────
        // Lock controls
        // ─────────────────────────────────────────────────────────────
        if previous.controls_locked != current.controls_locked {
            if let Some(locked) = current.controls_locked {
                logger::info(&format!(
                    "[tracker] controls locked changed -> {}",
                    locked
                ));
            }
        }

        // ─────────────────────────────────────────────────────────────
        // Control style
        // ─────────────────────────────────────────────────────────────
        if previous.control_style != current.control_style {
            if let Some(style) = current.control_style.as_deref() {
                logger::info(&format!(
                    "[tracker] control style changed -> \"{}\"",
                    style
                ));
            }
        }

        // ─────────────────────────────────────────────────────────────
        // Позиция / движение / телепорт
        // ─────────────────────────────────────────────────────────────
        match (previous.position, current.position) {
            (Some(old_pos), Some(new_pos)) => {
                let dist = distance(old_pos, new_pos);

                if dist >= TELEPORT_DISTANCE {
                    logger::info(&format!(
                        "[tracker] teleport detected: {} -> {} (dist {:.2})",
                        old_pos, new_pos, dist
                    ));
                    self.moving = false;
                    self.still_ticks = 0;
                } else if dist >= MOVE_EPSILON {
                    if !self.moving {
                        logger::info(&format!(
                            "[tracker] movement started at {}",
                            new_pos
                        ));
                    }
                    self.moving = true;
                    self.still_ticks = 0;
                } else if self.moving {
                    self.still_ticks += 1;
                    if self.still_ticks >= STOP_TICKS_REQUIRED {
                        self.moving = false;
                        self.still_ticks = 0;
                        logger::info(&format!(
                            "[tracker] movement stopped at {}",
                            new_pos
                        ));
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

fn format_money(cents: i64) -> String {
    format!("$ {}.{:02}", cents / 100, (cents % 100).abs())
}

fn format_money_delta(cents: i64) -> String {
    if cents >= 0 {
        format!("+$ {}.{:02}", cents / 100, (cents % 100).abs())
    } else {
        let abs = cents.abs();
        format!("-$ {}.{:02}", abs / 100, (abs % 100).abs())
    }
}