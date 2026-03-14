//! SDS, map initialization, world scanning.

use common::logger;
use sdk::memory;

/// Дампит все зарегистрированные SDS stream map lines.
pub fn dump_sds_lines() {
    let base = sdk::game::base();

    let sds_mgr = unsafe {
        memory::read_ptr(base + sdk::addresses::globals::SDS_MANAGER)
    };

    let Some(_sds_mgr) = sds_mgr else {
        logger::warn("[sds] SDSManager не готов");
        return;
    };

    // qword_141CAF7B0 — вторичный менеджер, используется в ActivateStreamMapLine
    // Внутри ActivateStreamMapLine: v10 = qword_141CAF7B0
    // cache = v10[3]..v10[4] (24 bytes/entry: hash(8) + data(8) + flags(4) + index(4))
    let mgr2_addr = base + 0x1CA_F7B0; // qword_141CAF7B0
    let mgr2 = unsafe { memory::read_ptr(mgr2_addr) };
    let Some(mgr2) = mgr2 else {
        logger::warn("[sds] Вторичный SDS менеджер NULL");
        return;
    };

    // mgr2 layout: [0]=vtable?, [1]=?, [2]=?, [3]=cache_begin, [4]=cache_end
    let cache_begin = unsafe { memory::read_ptr_raw(mgr2 + 24).unwrap_or(0) }; // mgr2[3]
    let cache_end = unsafe { memory::read_ptr_raw(mgr2 + 32).unwrap_or(0) };   // mgr2[4]

    if cache_begin == 0 || cache_end <= cache_begin {
        logger::warn("[sds] Кеш линий пуст");
        // Попробуем другие оффсеты
        for off in [8usize, 16, 24, 32, 40, 48, 56, 64] {
            let v = unsafe { memory::read_ptr_raw(mgr2 + off).unwrap_or(0) };
            logger::debug(&format!("  mgr2+0x{:02X} = 0x{:X}", off, v));
        }
        return;
    }

    let entry_size = 24usize; // из ActivateStreamMapLine: 24 bytes/entry
    let count = (cache_end - cache_begin) / entry_size;

    logger::info(&format!(
        "[sds] SDS линии: {} записей (0x{:X}..0x{:X})",
        count, cache_begin, cache_end
    ));

    for i in 0..count.min(200) {
        let entry = cache_begin + i * entry_size;
        let hash = unsafe { memory::read_value::<u64>(entry).unwrap_or(0) };
        let data = unsafe { memory::read_ptr_raw(entry + 8).unwrap_or(0) };
        let flags = unsafe { memory::read_value::<u32>(entry + 16).unwrap_or(0) };
        let index = unsafe { memory::read_value::<u32>(entry + 20).unwrap_or(0) };

        logger::info(&format!(
            "  [{:3}] hash=0x{:016X} data=0x{:X} flags=0x{:X} idx={}",
            i, hash, data, flags, index
        ));
    }
}

pub fn init_map(season: &str, weather: &str) {
    match season {
        "summer" => sdk::game::sds::activate_stream_map_line("free_summer_load"),
        "winter" => sdk::game::sds::activate_stream_map_line("free_winter_load"),
        _ => sdk::game::sds::activate_stream_map_line("free_summer_load"),
    };

    let weather_cmd = format!("game.gfx:SetWeatherTemplate(\"{}\")", weather);
    sdk::game::lua::exec(&weather_cmd).ok();
    sdk::game::sds::activate_stream_map_line("free_joe_load");
    sdk::game::lua::exec("game.traffic:OpenSeason(140)").ok();

    logger::info("[mapping] Map initialized");
}

// Поиск SDS линии по имени в runtime кеше.
/// Возвращает (found, index) если линия существует.
pub fn find_sds_line_by_name(name: &str) -> Option<(u64, u32)> {
    let target_hash = sdk::game::sds::fnv1a_sds_hash(name.as_bytes());
    
    let base = sdk::game::base();
    let mgr2 = unsafe {
        memory::read_ptr(base + 0x1CA_F7B0)?  // qword_141CAF7B0
    };

    let cache_begin = unsafe { memory::read_ptr_raw(mgr2 + 24)? };
    let cache_end = unsafe { memory::read_ptr_raw(mgr2 + 32)? };
    if cache_begin == 0 || cache_end <= cache_begin { return None; }

    let count = (cache_end - cache_begin) / 24;

    // Binary search по хешу
    let mut lo: usize = 0;
    let mut hi: usize = count;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let entry_hash = unsafe {
            memory::read_value::<u64>(cache_begin + mid * 24).unwrap_or(0)
        };
        if entry_hash < target_hash {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    if lo < count {
        let found_hash = unsafe {
            memory::read_value::<u64>(cache_begin + lo * 24).unwrap_or(0)
        };
        if found_hash == target_hash {
            let idx = unsafe {
                memory::read_value::<u32>(cache_begin + lo * 24 + 20).unwrap_or(0xFFFFFFFF)
            };
            return Some((target_hash, idx));
        }
    }
    None
}

/// Проверить что конкретные SDS линии существуют в кеше.
pub fn verify_known_sds_lines() {
    let lines = [
        "free_joe_load", "free_summer_load", "free_winter_load",
        "summer_unload", "fr_sds_mansion_load", "fr_sds_mansion_unload",
        "fr_mansion_areas_1", "fr_mansion_areas_2",
        "fmv_1103_load", "m15_planet_load",
    ];

    logger::info("[sds] Проверка известных SDS линий:");
    for name in &lines {
        match find_sds_line_by_name(name) {
            Some((hash, idx)) => {
                logger::info(&format!(
                    "  ✓ '{}' → hash=0x{:016X} idx={}",
                    name, hash, idx
                ));
            }
            None => {
                let hash = sdk::game::sds::fnv1a_sds_hash(name.as_bytes());
                logger::warn(&format!(
                    "  ✗ '{}' → hash=0x{:016X} NOT FOUND",
                    name, hash
                ));
            }
        }
    }
}

/// Сканирует мир и определяет entity_type для каждого найденного entity.
pub fn scan_world_entity_types() {
    let known_names = [
        // NPC
        "Joe", "Henry", "Eddie",
    ];

    logger::info("[world-scan] Сканирование entity по именам:");

    for name in &known_names {
        match sdk::game::entity::find_native_entity(name) {
            Some(ptr) => {
                let entity_type = unsafe {
                    memory::read_value::<u8>(ptr + 0x24).unwrap_or(0xFF)
                };
                let vtable = unsafe {
                    memory::read_ptr_raw(ptr).unwrap_or(0)
                };
                logger::info(&format!(
                    "  '{}' → type=0x{:02X} ({}) vtbl=0x{:X}",
                    name, entity_type, entity_type, vtable
                ));
            }
            None => {
                logger::debug(&format!("  '{}' → не найден", name));
            }
        }
    }
}
