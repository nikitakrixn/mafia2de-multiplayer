//! PlayerTracker — локальный трекер изменений игрока.
//!
//! Работает на главном игровом потоке из `Game Tick Always` hook.
//! Превращает snapshot'ы игрока в высокоуровневые события.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use common::logger;
use protocol::{NetPlayerSnapshot, NetVec3};

fn vec3_to_net(v: Vec3) -> NetVec3 {
    NetVec3 { x: v.x, y: v.y, z: v.z }
}
use sdk::game::Player;
use sdk::types::Vec3;

use crate::{
    player_events::{self, PlayerEvent},
    state::{self, GameSessionState},
};

const TRACK_INTERVAL_MS: u64 = 150;
const TELEPORT_DISTANCE: f32 = 40.0;
/// За один интервал `TRACK_INTERVAL_MS` нужно сместиться хотя бы на столько (м),
/// чтобы включить `moving` / `is_moving`. Иначе микродрейф пола даёт ложные
/// MovementStarted и на remote снова включается walk.
const MOVE_START_M: f32 = 0.22;
/// Ниже этого смещения за интервал считаем «почти стоим» — копим к снятию moving.
const MOVE_STOP_M: f32 = 0.07;
/// Сколько подряд «тихих» snapshot (~TRACK_INTERVAL_MS), чтобы снять moving.
const STOP_TICKS_REQUIRED: u32 = 2;

#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub position: Option<Vec3>,
    pub money_cents: Option<i64>,
    pub controls_locked: Option<bool>,
    pub control_style: Option<String>,
    pub in_vehicle: Option<bool>,
}

#[derive(Debug)]
struct PlayerTracker {
    last_update: Option<Instant>,
    last_snapshot: Option<PlayerSnapshot>,
    moving: bool,
    still_ticks: u32,
    last_is_aiming: bool,
}

static TRACKER: OnceLock<Mutex<PlayerTracker>> = OnceLock::new();
static NET_SNAPSHOT_TICK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn tracker() -> &'static Mutex<PlayerTracker> {
    TRACKER.get_or_init(|| {
        Mutex::new(PlayerTracker {
            last_update: None,
            last_snapshot: None,
            moving: false,
            still_ticks: 0,
            last_is_aiming: false,
        })
    })
}

pub fn init() {
    let _ = tracker();
}

pub fn update_main_thread() {
    match state::get() {
        GameSessionState::InGame => {}
        GameSessionState::Paused => {
            // На паузе не обновляем, но и не сбрасываем —
            // чтобы после unpause не было ложного "initial snapshot captured"
            // и ложных teleport/movement start.
            return;
        }
        _ => {
            reset();
            return;
        }
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

    // Отправляем network snapshot если подключены
    if crate::network::is_connected() {
        let is_moving = guard.moving;
        if let Some(mut net_snap) = capture_network_snapshot(&player) {
            net_snap.is_moving = is_moving;
            // Лог transition aim (чтобы видеть момент нажатия RMB).
            if net_snap.is_aiming != guard.last_is_aiming {
                guard.last_is_aiming = net_snap.is_aiming;
                if net_snap.is_aiming {
                    let d = net_snap.aim_dir.unwrap_or_default();
                    logger::info(&format!(
                        "[player-event] AimStart dir=({:.2},{:.2},{:.2})",
                        d.x, d.y, d.z
                    ));
                } else {
                    logger::info("[player-event] AimStop");
                }
            }
            crate::network::push_local_snapshot(net_snap);
        }
    }
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
        in_vehicle: player.is_in_vehicle(),
    }
}

/// Собрать сетевой snapshot локального игрока.
///
/// Это multiplayer-ready snapshot:
/// - только подтверждённые reverse'ом поля
/// - без сырых указателей
/// - без внутренних движковых объектов
///
/// Возвращает `None` если любое обязательное поле недоступно.
fn capture_network_snapshot(player: &Player) -> Option<NetPlayerSnapshot> {
    let player_id = crate::network::local_player_id()?;

    let position = player.get_position()?;
    let forward = player.get_forward_vector()?;
    let health = player.get_health()?;
    let is_dead = !player.is_alive()?;
    let state_code = player.get_state_code()?;
    let car_wrapper_state = player.get_car_wrapper_state()?;
    let ctrl_style_mask = player.get_ctrl_style_mask()?;
    let sub45c_state = player.get_sub45c_state()?;
    let in_vehicle = player.is_in_vehicle()?;

    // Aim state — для v0 используем forward как направление прицела.
    // (Точное aim direction берётся из камеры; forward — приемлемая
    // аппроксимация на стороне remote proxy, тем более что
    // SetupAimDir с ним вызывается в TickPrePhysics в любом случае)
    // Я помечу как TODO: Наводится прицел с лагами
    let is_aiming = player.is_aiming().unwrap_or(false);
    let aim_dir = if is_aiming { Some(vec3_to_net(forward)) } else { None };

    let movement_mode = player.get_movement_mode_byte().unwrap_or(0);

    let tick = NET_SNAPSHOT_TICK.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    Some(NetPlayerSnapshot {
        tick,
        player_id,
        position: vec3_to_net(position),
        forward: vec3_to_net(forward),
        health,
        is_dead,
        state_code,
        car_wrapper_state,
        ctrl_style_mask,
        sub45c_state,
        in_vehicle,
        is_aiming,
        aim_dir,
        is_moving: false,
        movement_mode,
    })
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

        let prev_in_vehicle = previous.in_vehicle.unwrap_or(false);
        let curr_in_vehicle = current.in_vehicle.unwrap_or(false);

        if prev_in_vehicle || curr_in_vehicle {
            // Сбрасываем локальный пеший movement-state.
            self.moving = false;
            self.still_ticks = 0;
        } else {
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
                    } else if dist >= MOVE_START_M {
                        if !self.moving {
                            player_events::push(PlayerEvent::MovementStarted { pos: new_pos });
                        }
                        self.moving = true;
                        self.still_ticks = 0;
                    } else if self.moving && dist < MOVE_STOP_M {
                        self.still_ticks += 1;
                        if self.still_ticks >= STOP_TICKS_REQUIRED {
                            self.moving = false;
                            self.still_ticks = 0;
                            player_events::push(PlayerEvent::MovementStopped { pos: new_pos });
                        }
                    } else if self.moving {
                        self.still_ticks = 0;
                    }
                }
                _ => {
                    self.moving = false;
                    self.still_ticks = 0;
                }
            }
        }

        self.last_snapshot = Some(current);
    }
}
