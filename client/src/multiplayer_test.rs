//! Локальные тесты мультиплеера без реального сервера.
//!
//! Назначение:
//! - проверить remote player pipeline
//! - проверить binding на Joe / Henry
//! - проверить применение snapshot'ов
//! - не ждать transport v1
//!
//! Использование:
//! - F5 — заспавнить mock remote player рядом с локальным
//! - F6 — сдвинуть mock remote player по кругу
//! - F7 — отправить mock remote event (Shot)
//! - F8 — удалить mock remote player
//!
//! ВАЖНО:
//! Работает только если multiplayer session уже активна
//! (через меню F2 -> Подключиться).

use std::sync::{Mutex, OnceLock};

use common::logger;
use protocol::{NetPlayerEvent, NetPlayerSnapshot, NetVec3, PlayerId};
use sdk::game::Player;

/// ID мок-игрока в сессии.
const MOCK_REMOTE_ID: PlayerId = 2;

/// Имя мок-игрока (привяжется к Joe или Henry).
const MOCK_REMOTE_NAME: &str = "RemoteJoe";

#[derive(Debug, Clone, Copy)]
struct MockRemoteState {
    spawned: bool,
    phase: f32,
    base_x: f32,
    base_y: f32,
    base_z: f32,
}

static MOCK: OnceLock<Mutex<MockRemoteState>> = OnceLock::new();

fn mock() -> &'static Mutex<MockRemoteState> {
    MOCK.get_or_init(|| {
        Mutex::new(MockRemoteState {
            spawned: false,
            phase: 0.0,
            base_x: 0.0,
            base_y: 0.0,
            base_z: 0.0,
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
        logger::info("[mp-test] Mock remote уже заспавнен (F8 для удаления)");
        return;
    }

    guard.spawned = true;
    guard.phase = 0.0;
    guard.base_x = pos.x + 4.0;
    guard.base_y = pos.y + 2.0;
    guard.base_z = pos.z;

    let base_x = guard.base_x;
    let base_y = guard.base_y;
    let base_z = guard.base_z;
    drop(guard);

    crate::network::inject_player_spawn(MOCK_REMOTE_ID, MOCK_REMOTE_NAME.to_string());

    crate::network::inject_remote_snapshot(NetPlayerSnapshot {
        tick: 1,
        player_id: MOCK_REMOTE_ID,
        position: NetVec3 { x: base_x, y: base_y, z: base_z },
        forward: NetVec3 { x: 1.0, y: 0.0, z: 0.0 },
        health: 1000.0,
        is_dead: false,
        state_code_430: 0,
        state_flags_3d8: 0,
        state_flags_490: 0,
        sub45c_state: 0,
        in_vehicle: false,
    });

    logger::info(&format!(
        "[mp-test] Mock remote spawned at ({:.2}, {:.2}, {:.2})",
        base_x, base_y, base_z
    ));
}

/// Сдвинуть mock remote player по кругу вокруг базовой точки.
pub fn step_mock_remote() {
    if !crate::network::is_connected() {
        logger::warn("[mp-test] Session не активна");
        return;
    }

    let mut guard = match mock().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[mp-test] mutex poisoned in step_mock_remote");
            return;
        }
    };

    if !guard.spawned {
        logger::warn("[mp-test] Mock remote ещё не заспавнен (F5)");
        return;
    }

    guard.phase += 0.35;

    let radius = 3.0f32;
    let px = guard.base_x + guard.phase.cos() * radius;
    let py = guard.base_y + guard.phase.sin() * radius;
    let pz = guard.base_z;

    // Forward — касательная к окружности
    let fx = -guard.phase.sin();
    let fy = guard.phase.cos();

    let tick = (guard.phase * 1000.0) as u64;
    drop(guard);

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

/// Отправить mock remote event (Shot).
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
        logger::warn("[mp-test] Mock remote ещё не заспавнен (F5)");
        return;
    }
    drop(guard);

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
    drop(guard);

    crate::network::inject_player_despawn(MOCK_REMOTE_ID);
    logger::info("[mp-test] Mock remote despawned");
}
