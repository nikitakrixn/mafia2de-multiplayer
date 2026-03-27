//! Высокоуровневый API для работы с игрой.
//!
//! ## Порядок инициализации движка
//!
//! 1. Модуль загружен -> `base()` работает
//! 2. GameManager создан -> `is_game_initialized()` = true
//! 3. Player создан -> `Player::get()` возвращает Some
//! 4. Lua VM готова -> `lua::is_ready()` = true

pub mod callbacks;
pub mod camera;
pub mod car;
pub mod entity;
pub mod entity_ref;
pub mod entity_types;
mod hash;
pub mod lua;
pub mod npc;
pub mod player;
pub mod police;
pub mod render;
pub mod script_entity;
pub mod sds;
pub mod world;

pub use entity_types::{EntityMessageType, EntityType, FactoryType};
pub use player::Player;

use crate::{addresses, memory};
use common::logger;
use std::sync::LazyLock;

/// Кэш базового адреса модуля игры.
///
/// `LazyLock` (Rust 1.80+) — инициализируется при первом обращении,
/// потокобезопасно, без boilerplate.
static GAME_BASE: LazyLock<usize> = LazyLock::new(|| {
    memory::get_module_base(addresses::GAME_MODULE)
        .expect("Модуль игры не найден — DLL не инжектирована в процесс игры")
});

/// Базовый адрес модуля игры.
///
/// Все RVA из [`addresses`] прибавляются к этому значению.
///
/// # Паника
///
/// Паникует если модуль не найден — без него SDK бесполезен.
#[inline]
pub fn base() -> usize {
    *GAME_BASE
}

/// Проверяет что GameManager проинициализирован.
///
/// Пока `false` — Player, Lua, Entity system ещё не готовы.
pub fn is_game_initialized() -> bool {
    unsafe { memory::read_validated_ptr(base() + addresses::globals::GAME_MANAGER).is_some() }
}

/// Логирует базовый адрес и размер модуля игры.
pub fn log_module_info() {
    match memory::get_module_info(addresses::GAME_MODULE) {
        Some(info) => logger::info(&format!(
            "Модуль игры: base=0x{:X}, size=0x{:X} ({:.1} МБ)",
            info.base,
            info.size,
            info.size as f64 / (1024.0 * 1024.0),
        )),
        None => logger::error("Не удалось получить информацию о модуле игры!"),
    }
}
