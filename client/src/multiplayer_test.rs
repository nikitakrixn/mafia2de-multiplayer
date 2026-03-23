//! Локальные multiplayer-тесты без реального сервера.
//!
//! ВАЖНО:
//! Это не transport слой, а только удобная проверка:
//! - remote player binding
//! - remote snapshot apply
//! - remote event apply
//!
//! Горячие клавиши:
//! - F5 — spawn mock remote player
//! - F6 — включить/выключить auto-move mock remote
//! - F7 — отправить mock remote event (Shot)
//! - F8 — despawn mock remote player

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use common::logger;
use protocol::{NetPlayerEvent, NetPlayerSnapshot, NetVec3, PlayerId};
use sdk::game::Player;

/// ID мок-игрока.
const MOCK_REMOTE_ID: PlayerId = 2;

/// Имя мок-игрока.
const MOCK_REMOTE_NAME: &str = "RemoteJoe";

/// Интервал автодвижения мок-игрока.
const AUTO_STEP_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
struct MockRemoteState {
    spawned: bool,
    auto_move: bool,
    phase: f32,
    base_x: f32,
    base_y: f32,
    base_z: f32,
    last_step: Instant,
}

static MOCK: OnceLock<Mutex<MockRemoteState>> = OnceLock::new();

fn mock() -> &'static Mutex<MockRemoteState> {
    MOCK.get_or_init(|| {
        Mutex::new(MockRemoteState {
            spawned: false,
            auto_move: false,
            phase: 0.0,
            base_x: 0.0,
            base_y: 0.0,
            base_z: 0.0,
            last_step: Instant::now(),
        })
    })
}

/// Заспавнить mock remote player рядом с локальным игроком.
pub fn spawn_mock_remote() {
    if !crate::network::is_connected() {
        logger::warn("[mp-test] Сначала открой multiplayer session через F2");
        return;
    }

    let Some(player) = Player::get_active() else {
        logger::warn("[mp-test] Локальный игрок не найден");
        return;
    };

    let Some(pos) = player.get_position() else {
        logger::warn("[mp-test] Не удалось получить позицию локального игрока");
        return;
    };

    let mut guard = match mock().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[mp-test] mutex poisoned in spawn_mock_remote");
            return;
        }
    };

    if guard.spawned {
        logger::info("[mp-test] Mock remote уже заспавнен");
        return;
    }

    guard.spawned = true;
    guard.auto_move = false;
    guard.phase = 0.0;
    guard.base_x = pos.x + 4.0;
    guard.base_y = pos.y + 2.0;
    guard.base_z = pos.z;
    guard.last_step = Instant::now();

    crate::network::inject_player_spawn(MOCK_REMOTE_ID, MOCK_REMOTE_NAME.to_string());

    let snapshot = NetPlayerSnapshot {
        tick: 1,
        player_id: MOCK_REMOTE_ID,
        position: NetVec3 {
            x: guard.base_x,
            y: guard.base_y,
            z: guard.base_z,
        },
        forward: NetVec3 { x: 1.0, y: 0.0, z: 0.0 },
        health: 1000.0,
        is_dead: false,
        state_code_430: 0,
        state_flags_3d8: 0,
        state_flags_490: 0,
        sub45c_state: 0,
        in_vehicle: false,
    };

    logger::info(&format!(
        "[mp-test] Mock remote spawned at ({:.2}, {:.2}, {:.2})",
        guard.base_x, guard.base_y, guard.base_z
    ));

    drop(guard);
    crate::network::inject_remote_snapshot(snapshot);
}

/// Включить/выключить автоматическое движение mock remote player.
pub fn toggle_mock_auto_move() {
    if !crate::network::is_connected() {
        logger::warn("[mp-test] Session не активна");
        return;
    }

    let mut guard = match mock().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[mp-test] mutex poisoned in toggle_mock_auto_move");
            return;
        }
    };

    if !guard.spawned {
        logger::warn("[mp-test] Mock remote ещё не заспавнен (F5)");
        return;
    }

    guard.auto_move = !guard.auto_move;
    guard.last_step = Instant::now();

    logger::info(&format!(
        "[mp-test] Mock remote auto-move: {}",
        if guard.auto_move { "ON" } else { "OFF" }
    ));
}

/// Сдвинуть mock remote player по кругу на один шаг (внутренняя функция).
fn step_mock_remote_inner(guard: &mut MockRemoteState) {
    guard.phase += 0.35;

    let radius = 3.0f32;
    let px = guard.base_x + guard.phase.cos() * radius;
    let py = guard.base_y + guard.phase.sin() * radius;
    let pz = guard.base_z;

    // Forward — касательная к окружности
    let fx = -guard.phase.sin();
    let fy = guard.phase.cos();
    let tick = (guard.phase * 1000.0) as u64;

    crate::network::inject_remote_snapshot(NetPlayerSnapshot {
        tick,
        player_id: MOCK_REMOTE_ID,
        position: NetVec3 { x: px, y: py, z: pz },
        forward: NetVec3 { x: fx, y: fy, z: 0.0 },
        health: 1000.0,
        is_dead: false,
        state_code_430: 0,
        state_flags_3d8: 0,
        state_flags_490: 0,
        sub45c_state: 0,
        in_vehicle: false,
    });

    logger::debug(&format!(
        "[mp-test] Mock remote moved to ({:.2}, {:.2}, {:.2})",
        px, py, pz
    ));
}

/// Вызывается на game thread каждый tick.
pub fn update_main_thread() {
    if !crate::network::is_connected() {
        return;
    }

    let mut guard = match mock().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[mp-test] mutex poisoned in update_main_thread");
            return;
        }
    };

    if !guard.spawned || !guard.auto_move {
        return;
    }

    if guard.last_step.elapsed() < AUTO_STEP_INTERVAL {
        return;
    }

    guard.last_step = Instant::now();
    step_mock_remote_inner(&mut guard);
}

/// Отправить mock remote event.
pub fn send_mock_remote_event() {
    if !crate::network::is_connected() {
        logger::warn("[mp-test] Session не активна");
        return;
    }

    let guard = match mock().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[mp-test] mutex poisoned in send_mock_remote_event");
            return;
        }
    };

    if !guard.spawned {
        logger::warn("[mp-test] Mock remote ещё не заспавнен");
        return;
    }

    crate::network::inject_remote_event(MOCK_REMOTE_ID, NetPlayerEvent::Shot);
    logger::info("[mp-test] Mock remote event: Shot");
}

/// Удалить mock remote player.
pub fn despawn_mock_remote() {
    if !crate::network::is_connected() {
        logger::warn("[mp-test] Session не активна");
        return;
    }

    let mut guard = match mock().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[mp-test] mutex poisoned in despawn_mock_remote");
            return;
        }
    };

    if !guard.spawned {
        logger::info("[mp-test] Mock remote уже отсутствует");
        return;
    }

    guard.spawned = false;
    guard.auto_move = false;
    crate::network::inject_player_despawn(MOCK_REMOTE_ID);

    logger::info("[mp-test] Mock remote despawned");
}
