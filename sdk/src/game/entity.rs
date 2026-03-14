//! Поиск entity по имени через native path
//!
//! Использует M2DE_EntityManager_FindByName напрямую.
//! Возвращает Script Wrapper pointer, из которого можно
//! получить native entity через wrapper+0x10.

use crate::addresses;
use crate::addresses::fields::script_wrapper;
use crate::memory;
use super::base;
use common::logger;

/// FNV-1a хеш, используемый движком для entity names.
pub fn fnv1a_hash(name: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF29CE484222325;
    for &byte in name {
        hash = (byte as u64) ^ hash.wrapping_mul(0x100000001B3);
    }
    hash
}

/// Найти entity по имени через native path.
/// Возвращает указатель на Script Wrapper (refcounted).
///
/// ⚠️ Wrapper != native entity. Native = *(wrapper + 0x10).
///
/// Пример:
/// ```ignore
/// let wrapper = find_entity_by_name("Joe")?;
/// let native = unsafe { memory::read_ptr(wrapper + 0x10)? };
/// // native — это C_Human* для NPC
/// ```
pub fn find_entity_by_name(name: &str) -> Option<usize> {
    let base = base();

    // Получить ScriptWrapperManager
    let mgr = unsafe {
        memory::read_ptr(base + addresses::globals::SCRIPT_WRAPPER_MANAGER)?
    };

    // Вызвать M2DE_EntityManager_FindByName(mgr, name)
    let c_name = std::ffi::CString::new(name).ok()?;

    type FindByNameFn = unsafe extern "C" fn(usize, *const i8) -> usize;
    let func: FindByNameFn = unsafe {
        memory::fn_at(base + addresses::functions::entity_manager::FIND_BY_NAME)
    };

    let wrapper = unsafe { func(mgr, c_name.as_ptr()) };

    if wrapper == 0 {
        logger::debug(&format!("[entity] '{}' не найден", name));
        None
    } else {
        logger::debug(&format!(
            "[entity] '{}' → wrapper=0x{:X}",
            name, wrapper
        ));
        Some(wrapper)
    }
}

/// Найти native entity pointer по имени.
/// Возвращает C_Human*/C_Car*/etc. напрямую.
pub fn find_native_entity(name: &str) -> Option<usize> {
    let wrapper = find_entity_by_name(name)?;
    unsafe { memory::read_ptr(wrapper + script_wrapper::NATIVE) }
}

/// Получить тип entity по имени.
/// 0x0E = NPC human, 0x10 = player, 0x12 = physics, etc.
pub fn get_entity_type(name: &str) -> Option<u8> {
    let native = find_native_entity(name)?;
    unsafe { memory::read_value::<u8>(native + addresses::fields::player::ENTITY_TYPE) }
}

/// Проверить что entity жива (для C_Human).
pub fn is_entity_alive(name: &str) -> Option<bool> {
    let native = find_native_entity(name)?;
    unsafe {
        memory::read_value::<u8>(native + addresses::fields::player::IS_DEAD)
            .map(|v| v == 0)
    }
}

/// Прочитать здоровье entity (для C_Human).
pub fn get_entity_health(name: &str) -> Option<f32> {
    let native = find_native_entity(name)?;
    unsafe { memory::read_value::<f32>(native + addresses::fields::player::CURRENT_HEALTH) }
}