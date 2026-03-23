//! VehicleTracker — authoritative polling детектор входа/выхода из машины.
//!
//! Почему нужен отдельный трекер, если у нас уже есть HUMAN messages?
//! - Message hook даёт "попытку/фазу" transition
//! - Polling по `owner (+0x80)` даёт реальный runtime-факт
//!
//! Для мультиплеера именно polling нужен как "истинный" источник состояния.

use std::sync::{Mutex, OnceLock};

use common::logger;
use sdk::game::Player;

use crate::{
    player_events::{self, PlayerEvent},
    state::{self, GameSessionState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VehicleState {
    OnFoot,
    InVehicle(usize),
}

#[derive(Debug)]
struct VehicleTracker {
    last_state: Option<VehicleState>,
}

static TRACKER: OnceLock<Mutex<VehicleTracker>> = OnceLock::new();

fn tracker() -> &'static Mutex<VehicleTracker> {
    TRACKER.get_or_init(|| Mutex::new(VehicleTracker { last_state: None }))
}

/// Инициализация трекера.
pub fn init() {
    let _ = tracker();
}

/// Сбросить внутреннее состояние.
///
/// Вызывается когда мы не в InGame или player не готов.
fn reset() {
    if let Ok(mut t) = tracker().lock() {
        t.last_state = None;
    }
}

/// Вызывается на game thread каждый tick.
pub fn update_main_thread() {
    match state::get() {
        GameSessionState::InGame => {}
        GameSessionState::Paused => {
            // На паузе не обновляем, но сохраняем last_state —
            // чтобы после unpause не было ложного VehicleEntered/VehicleLeft.
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

    let current = match player.get_vehicle_ptr() {
        Some(ptr) if ptr != 0 => VehicleState::InVehicle(ptr),
        _ => VehicleState::OnFoot,
    };

    let mut guard = match tracker().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[vehicle-tracker] mutex poisoned");
            return;
        }
    };

    let previous = guard.last_state;

    // Первый тик — просто запоминаем, ничего не эмитим.
    let Some(prev) = previous else {
        guard.last_state = Some(current);
        return;
    };

    if prev == current {
        return;
    }

    match (prev, current) {
        (VehicleState::OnFoot, VehicleState::InVehicle(ptr)) => {
            player_events::push(PlayerEvent::VehicleEntered { vehicle_ptr: ptr });
        }
        (VehicleState::InVehicle(_), VehicleState::OnFoot) => {
            player_events::push(PlayerEvent::VehicleLeft);
        }
        (VehicleState::InVehicle(old_ptr), VehicleState::InVehicle(new_ptr)) => {
            if old_ptr != new_ptr {
                // Редкий случай: сразу пересели в другой транспорт.
                player_events::push(PlayerEvent::VehicleLeft);
                player_events::push(PlayerEvent::VehicleEntered {
                    vehicle_ptr: new_ptr,
                });
            }
        }
        _ => {}
    }

    guard.last_state = Some(current);
}
