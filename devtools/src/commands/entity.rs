//! Entity сканирование и дамп.

use common::logger;
use sdk::{memory, game::Player};

pub fn dump_factory_registry() {
    let base = sdk::game::base();
    let registry = unsafe {
        memory::read_ptr(base + sdk::addresses::globals::ENTITY_WRAPPER_FACTORY_REGISTRY)
    };

    let Some(registry) = registry else {
        logger::warn("[entity-types] Registry not initialized");
        return;
    };

    let Some(tree_root) = (unsafe { memory::read_ptr(registry) }) else {
        logger::warn("[entity-types] Tree root NULL");
        return;
    };

    let Some(first_node) = (unsafe { memory::read_ptr(tree_root + 8) }) else {
        logger::warn("[entity-types] Tree empty");
        return;
    };

    logger::info(&format!(
        "[entity-types] Registry=0x{registry:X}, root=0x{tree_root:X}"
    ));

    let mut types = Vec::new();
    collect_rb_nodes(tree_root, first_node, &mut types);

    logger::info(&format!("[entity-types] Found {} types:", types.len()));
    for (type_id, factory) in &types {
        let vtbl = unsafe { memory::read_ptr_raw(*factory).unwrap_or(0) };
        let create_fn = unsafe { memory::read_ptr_raw(*factory + 0x10).unwrap_or(0) };
        logger::info(&format!(
            "  type=0x{type_id:02X} ({type_id:3}) | factory=0x{factory:X} | vtbl=0x{vtbl:X} | create=0x{create_fn:X}"
        ));
    }
}

fn collect_rb_nodes(sentinel: usize, node: usize, result: &mut Vec<(u32, usize)>) {
    if node == 0 || node == sentinel { return; }
    let is_sentinel = unsafe { memory::read_value::<u8>(node + 0x19).unwrap_or(1) };
    if is_sentinel != 0 { return; }

    let left = unsafe { memory::read_ptr_raw(node).unwrap_or(0) };
    collect_rb_nodes(sentinel, left, result);

    let type_id = unsafe { memory::read_value::<u32>(node + 0x20).unwrap_or(0xFFFFFFFF) };
    let factory = unsafe { memory::read_ptr_raw(node + 0x28).unwrap_or(0) };
    if type_id != 0xFFFFFFFF {
        result.push((type_id, factory));
    }

    let right = unsafe { memory::read_ptr_raw(node + 0x10).unwrap_or(0) };
    collect_rb_nodes(sentinel, right, result);
}

pub fn scan_all_cached() {
    let base = sdk::game::base();
    let mgr = unsafe {
        memory::read_ptr(base + sdk::addresses::globals::SCRIPT_WRAPPER_MANAGER)
    };
    let Some(mgr) = mgr else {
        logger::warn("[entity-scan] ScriptWrapperManager not ready");
        return;
    };

    let cache_begin = unsafe { memory::read_ptr_raw(mgr + 0x28).unwrap_or(0) };
    let cache_end = unsafe { memory::read_ptr_raw(mgr + 0x30).unwrap_or(0) };

    if cache_begin == 0 || cache_end <= cache_begin {
        logger::warn("[entity-scan] Cache empty");
        return;
    }

    let count = (cache_end - cache_begin) / 16;
    logger::info(&format!("[entity-scan] Cache: {count} entries"));

    let mut type_counts: std::collections::HashMap<u8, u32> = std::collections::HashMap::new();

    for i in 0..count.min(500) {
        let entry = cache_begin + i * 16;
        let wrapper = unsafe { memory::read_ptr_raw(entry + 8).unwrap_or(0) };
        if wrapper == 0 { continue; }
        let native = unsafe { memory::read_ptr_raw(wrapper + 0x10).unwrap_or(0) };
        if native == 0 { continue; }
        let entity_type = unsafe { memory::read_value::<u8>(native + 0x24).unwrap_or(0xFF) };
        *type_counts.entry(entity_type).or_insert(0) += 1;
    }

    let mut types: Vec<_> = type_counts.iter().collect();
    types.sort_by_key(|(t, _)| **t);

    logger::info(&format!("[entity-scan] {} types found:", types.len()));
    for (t, count) in &types {
        logger::info(&format!("  type=0x{t:02X} ({t:3}) | count={count:4}"));
    }
}

