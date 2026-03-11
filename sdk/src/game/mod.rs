//! Высокоуровневый API для работы с игрой.

pub mod callbacks;
pub mod player;
pub mod lua;
pub mod render;

pub use player::Player;

use std::sync::OnceLock;
use crate::{addresses, memory};
use common::logger;

static GAME_BASE: OnceLock<usize> = OnceLock::new();

/// Базовый адрес модуля (кэшируется).
///
/// # Panics
///
/// Паникует если DLL не в процессе игры.
pub(crate) fn base() -> usize {
    *GAME_BASE.get_or_init(|| {
        memory::get_module_base(addresses::GAME_MODULE)
            .expect("Game module not found — DLL not injected?")
    })
}

/// Проверяет что GameManager проинициализирован.
/// Используем собственный read_ptr для единообразия.
pub fn is_game_initialized() -> bool {
    unsafe { memory::read_ptr(base() + addresses::globals::GAME_MANAGER).is_some() }
}

/// Логирует информацию о модуле.
pub fn log_module_info() {
    match memory::get_module_info(addresses::GAME_MODULE) {
        Some(info) => logger::info(&format!(
            "Game module: base=0x{:X}, size=0x{:X} ({:.1} MB)",
            info.base, info.size, info.size as f64 / (1024.0 * 1024.0),
        )),
        None => logger::error("Failed to get game module info!"),
    }
}