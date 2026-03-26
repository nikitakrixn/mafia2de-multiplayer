//! Entity system — поиск сущностей, type helpers и alias cache.
//!
//! Движок использует FNV-1 64-bit hash для имён entity.
//! Поиск идёт через `C_ScriptWrapperManager`.
//!
//! ВАЖНО:
//! - `wrapper + 0x10` = native entity pointer
//! - player НЕ ищется через FindByName
//! - native runtime type = factory type byte из `entity + 0x24`

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use super::{base, entity_ref::EntityRef, entity_types::FactoryType};
use crate::addresses;
use crate::addresses::fields::{entity, script_wrapper};
use crate::memory;
use common::logger;

// =============================================================================
//  Entity lookup
// =============================================================================

/// Найти entity по имени через `M2DE_EntityManager_FindByName`.
///
/// Возвращает указатель на `C_ScriptWrapper`.
pub fn find_entity_by_name(name: &str) -> Option<usize> {
    let base = base();

    let mgr =
        unsafe { memory::read_validated_ptr(base + addresses::globals::SCRIPT_WRAPPER_MANAGER)? };

    let c_name = std::ffi::CString::new(name).ok()?;

    let hash = super::hash::fnv1_64(name.as_bytes());
    observe_hashed_name(hash, name);

    type FindByNameFn = unsafe extern "C" fn(usize, *const i8) -> usize;
    let func: FindByNameFn =
        unsafe { memory::fn_at(base + addresses::functions::entity_manager::FIND_BY_NAME) };

    let wrapper = unsafe { func(mgr, c_name.as_ptr()) };

    if wrapper == 0 {
        logger::debug(&format!("[entity] '{}' не найден", name));
        None
    } else {
        logger::debug(&format!("[entity] '{}' -> wrapper=0x{:X}", name, wrapper));

        if let Some(native) =
            unsafe { memory::read_validated_ptr(wrapper + script_wrapper::NATIVE) }
        {
            register_entity_alias(native, name);
        }

        Some(wrapper)
    }
}

pub fn find_native_entity(name: &str) -> Option<usize> {
    let wrapper = find_entity_by_name(name)?;
    unsafe { memory::read_validated_ptr(wrapper + script_wrapper::NATIVE) }
}

pub fn find_entity_ref_by_name(name: &str) -> Option<EntityRef> {
    let native = find_native_entity(name)?;
    EntityRef::from_ptr(native)
}

pub fn get_entity_table_id(name: &str) -> Option<u32> {
    find_entity_ref_by_name(name)?.table_id()
}

pub fn get_entity_factory_type(name: &str) -> Option<u8> {
    find_entity_ref_by_name(name)?.factory_type_byte()
}

pub fn get_entity_factory_type_enum(name: &str) -> Option<FactoryType> {
    find_entity_ref_by_name(name)?.factory_type()
}

pub fn get_entity_health(name: &str) -> Option<f32> {
    find_entity_ref_by_name(name)?.health()
}

pub fn is_entity_alive(name: &str) -> Option<bool> {
    find_entity_ref_by_name(name)?.is_alive()
}

pub fn factory_type_name(ft: u8) -> &'static str {
    FactoryType::from_byte(ft)
        .map(|t| t.display_name())
        .unwrap_or("UNKNOWN")
}

#[inline]
pub fn table_id_factory_type(table_id: u32) -> u8 {
    (table_id & 0xFF) as u8
}

#[inline]
pub fn table_id_instance_id(table_id: u32) -> u32 {
    table_id >> 8
}

pub fn native_factory_type_byte(entity_ptr: usize) -> Option<u8> {
    if entity_ptr == 0 || !memory::is_valid_ptr(entity_ptr) {
        return None;
    }
    unsafe { memory::read::<u8>(entity_ptr + entity::TABLE_ID) }
}

pub fn native_factory_type(entity_ptr: usize) -> Option<FactoryType> {
    FactoryType::from_byte(native_factory_type_byte(entity_ptr)?)
}

// =============================================================================
//  Runtime alias cache
// =============================================================================

/// Кеш: `FNV-1 hash имени -> строковое имя`.
static HASH_TO_NAME: LazyLock<Mutex<HashMap<u64, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(2048)));

/// Кеш: `table_id -> строковое имя`.
static TABLE_ID_TO_NAME: LazyLock<Mutex<HashMap<u32, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(2048)));

pub fn observe_hashed_name(hash: u64, name: &str) {
    if hash == 0 || name.is_empty() || name.len() > 256 {
        return;
    }

    let Ok(mut map) = HASH_TO_NAME.try_lock() else {
        return;
    };

    map.entry(hash).or_insert_with(|| name.to_string());
}

pub fn observe_db_record_name_hash(db_record_ptr: usize, name_hash: u64) {
    if db_record_ptr == 0 || !memory::is_valid_ptr(db_record_ptr) {
        return;
    }

    if let Some(name) = lookup_name_by_hash(name_hash)
        && let Some(table_id) = (unsafe { memory::read::<u32>(db_record_ptr + entity::TABLE_ID) })
    {
        register_table_id_alias(table_id, &name);
    }
}

pub fn register_table_id_alias(table_id: u32, name: &str) {
    if table_id == 0 || name.is_empty() {
        return;
    }

    let Ok(mut map) = TABLE_ID_TO_NAME.lock() else {
        return;
    };

    map.entry(table_id).or_insert_with(|| name.to_string());
}

pub fn register_entity_alias(entity_ptr: usize, name: &str) {
    if entity_ptr == 0 || !memory::is_valid_ptr(entity_ptr) || name.is_empty() {
        return;
    }

    if let Some(table_id) = unsafe { memory::read::<u32>(entity_ptr + entity::TABLE_ID) } {
        register_table_id_alias(table_id, name);
    }
}

pub fn lookup_name_by_hash(hash: u64) -> Option<String> {
    HASH_TO_NAME.lock().ok()?.get(&hash).cloned()
}

pub fn lookup_known_name_by_table_id(table_id: u32) -> Option<String> {
    TABLE_ID_TO_NAME.lock().ok()?.get(&table_id).cloned()
}

pub fn lookup_known_name_for_entity(entity_ptr: usize) -> Option<String> {
    if entity_ptr == 0 || !memory::is_valid_ptr(entity_ptr) {
        return None;
    }

    let table_id = unsafe { memory::read::<u32>(entity_ptr + entity::TABLE_ID) }?;
    lookup_known_name_by_table_id(table_id)
}

pub fn alias_cache_stats() -> (usize, usize) {
    let hash_count = HASH_TO_NAME.lock().map(|m| m.len()).unwrap_or(0);
    let table_count = TABLE_ID_TO_NAME.lock().map(|m| m.len()).unwrap_or(0);
    (hash_count, table_count)
}

pub fn dump_alias_cache() {
    let (hash_count, table_count) = alias_cache_stats();
    logger::info(&format!(
        "[entity-alias] hash->name: {}, table_id->name: {}",
        hash_count, table_count
    ));

    if let Ok(map) = TABLE_ID_TO_NAME.lock() {
        for (table_id, name) in map.iter().take(128) {
            logger::info(&format!("  tid=0x{:08X} -> {}", table_id, name));
        }
    }
}