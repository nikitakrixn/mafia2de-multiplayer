//! Runtime helpers for `GameCallbackManager`.
//!
//! Этот модуль не вмешивается в callback system игры,
//! а только читает уже зарегистрированные event descriptor'ы и callback entries.
//!
//! Полезно для:
//! - диагностики
//! - проверки reverse engineering гипотез
//! - поиска хороших hook / lifecycle target'ов

use common::logger;

use crate::{
    addresses,
    memory,
    structures::{CallbackEventDesc, CallbackFunctionEntry, GameCallbackManager},
};

use super::base;

#[derive(Debug, Clone)]
pub struct CallbackEventInfo {
    pub desc_addr: usize,
    pub name: String,
    pub event_id: i32,
    pub in_dispatch: i32,
    pub func_count: usize,
    pub funcs_begin: usize,
    pub funcs_end: usize,
    pub funcs_capacity: usize,
}

#[derive(Debug, Clone)]
pub struct CallbackFunctionInfo {
    pub entry_addr: usize,
    pub callback_object: usize,
    pub callback_function: usize,
    pub priority: i32,
    pub flags: u8,
    pub config_mask: i32,
    pub float_param: f32,
    pub int_param: i32,
    pub reserved: i32,
}

fn fixed_c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

pub fn get_manager_ptr() -> Option<usize> {
    unsafe { memory::read_ptr(base() + addresses::globals::GAME_CALLBACK_MANAGER) }
}

pub fn list_events() -> Vec<CallbackEventInfo> {
    let Some(manager_ptr) = get_manager_ptr() else {
        return Vec::new();
    };

    let Some(manager) = (unsafe { memory::read_value::<GameCallbackManager>(manager_ptr) }) else {
        return Vec::new();
    };

    let begin = manager.entries_begin as usize;
    let end = manager.entries_end as usize;
    let cap = manager.entries_capacity as usize;

    if begin == 0 || end < begin || cap < end {
        return Vec::new();
    }

    let entry_size = std::mem::size_of::<CallbackEventDesc>();
    let event_count = (end - begin) / entry_size;
    let mut result = Vec::with_capacity(event_count);

    for i in 0..event_count {
        let entry_addr = begin + i * entry_size;

        let Some(entry) = (unsafe { memory::read_value::<CallbackEventDesc>(entry_addr) }) else {
            continue;
        };

        let funcs_begin = entry.funcs_begin as usize;
        let funcs_end = entry.funcs_end as usize;
        let funcs_capacity = entry.funcs_capacity as usize;

        let func_count = if funcs_begin != 0 && funcs_end >= funcs_begin {
            (funcs_end - funcs_begin) / std::mem::size_of::<CallbackFunctionEntry>()
        } else {
            0
        };

        result.push(CallbackEventInfo {
            desc_addr: entry_addr,
            name: fixed_c_string(&entry.name),
            event_id: entry.event_id,
            in_dispatch: entry.in_dispatch,
            func_count,
            funcs_begin,
            funcs_end,
            funcs_capacity,
        });
    }

    result
}

pub fn find_event_by_id(event_id: i32) -> Option<CallbackEventInfo> {
    list_events().into_iter().find(|e| e.event_id == event_id)
}

pub fn find_event_by_name(name: &str) -> Option<CallbackEventInfo> {
    list_events()
        .into_iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
}

pub fn get_functions_for_event(event_id: i32) -> Vec<CallbackFunctionInfo> {
    let Some(event) = find_event_by_id(event_id) else {
        return Vec::new();
    };

    if event.funcs_begin == 0 || event.funcs_end < event.funcs_begin {
        return Vec::new();
    }

    let entry_size = std::mem::size_of::<CallbackFunctionEntry>();
    let func_count = (event.funcs_end - event.funcs_begin) / entry_size;
    let mut result = Vec::with_capacity(func_count);

    for i in 0..func_count {
        let entry_addr = event.funcs_begin + i * entry_size;

        let Some(entry) = (unsafe { memory::read_value::<CallbackFunctionEntry>(entry_addr) }) else {
            continue;
        };

        result.push(CallbackFunctionInfo {
            entry_addr,
            callback_object: entry.callback_object as usize,
            callback_function: entry.callback_function as usize,
            priority: entry.priority,
            flags: entry.flags,
            config_mask: entry.config_mask,
            float_param: entry.float_param,
            int_param: entry.int_param,
            reserved: entry.reserved,
        });
    }

    result
}

pub fn dump_interesting_events() {
    use crate::addresses::constants::game_events as ev;

    let ids = [
        ev::NO_GAME_START,
        ev::NO_GAME_END,
        ev::MISSION_BEFORE_OPEN,
        ev::MISSION_AFTER_OPEN,
        ev::MISSION_BEFORE_CLOSE,
        ev::MISSION_AFTER_CLOSE,
        ev::LOADING_PROCESS_STARTED,
        ev::LOADING_PROCESS_FINISHED,
        ev::LOADING_FADE_FINISHED,
        ev::GAME_INIT,
        ev::GAME_DONE,
        ev::GAME_PAUSED,
        ev::GAME_UNPAUSED,
        ev::APP_ACTIVATE,
        ev::APP_DEACTIVATE,
        ev::SHUTDOWN,
        ev::GAME_TICK_ALWAYS,
        ev::GAME_RENDER,
    ];

    logger::info("[callbacks] interesting events:");

    for id in ids {
        if let Some(event) = find_event_by_id(id) {
            logger::info(&format!(
                "  name=\"{}\" id={} in_dispatch={} funcs={} desc=0x{:X}",
                event.name, event.event_id, event.in_dispatch, event.func_count, event.desc_addr
            ));
        }
    }
}

pub fn dump_registry() {
    let Some(manager_ptr) = get_manager_ptr() else {
        logger::warn("[callbacks] GameCallbackManager is NULL");
        return;
    };

    let events = list_events();
    if events.is_empty() {
        logger::warn("[callbacks] event registry is empty or invalid");
        return;
    }

    let Some(manager) = (unsafe { memory::read_value::<GameCallbackManager>(manager_ptr) }) else {
        logger::error("[callbacks] Failed to read GameCallbackManager");
        return;
    };

    logger::info(&format!(
        "[callbacks] manager=0x{:X} begin=0x{:X} end=0x{:X} cap=0x{:X} events={}",
        manager_ptr,
        manager.entries_begin as usize,
        manager.entries_end as usize,
        manager.entries_capacity as usize,
        events.len(),
    ));

    for (i, event) in events.iter().enumerate() {
        logger::info(&format!(
            "[callbacks][{i:02}] id={} in_dispatch={} funcs={} name=\"{}\" desc=0x{:X}",
            event.event_id,
            event.in_dispatch,
            event.func_count,
            event.name,
            event.desc_addr,
        ));

        for (j, func) in get_functions_for_event(event.event_id).iter().enumerate() {
            logger::debug(&format!(
                "  [fn {j:02}] obj=0x{:X} func=0x{:X} prio={} flags=0x{:02X} mask=0x{:X} float_param={} int_param={} entry=0x{:X}",
                func.callback_object,
                func.callback_function,
                func.priority,
                func.flags,
                func.config_mask,
                func.float_param,
                func.int_param,
                func.entry_addr,
            ));
        }
    }
}