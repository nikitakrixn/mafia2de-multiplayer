//! Высокоуровневый API для работы с игрой.
//!
//! ## Порядок инициализации движка
//!
//! 1. Модуль загружен -> `base()` работает
//! 2. GameManager создан -> `is_game_initialized()` = true
//! 3. Player создан -> `Player::get()` возвращает Some
//! 4. Lua VM готова -> `lua::is_ready()` = true

pub mod c_sys_input;
pub mod callbacks;
pub mod camera;
pub mod car;
pub mod entity;
pub mod entity_ref;
pub mod entity_types;
pub mod game_input_module;
mod hash;
pub mod lua;
pub mod npc;
pub mod player;
pub mod police;
pub mod render;
pub mod script_entity;
pub mod sds;
pub mod world;
pub mod manager;
pub mod application;
pub mod mission;

pub use c_sys_input::SysInput;
pub use entity_types::{EntityMessageType, EntityType, FactoryType};
pub use player::Player;
pub use manager::Game;
pub use application::Application;
pub use game_input_module::GameInputModule;
pub use mission::Mission;

use crate::{addresses, memory};
use common::logger;
use std::sync::LazyLock;

/// Кэш базового адреса модуля игры.
///
/// `LazyLock` — инициализируется при первом обращении,
/// потокобезопасно, без boilerplate.
static GAME_BASE: LazyLock<usize> = LazyLock::new(|| {
    memory::get_module_base(addresses::GAME_MODULE)
        .expect("Модуль игры не найден")
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

/// Проверяет, был ли базовый адрес уже разрешён (без форсинга).
///
/// Полезно для диагностики на ранних стадиях инициализации,
/// когда форсить LazyLock небезопасно.
pub fn is_base_resolved() -> bool {
    LazyLock::get(&GAME_BASE).is_some()
}

/// Логирует базовый адрес и размер модуля игры.
pub fn log_module_info() {
    if !is_base_resolved() {
        logger::warn("log_module_info вызван до разрешения базового адреса");
    }
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

/// Информация о модуле игры (кешированная).
pub fn module_info() -> Option<memory::ModuleInfo> {
    static INFO: std::sync::LazyLock<Option<memory::ModuleInfo>> =
        std::sync::LazyLock::new(|| memory::get_module_info(addresses::GAME_MODULE));
    *INFO
}