/// Дамп NPC структуры и сравнение с player.
pub fn dump_npc_vs_player() {
    let Some(player) = Player::get_active() else { return; };

    let joe = match sdk::game::entity::find_native_entity("Joe") {
        Some(ptr) => ptr,
        None => {
            logger::warn("[npc-probe] Joe не найден");
            return;
        }
    };

    let pp = player.as_ptr();

    logger::info(&format!("[npc-probe] Player=0x{:X}, Joe=0x{:X}", pp, joe));

    // Сравнение ключевых оффсетов
    unsafe {
        // Entity type
        let p_type = sdk::memory::read_value::<u8>(pp + 0x24).unwrap_or(0);
        let j_type = sdk::memory::read_value::<u8>(joe + 0x24).unwrap_or(0);
        logger::info(&format!("  entity_type: player=0x{:02X}, joe=0x{:02X}", p_type, j_type));

        // vtable
        let p_vt = sdk::memory::read_ptr_raw(pp).unwrap_or(0);
        let j_vt = sdk::memory::read_ptr_raw(joe).unwrap_or(0);
        logger::info(&format!("  vtable: player=0x{:X}, joe=0x{:X}", p_vt, j_vt));

        // Owner (+0x80)
        let p_own = sdk::memory::read_ptr_raw(pp + 0x80).unwrap_or(0);
        let j_own = sdk::memory::read_ptr_raw(joe + 0x80).unwrap_or(0);
        logger::info(&format!("  owner(+0x80): player=0x{:X}, joe=0x{:X}", p_own, j_own));

        // Health (+0x148)
        let p_hp = sdk::memory::read_value::<f32>(pp + 0x148).unwrap_or(0.0);
        let j_hp = sdk::memory::read_value::<f32>(joe + 0x148).unwrap_or(0.0);
        logger::info(&format!("  health(+0x148): player={:.0}, joe={:.0}", p_hp, j_hp));

        // HealthMax NPC (+0x14C)
        let j_hmax = sdk::memory::read_value::<f32>(joe + 0x14C).unwrap_or(0.0);
        logger::info(&format!("  npc_healthmax(+0x14C): joe={:.0}", j_hmax));

        // Invuln (+0x160)
        let p_inv = sdk::memory::read_value::<u8>(pp + 0x160).unwrap_or(255);
        let j_inv = sdk::memory::read_value::<u8>(joe + 0x160).unwrap_or(255);
        logger::info(&format!("  invulnerability(+0x160): player={}, joe={}", p_inv, j_inv));

        // Dead (+0x161)
        let p_dead = sdk::memory::read_value::<u8>(pp + 0x161).unwrap_or(255);
        let j_dead = sdk::memory::read_value::<u8>(joe + 0x161).unwrap_or(255);
        logger::info(&format!("  is_dead(+0x161): player={}, joe={}", p_dead, j_dead));

        // Demigod (+0x162)
        let p_demi = sdk::memory::read_value::<u8>(pp + 0x162).unwrap_or(255);
        let j_demi = sdk::memory::read_value::<u8>(joe + 0x162).unwrap_or(255);
        logger::info(&format!("  demigod(+0x162): player={}, joe={}", p_demi, j_demi));

        // Entity flags (+0x28)
        let p_flags = sdk::memory::read_value::<u32>(pp + 0x28).unwrap_or(0);
        let j_flags = sdk::memory::read_value::<u32>(joe + 0x28).unwrap_or(0);
        logger::info(&format!("  flags(+0x28): player=0x{:08X}, joe=0x{:08X}", p_flags, j_flags));

        // Inventory (+0xE8)
        let p_inv_ptr = sdk::memory::read_ptr_raw(pp + 0xE8).unwrap_or(0);
        let j_inv_ptr = sdk::memory::read_ptr_raw(joe + 0xE8).unwrap_or(0);
        logger::info(&format!("  inventory(+0xE8): player=0x{:X}, joe=0x{:X}", p_inv_ptr, j_inv_ptr));

        // AI params (+0xA8)
        let p_ai = sdk::memory::read_ptr_raw(pp + 0xA8).unwrap_or(0);
        let j_ai = sdk::memory::read_ptr_raw(joe + 0xA8).unwrap_or(0);
        logger::info(&format!("  ai_params(+0xA8): player=0x{:X}, joe=0x{:X}", p_ai, j_ai));

        // Aggressivity (*(+0xA8) + 4)
        if j_ai != 0 {
            let j_aggr = sdk::memory::read_value::<i32>(j_ai + 4).unwrap_or(-1);
            logger::info(&format!("  aggressivity: joe={}", j_aggr));
        }

        // Behavior (+0xF8)
        let p_beh = sdk::memory::read_ptr_raw(pp + 0xF8).unwrap_or(0);
        let j_beh = sdk::memory::read_ptr_raw(joe + 0xF8).unwrap_or(0);
        logger::info(&format!("  behavior(+0xF8): player=0x{:X}, joe=0x{:X}", p_beh, j_beh));

        // Self ref (+0x190)
        let p_self = sdk::memory::read_ptr_raw(pp + 0x190).unwrap_or(0);
        let j_self = sdk::memory::read_ptr_raw(joe + 0x190).unwrap_or(0);
        logger::info(&format!("  self_ref(+0x190): player=0x{:X}(==ptr:{}), joe=0x{:X}(==ptr:{})",
            p_self, p_self == pp, j_self, j_self == joe));

        // Component pointers comparison
        for (name, off) in [
            ("frame", 0x78usize), ("ai_nav", 0xC0), ("transform_sync", 0xD0),
            ("opt_comp", 0xD8), ("property_acc", 0xF0),
            ("weapon_state", 0x108), ("physics", 0x258),
        ] {
            let pv = sdk::memory::read_ptr_raw(pp + off).unwrap_or(0);
            let jv = sdk::memory::read_ptr_raw(joe + off).unwrap_or(0);
            let p_null = if pv == 0 { " (NULL)" } else { "" };
            let j_null = if jv == 0 { " (NULL)" } else { "" };
            logger::info(&format!("  {}(+0x{:X}): player=0x{:X}{}, joe=0x{:X}{}",
                name, off, pv, p_null, jv, j_null));
        }

        // Полный hex дамп Joe 0x00..0x1A0
        logger::info("[npc-probe] Joe hex dump +0x00..+0x1A0:");
        logger::info(&sdk::memory::hex_dump(joe, 0x1A0));
    }
}

