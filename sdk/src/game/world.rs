//! Сканирование мировых сущностей.
//!
//! Два режима:
//! 1. cached entities — через ScriptWrapperManager (стабильно, немного сущностей)
//! 2. all world entities — через WorldEntityManager / EntityDatabase (полный мир)
//!
//! Подтверждено runtime:
//! - WorldEntityManager == EntityDatabase (один объект)
//! - object + 0x18 = logical entity count
//! - object + 0x38 .. +0x38+4096*8 = open-addressing array of entity pointers
//! - нужно сканировать именно 4096 слотов, а не только count

use std::collections::{HashMap, HashSet};

use super::{base, entity, entity_ref::EntityRef, entity_types::EntityType};
use crate::{addresses, memory};

/// Количество слотов в EntityDatabase open-addressing массиве.
const DB_SLOT_COUNT: usize = 4096;

/// Информация об одной сущности мира.
#[derive(Clone, Debug)]
pub struct WorldEntityInfo {
    /// Native entity pointer.
    pub ptr: usize,
    /// Packed table_id.
    pub table_id: u32,
    /// Instance id (table_id >> 8).
    pub instance_id: u32,
    /// Native factory type byte.
    pub factory_type: u8,
    /// Имя factory type.
    pub type_name: &'static str,
    /// Lua-facing type, если известен.
    pub lua_type: Option<EntityType>,
    /// Primary vtable.
    pub vtable: usize,
    /// Name hash from entity+0x30.
    pub name_hash: u64,
    /// Позиция в мире.
    pub pos: Option<[f32; 3]>,
    /// Расстояние до игрока.
    pub distance: f32,
    /// Runtime alias name, если был замечен в lookup.
    pub runtime_name: Option<String>,
    /// Human-only health.
    pub health: Option<f32>,
    /// Human-only alive flag.
    pub is_alive: Option<bool>,
    /// Human-only in-vehicle state.
    pub in_vehicle: Option<bool>,
}

// =============================================================================
//  Nearby entities via wrapper cache
// =============================================================================

