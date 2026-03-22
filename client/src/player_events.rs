//! Очередь высокоуровневых событий локального игрока.
//!
//! Сюда складываются события из:
//! - Human Message hook
//! - PlayerTracker
//!
//! Потом они централизованно логируются и в будущем могут
//! отправляться в multiplayer transport слой.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use common::logger;
use sdk::game::player::Vec3;

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    EnterVehicle,
    EnterVehicleDone,
    LeaveVehicle,
    LeaveVehicleDone,

    Damage,
    Death,
    AnimNotify,

    Shot,
    WeaponSelect,
    WeaponHide,

    MoneyChanged {
        old_cents: i64,
        new_cents: i64,
        delta_cents: i64,
    },

    MovementStarted {
        pos: Vec3,
    },

    MovementStopped {
        pos: Vec3,
    },

    Teleported {
        from: Vec3,
        to: Vec3,
        distance: f32,
    },

    ControlsLockedChanged {
        locked: bool,
    },

    ControlStyleChanged {
        style: String,
    },
}

static PLAYER_EVENT_QUEUE: OnceLock<Mutex<VecDeque<PlayerEvent>>> = OnceLock::new();

fn queue() -> &'static Mutex<VecDeque<PlayerEvent>> {
    PLAYER_EVENT_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn init() {
    let _ = queue();
}

pub fn push(event: PlayerEvent) {
    match queue().lock() {
        Ok(mut q) => q.push_back(event),
        Err(_) => logger::error("[player-events] mutex poisoned in push"),
    }
}

pub fn process_pending() {
    let drained: Vec<_> = match queue().lock() {
        Ok(mut q) => q.drain(..).collect(),
        Err(_) => {
            logger::error("[player-events] mutex poisoned");
            return;
        }
    };

    for ev in &drained {
        log_event(ev);
    }
}

fn log_event(ev: &PlayerEvent) {
    match ev {
        PlayerEvent::EnterVehicle => {
            logger::info("[player-event] EnterVehicle");
        }
        PlayerEvent::EnterVehicleDone => {
            logger::info("[player-event] EnterVehicleDone");
        }
        PlayerEvent::LeaveVehicle => {
            logger::info("[player-event] LeaveVehicle");
        }
        PlayerEvent::LeaveVehicleDone => {
            logger::info("[player-event] LeaveVehicleDone");
        }

        PlayerEvent::Damage => {
            logger::info("[player-event] Damage");
        }
        PlayerEvent::Death => {
            logger::info("[player-event] Death");
        }
        PlayerEvent::AnimNotify => {
            logger::debug("[player-event] AnimNotify");
        }

        PlayerEvent::Shot => {
            logger::info("[player-event] Shot");
        }
        PlayerEvent::WeaponSelect => {
            logger::info("[player-event] WeaponSelect");
        }
        PlayerEvent::WeaponHide => {
            logger::info("[player-event] WeaponHide");
        }

        PlayerEvent::MoneyChanged {
            old_cents,
            new_cents,
            delta_cents,
        } => {
            logger::info(&format!(
                "[player-event] MoneyChanged: {} -> {} (delta: {})",
                format_money(*old_cents),
                format_money(*new_cents),
                format_money_delta(*delta_cents),
            ));
        }

        PlayerEvent::MovementStarted { pos } => {
            logger::info(&format!("[player-event] MovementStarted at {}", pos));
        }

        PlayerEvent::MovementStopped { pos } => {
            logger::info(&format!("[player-event] MovementStopped at {}", pos));
        }

        PlayerEvent::Teleported { from, to, distance } => {
            logger::info(&format!(
                "[player-event] Teleported: {} -> {} (dist {:.2})",
                from, to, distance
            ));
        }

        PlayerEvent::ControlsLockedChanged { locked } => {
            logger::info(&format!(
                "[player-event] ControlsLockedChanged -> {}",
                locked
            ));
        }

        PlayerEvent::ControlStyleChanged { style } => {
            logger::info(&format!(
                "[player-event] ControlStyleChanged -> \"{}\"",
                style
            ));
        }
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
