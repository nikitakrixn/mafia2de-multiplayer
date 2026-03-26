//! Noclip / Fly mode для Mafia II: DE.
//!
//! Базовая безопасная реализация:
//! - вся игровая логика выполняется на main thread
//! - toggle по F4
//! - каждый тик удерживаем позицию через `player.set_position()`
//! - при включении включаем неуязвимость и блокируем обычное управление
//! - при выключении восстанавливаем исходные флаги

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use common::logger;
use sdk::game::Player;
use sdk::types::Vec3;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

use crate::state::GameSessionState;

const VK_F4: i32 = 0x73;
const VK_SHIFT: i32 = 0x10;
const VK_CTRL: i32 = 0x11;
const VK_SPACE: i32 = 0x20;
const VK_W: i32 = 0x57;
const VK_A: i32 = 0x41;
const VK_S: i32 = 0x53;
const VK_D: i32 = 0x44;

/// Скорость полёта по горизонтали.
const FLY_SPEED: f32 = 1.5;

/// Быстрый режим при зажатом Shift.
const FLY_SPEED_FAST: f32 = 5.0;

/// Скорость вверх/вниз.
const FLY_SPEED_VERTICAL: f32 = 1.2;

static NOCLIP_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Можно безопасно запросить toggle из любого потока.
static TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Защита от автоповтора клавиши F4.
static TOGGLE_WAS_DOWN: AtomicBool = AtomicBool::new(false);

static WAS_INVULNERABLE: AtomicBool = AtomicBool::new(false);
static WAS_CONTROLS_LOCKED: AtomicBool = AtomicBool::new(false);

static NOCLIP_POS: OnceLock<Mutex<Vec3>> = OnceLock::new();

fn pos_cell() -> &'static Mutex<Vec3> {
    NOCLIP_POS.get_or_init(|| {
        Mutex::new(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
    })
}

fn read_pos_snapshot() -> Vec3 {
    match pos_cell().lock() {
        Ok(g) => *g,
        Err(_) => Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    }
}

pub fn is_active() -> bool {
    NOCLIP_ACTIVE.load(Ordering::Acquire)
}

/// Публичная функция для переключения noclip (можно вызывать из input).
pub fn toggle() {
    request_toggle();
}

/// Можно вызывать из input thread / debug command / console.
/// Сам toggle всё равно выполнится на main thread.
pub fn request_toggle() {
    TOGGLE_REQUESTED.store(true, Ordering::Release);
}

/// Вызывать из runtime shutdown.
pub fn shutdown() {
    if !is_active() {
        return;
    }

    if let Some(player) = Player::get_active() {
        disable_internal(&player, "[noclip] shutdown -> disabled");
    } else {
        force_off("[noclip] shutdown -> player unavailable, forcing OFF");
    }
}

/// Главный main-thread тик noclip.
pub fn tick_main_thread() {
    poll_toggle_hotkey();

    if TOGGLE_REQUESTED.swap(false, Ordering::AcqRel) {
        toggle_main_thread();
    }

    if !is_active() {
        return;
    }

    match crate::state::get() {
        GameSessionState::InGame => {}
        GameSessionState::Paused => return,
        _ => {
            disable_current_or_force("[noclip] session changed -> disabled");
            return;
        }
    }

    let Some(player) = Player::get_active() else {
        force_off("[noclip] active player lost -> forcing OFF");
        return;
    };

    if !player.is_ready() {
        return;
    }

    let (forward, right) = movement_basis(&player);

    let fast = is_key_down(VK_SHIFT);
    let move_speed = if fast { FLY_SPEED_FAST } else { FLY_SPEED };
    let vertical_speed = if fast { FLY_SPEED_FAST } else { FLY_SPEED_VERTICAL };

    let mut delta = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    if is_key_down(VK_W) {
        delta.x += forward.x * move_speed;
        delta.y += forward.y * move_speed;
    }
    if is_key_down(VK_S) {
        delta.x -= forward.x * move_speed;
        delta.y -= forward.y * move_speed;
    }
    if is_key_down(VK_D) {
        delta.x += right.x * move_speed;
        delta.y += right.y * move_speed;
    }
    if is_key_down(VK_A) {
        delta.x -= right.x * move_speed;
        delta.y -= right.y * move_speed;
    }

    if is_key_down(VK_SPACE) {
        delta.z += vertical_speed;
    }
    if is_key_down(VK_CTRL) {
        delta.z -= vertical_speed;
    }

    let target = {
        let mut guard = match pos_cell().lock() {
            Ok(g) => g,
            Err(_) => {
                logger::error("[noclip] pos mutex poisoned");
                return;
            }
        };

        guard.x += delta.x;
        guard.y += delta.y;
        guard.z += delta.z;

        *guard
    };

    // Важно: даже если delta == 0, всё равно держим позицию каждый тик,
    // чтобы игра не пыталась вернуть актёра назад через обычный sync/physics path.
    if !player.set_position(&target) {
        disable_internal(&player, "[noclip] set_position failed -> disabled");
    }
}