/// Дамп всех зарегистрированных entity types из FactoryRegistry.
/// RB-tree: node+0x19=sentinel_flag, node+0x20=type_id(u32), node+0x28=factory_ptr
pub fn dump_entity_factory_registry() {
    let base = sdk::game::base();
    let registry = unsafe {
        memory::read_ptr(base + sdk::addresses::globals::ENTITY_WRAPPER_FACTORY_REGISTRY)
    };

    let Some(registry) = registry else {
        logger::warn("[entity-types] Registry не инициализирован");
        return;
    };

    // registry+0x00 → tree root (ptr to sentinel/root node)
    let Some(tree_root_ptr) = (unsafe { memory::read_ptr(registry) }) else {
        logger::warn("[entity-types] Tree root NULL");
        return;
    };

    // root node → +0x08 → first real node (left-most)
    let Some(first_node) = (unsafe { memory::read_ptr(tree_root_ptr + 8) }) else {
        logger::warn("[entity-types] Tree empty");
        return;
    };

    logger::info(&format!(
        "[entity-types] Registry=0x{:X}, root=0x{:X}",
        registry, tree_root_ptr
    ));

    // Обход in-order: рекурсивно собираем все узлы
    let mut types = Vec::new();
    collect_rb_tree_nodes(tree_root_ptr, first_node, &mut types);

    logger::info(&format!("[entity-types] Найдено {} типов:", types.len()));
    for (type_id, factory) in &types {
        // Прочитать vtable фабрики для идентификации
        let factory_vtbl = unsafe {
            memory::read_ptr_raw(*factory).unwrap_or(0)
        };
        // Прочитать create function (factory+0x10)
        let create_fn = unsafe {
            memory::read_ptr_raw(*factory + 0x10).unwrap_or(0)
        };
        logger::info(&format!(
            "  type=0x{:02X} ({:3}) | factory=0x{:X} | vtbl=0x{:X} | create=0x{:X}",
            type_id, type_id, factory, factory_vtbl, create_fn
        ));
    }
}

/// Рекурсивный обход RB-дерева (in-order).
/// node layout: +0x00=left, +0x10=right, +0x19=is_sentinel, +0x20=key(u32), +0x28=value(ptr)
fn collect_rb_tree_nodes(
    sentinel: usize,
    node: usize,
    result: &mut Vec<(u32, usize)>,
) {
    if node == 0 || node == sentinel {
        return;
    }

    // Проверить sentinel flag
    let is_sentinel = unsafe {
        memory::read_value::<u8>(node + 0x19).unwrap_or(1)
    };
    if is_sentinel != 0 {
        return;
    }

    // Left subtree
    let left = unsafe { memory::read_ptr_raw(node).unwrap_or(0) };
    collect_rb_tree_nodes(sentinel, left, result);

    // Current node
    let type_id = unsafe {
        memory::read_value::<u32>(node + 0x20).unwrap_or(0xFFFFFFFF)
    };
    let factory = unsafe {
        memory::read_ptr_raw(node + 0x28).unwrap_or(0)
    };
    if type_id != 0xFFFFFFFF {
        result.push((type_id, factory));
    }

    // Right subtree
    let right = unsafe { memory::read_ptr_raw(node + 0x10).unwrap_or(0) };
    collect_rb_tree_nodes(sentinel, right, result);
}