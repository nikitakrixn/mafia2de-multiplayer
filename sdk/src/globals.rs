use std::ffi::{c_void};

use crate::game::CGame;
use crate::physical_processor::CPhysicalProcessor;

/// Адреса глобальных указателей в памяти игры
pub mod addresses {
    /// Адрес глобального указателя на C_Game
    pub const C_GAME_ADDRESS: u64 = 0x141CAF778;
    
    /// Адрес указателя на физический процессор (из анализа)
    pub const PHYSICAL_PROCESSOR_ADDR: u64 = 0x141CA1F68;
    
    /// Адрес глобального списка объектов
    pub const OBJECT_LIST_ADDR: u64 = 0x141CADDC0;
    
    /// Адрес счетчика объектов в списке
    pub const OBJECT_LIST_COUNT_ADDR: u64 = 0x141CADDAC;
}

/// Получает глобальный экземпляр игры
pub fn get_game_instance() -> Option<&'static mut CGame> {
    unsafe {
        let ptr = *(addresses::C_GAME_ADDRESS as *const *mut CGame);
        if ptr.is_null() {
            None
        } else {
            Some(&mut *ptr)
        }
    }
}

/// Получает глобальный экземпляр физического процессора
pub fn get_physical_processor() -> Option<&'static mut CPhysicalProcessor> {
    unsafe {
        let ptr = *(addresses::PHYSICAL_PROCESSOR_ADDR as *const *mut CPhysicalProcessor);
        if ptr.is_null() {
            None
        } else {
            Some(&mut *ptr)
        }
    }
}

/// Получает указатель на глобальный экземпляр игры как сырой указатель
pub fn get_game_instance_ptr() -> *mut c_void {
    unsafe {
        *(addresses::C_GAME_ADDRESS as *const *mut c_void)
    }
}

/// Получает указатель на глобальный физический процессор как сырой указатель
pub fn get_physical_processor_ptr() -> *mut c_void {
    unsafe {
        *(addresses::PHYSICAL_PROCESSOR_ADDR as *const *mut c_void)
    }
}

/// Проверяет, инициализирована ли игра
pub fn is_game_initialized() -> bool {
    get_game_instance().is_some()
}

/// Безопасно получает экземпляр игры с проверкой
pub fn get_game() -> Result<&'static mut CGame, &'static str> {
    match get_game_instance() {
        Some(game) => Ok(game),
        None => Err("Game instance is not initialized"),
    }
} 