fn toggle_main_thread() {
    let Some(player) = Player::get_active() else {
        logger::warn("[noclip] player not found");
        return;
    };

    if !player.is_ready() {
        logger::warn("[noclip] player not ready");
        return;
    }

    if is_active() {
        disable_internal(&player, "[noclip] disabled");
    } else {
        enable_internal(&player);
    }
}

fn enable_internal(player: &Player) {
    let pos = player.get_position().unwrap_or(Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });

    match pos_cell().lock() {
        Ok(mut g) => *g = pos,
        Err(_) => {
            logger::error("[noclip] pos mutex poisoned on enable");
            return;
        }
    }

    WAS_INVULNERABLE.store(
        player.is_invulnerable().unwrap_or(false),
        Ordering::Release,
    );
    WAS_CONTROLS_LOCKED.store(
        player.are_controls_locked().unwrap_or(false),
        Ordering::Release,
    );

    let _ = player.set_invulnerable(true);

    if !WAS_CONTROLS_LOCKED.load(Ordering::Acquire) {
        let _ = player.lock_controls(true);
    }

    NOCLIP_ACTIVE.store(true, Ordering::Release);

    // Один раз сразу зафиксируем позицию.
    let _ = player.set_position(&pos);

    logger::info(&format!("[noclip] enabled at {}", pos));
}

fn disable_internal(player: &Player, message: &str) {
    if !NOCLIP_ACTIVE.swap(false, Ordering::AcqRel) {
        return;
    }

    if !WAS_INVULNERABLE.load(Ordering::Acquire) {
        let _ = player.set_invulnerable(false);
    }

    if !WAS_CONTROLS_LOCKED.load(Ordering::Acquire) {
        let _ = player.lock_controls(false);
    }

    let pos = read_pos_snapshot();
    let _ = player.set_position(&pos);

    logger::info(&format!("{message} at {}", pos));
}

fn disable_current_or_force(message: &str) {
    if let Some(player) = Player::get_active() {
        disable_internal(&player, message);
    } else {
        force_off(message);
    }
}

fn force_off(message: &str) {
    if NOCLIP_ACTIVE.swap(false, Ordering::AcqRel) {
        logger::warn(message);
    }
}

fn poll_toggle_hotkey() {
    let down = is_key_down(VK_F4);
    let was_down = TOGGLE_WAS_DOWN.swap(down, Ordering::AcqRel);

    if down && !was_down {
        request_toggle();
    }
}

fn is_key_down(vk: i32) -> bool {
    unsafe { ((GetAsyncKeyState(vk) as u16) & 0x8000) != 0 }
}

fn normalize_xy(v: Vec3, fallback: Vec3) -> Vec3 {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len > 0.001 {
        Vec3 {
            x: v.x / len,
            y: v.y / len,
            z: 0.0,
        }
    } else {
        fallback
    }
}

fn movement_basis(player: &Player) -> (Vec3, Vec3) {
    let forward_raw = player.get_forward().unwrap_or(Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    });

    let right_raw = player.get_right().unwrap_or(Vec3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    });

    let forward = normalize_xy(
        forward_raw,
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    );

    let right = normalize_xy(
        right_raw,
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
    );

    (forward, right)
}