//! Высокоуровневый API для работы с игрой.
//!
//! Этот модуль — основной интерфейс SDK для клиента.
//! Скрывает за собой цепочки указателей, вызовы движка
//! и прочую низкоуровневую механику.

pub mod callbacks;
pub mod player;
pub mod lua;
pub mod render;

pub use player::Player;

use std::sync::OnceLock;
use crate::{addresses, memory};
use common::logger;

/// Кэш базового адреса модуля игры.
/// Инициализируется один раз при первом обращении.
static GAME_BASE: OnceLock<usize> = OnceLock::new();

/// Базовый адрес модуля игры (кэшируется).
///
/// Все RVA из addresses прибавляются к этому адресу.
///
/// # Паника
///
/// Паникует если DLL загружена не в процесс игры.
/// Это ожидаемое поведение — без модуля игры SDK бесполезен.
pub(crate) fn base() -> usize {
    *GAME_BASE.get_or_init(|| {
        memory::get_module_base(addresses::GAME_MODULE)
            .expect("Модуль игры не найден — DLL не инжектирована?")
    })
}

/// Проверяет что GameManager проинициализирован.
///
/// GameManager появляется не сразу — движку нужно время
/// на загрузку core-систем. До этого момента обращаться
/// к Player, Lua и прочим подсистемам бессмысленно.
pub fn is_game_initialized() -> bool {
    unsafe { memory::read_ptr(base() + addresses::globals::GAME_MANAGER).is_some() }
}

/// Логирует базовый адрес и размер модуля игры.
/// Полезно при старте клиента — сразу видно что инжект сработал.
pub fn log_module_info() {
    match memory::get_module_info(addresses::GAME_MODULE) {
        Some(info) => logger::info(&format!(
            "Модуль игры: base=0x{:X}, size=0x{:X} ({:.1} МБ)",
            info.base, info.size, info.size as f64 / (1024.0 * 1024.0),
        )),
        None => logger::error("Не удалось получить информацию о модуле игры!"),
    }
}