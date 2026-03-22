//! SDS (Streaming Data System) — загрузка и управление ресурсами мира.

use crate::addresses;
use crate::addresses::fields::sds_line_cache as slc;
use crate::game::base;
use crate::memory;
use common::logger;
use std::ffi::CString;

/// FNV-1a хеш для SDS stream map lines (lowercase, Mafia II variant).
/// hash = byte XOR (hash * prime)
pub fn fnv1a_sds_hash(name: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF29CE484222325;
    for &byte in name {
        let lower = byte.to_ascii_lowercase();
        hash = (lower as u64) ^ hash.wrapping_mul(0x100000001B3);
    }
    hash
}

pub fn activate_stream_map_line(name: &str) -> bool {
    let base = base();

    let sds_mgr = unsafe { memory::read_ptr(base + addresses::globals::SDS_MANAGER) };
    let Some(sds_mgr) = sds_mgr else {
        logger::error("[sds] SDSManager not ready");
        return false;
    };

    let loader_ctx = sds_mgr + 8;
    let c_name = match CString::new(name) {
        Ok(s) => s,
        Err(_) => return false,
    };

    type ActivateFn = unsafe extern "C" fn(usize, *mut usize, *const i8) -> *mut usize;
    let func: ActivateFn =
        unsafe { memory::fn_at(base + addresses::functions::sds::ACTIVATE_STREAM_MAP_LINE) };

    let mut result: usize = 0;
    let ret = unsafe { func(loader_ctx, &mut result, c_name.as_ptr()) };

    if ret.is_null() || result == 0 {
        logger::warn(&format!("[sds] '{name}' — not found or load error"));
        false
    } else {
        logger::info(&format!("[sds] '{name}' — loaded"));
        true
    }
}

/// Проверить существование SDS линии в кеше (без загрузки).
pub fn line_exists(name: &str) -> bool {
    find_line_index(name).is_some()
}

/// Найти индекс SDS линии по имени (binary search в hash кеше).
pub fn find_line_index(name: &str) -> Option<u32> {
    let target_hash = fnv1a_sds_hash(name.as_bytes());
    let base = base();

    let mgr2 = unsafe { memory::read_ptr(base + addresses::globals::SDS_LINE_MANAGER)? };
    let cache_begin = unsafe { memory::read_ptr_raw(mgr2 + slc::CACHE_BEGIN)? };
    let cache_end = unsafe { memory::read_ptr_raw(mgr2 + slc::CACHE_END)? };
    if cache_begin == 0 || cache_end <= cache_begin {
        return None;
    }

    let count = (cache_end - cache_begin) / slc::ENTRY_SIZE;

    // Binary search
    let mut lo: usize = 0;
    let mut hi: usize = count;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let h = unsafe {
            memory::read_value::<u64>(cache_begin + mid * slc::ENTRY_SIZE + slc::ENTRY_HASH)
                .unwrap_or(0)
        };
        if h < target_hash {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    if lo < count {
        let found = unsafe {
            memory::read_value::<u64>(cache_begin + lo * slc::ENTRY_SIZE + slc::ENTRY_HASH)
                .unwrap_or(0)
        };
        if found == target_hash {
            return unsafe {
                memory::read_value::<u32>(cache_begin + lo * slc::ENTRY_SIZE + slc::ENTRY_INDEX)
            };
        }
    }
    None
}

/// Известные SDS линии для freeride.
pub mod known_lines {
    pub const JOE_LOAD: &str = "free_joe_load";
    pub const SUMMER_LOAD: &str = "free_summer_load";
    pub const WINTER_LOAD: &str = "free_winter_load";
    pub const MANSION_LOAD: &str = "fr_sds_mansion_load";
    pub const MANSION_UNLOAD: &str = "fr_sds_mansion_unload";
    pub const MANSION_AREAS_1: &str = "fr_mansion_areas_1";
    pub const MANSION_AREAS_2: &str = "fr_mansion_areas_2";
}