/// Сканирует nearby cached entities через ScriptWrapperManager.
pub fn scan_nearby_entities(
    player_pos: [f32; 3],
    radius: f32,
    max_count: usize,
) -> Vec<WorldEntityInfo> {
    let base = base();

    let mgr = unsafe {
        match memory::read_ptr(base + addresses::globals::SCRIPT_WRAPPER_MANAGER) {
            Some(p) if p != 0 && memory::is_valid_ptr(p) => p,
            _ => return Vec::new(),
        }
    };

    let cache_begin = unsafe {
        memory::read_ptr_raw(mgr + addresses::fields::entity_cache::TABLE_ID_CACHE_BEGIN)
            .unwrap_or(0)
    };
    let cache_end = unsafe {
        memory::read_ptr_raw(mgr + addresses::fields::entity_cache::TABLE_ID_CACHE_END).unwrap_or(0)
    };

    if cache_begin == 0 || cache_end <= cache_begin || !memory::is_valid_ptr(cache_begin) {
        return Vec::new();
    }

    let entry_count = (cache_end - cache_begin) / addresses::fields::entity_cache::ENTRY_SIZE;
    if entry_count == 0 || entry_count > 10000 {
        return Vec::new();
    }

    let radius_sq = radius * radius;
    let mut results = Vec::with_capacity(64);

    for i in 0..entry_count {
        let entry = cache_begin + i * addresses::fields::entity_cache::ENTRY_SIZE;

        let wrapper = unsafe {
            match memory::read_ptr_raw(entry + addresses::fields::entity_cache::ENTRY_WRAPPER) {
                Some(p) if p != 0 && memory::is_valid_ptr(p) => p,
                _ => continue,
            }
        };

        let Some(ent) = EntityRef::from_wrapper_ptr(wrapper) else {
            continue;
        };

        let pos = ent.position().map(|p| [p.x, p.y, p.z]);
        let Some([x, y, z]) = pos else {
            continue;
        };

        let dx = x - player_pos[0];
        let dy = y - player_pos[1];
        let dz = z - player_pos[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq > radius_sq {
            continue;
        }

        let distance = dist_sq.sqrt();
        results.push(build_info(ent, distance));
    }

    results.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(max_count);
    results
}

// =============================================================================
//  Full DB scan
// =============================================================================

fn world_entity_manager_ptr() -> Option<usize> {
    unsafe {
        memory::read_ptr(base() + addresses::globals::WORLD_ENTITY_MANAGER)
            .or_else(|| memory::read_ptr(base() + addresses::globals::ENTITY_DATABASE))
    }
}

/// Считает все сущности в мире по factory type.
pub fn count_entities_by_type() -> Vec<(u8, &'static str, u32)> {
    let Some(mgr) = world_entity_manager_ptr() else {
        return Vec::new();
    };

    let mut seen = HashSet::with_capacity(2048);
    let mut by_type: HashMap<u8, u32> = HashMap::new();

    for slot in 0..DB_SLOT_COUNT {
        let ptr_addr = mgr + 0x38 + slot * 8;
        if !memory::is_valid_ptr(ptr_addr) {
            continue;
        }

        let entity_ptr = unsafe { memory::read_ptr_raw(ptr_addr).unwrap_or(0) };
        if entity_ptr == 0 || !memory::is_valid_ptr(entity_ptr) {
            continue;
        }

        if !seen.insert(entity_ptr) {
            continue;
        }

        let Some(ent) = EntityRef::from_ptr(entity_ptr) else {
            continue;
        };

        let Some(ft) = ent.factory_type_byte() else {
            continue;
        };

        *by_type.entry(ft).or_insert(0) += 1;
    }

    let mut result: Vec<_> = by_type
        .into_iter()
        .map(|(ft, count)| (ft, entity::factory_type_name(ft), count))
        .collect();

    result.sort_by_key(|(_, _, c)| std::cmp::Reverse(*c));
    result
}

/// Дампит все сущности мира через полный DB scan.
pub fn dump_all_world_entities() -> Vec<WorldEntityInfo> {
    let Some(mgr) = world_entity_manager_ptr() else {
        return Vec::new();
    };

    let mut seen = HashSet::with_capacity(2048);
    let mut results = Vec::with_capacity(2500);

    for slot in 0..DB_SLOT_COUNT {
        let ptr_addr = mgr + 0x38 + slot * 8;
        if !memory::is_valid_ptr(ptr_addr) {
            continue;
        }

        let entity_ptr = unsafe { memory::read_ptr_raw(ptr_addr).unwrap_or(0) };
        if entity_ptr == 0 || !memory::is_valid_ptr(entity_ptr) {
            continue;
        }

        if !seen.insert(entity_ptr) {
            continue;
        }

        let Some(ent) = EntityRef::from_ptr(entity_ptr) else {
            continue;
        };

        results.push(build_info(ent, 0.0));
    }

    results
}

// =============================================================================
//  Helpers
// =============================================================================

fn build_info(ent: EntityRef, distance: f32) -> WorldEntityInfo {
    let table_id = ent.table_id().unwrap_or(0);
    let factory_type = ent.factory_type_byte().unwrap_or(0);
    let vtable = ent.vtable().unwrap_or(0);
    let name_hash = ent.name_hash().unwrap_or(0);
    let pos = ent.position().map(|p| [p.x, p.y, p.z]);

    WorldEntityInfo {
        ptr: ent.ptr(),
        table_id,
        instance_id: table_id >> 8,
        factory_type,
        type_name: entity::factory_type_name(factory_type),
        lua_type: ent.lua_entity_type(),
        vtable,
        name_hash,
        pos,
        distance,
        runtime_name: entity::lookup_known_name_by_table_id(table_id),
        health: ent.health(),
        is_alive: ent.is_alive(),
        in_vehicle: ent.is_in_vehicle(),
    }
}
