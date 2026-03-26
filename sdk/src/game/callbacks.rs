//! Чтение callback/event реестра движка.

use common::logger;

use crate::{
    addresses, memory,
    memory::Ptr,
    structures::{CallbackEventDesc, CallbackFunctionEntry, GameCallbackManager},
};

use super::base;

// =============================================================================
//  Типы для удобного доступа
// =============================================================================

/// Информация об одном зарегистрированном событии.
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

/// Информация об одном callback внутри события.
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

// =============================================================================
//  Внутренний typed helper
// =============================================================================

/// Typed reference на `GameCallbackManager`.
///
/// # Safety
///
/// Менеджер должен быть уже создан движком.
/// Использовать только из game thread.
unsafe fn manager_ref() -> Option<&'static GameCallbackManager> {
    let ptr = get_manager_ptr()?;
    let typed = Ptr::<GameCallbackManager>::new(ptr);
    debug_assert!(
        typed.raw().is_aligned(),
        "unaligned GameCallbackManager pointer"
    );
    Some(unsafe { &*typed.raw() })
}

// =============================================================================
//  Чтение реестра
// =============================================================================

/// Указатель на глобальный `GameCallbackManager`.
pub fn get_manager_ptr() -> Option<usize> {
    unsafe { memory::read_ptr(base() + addresses::globals::GAME_CALLBACK_MANAGER) }
}

/// Прочитать список всех зарегистрированных событий.
pub fn list_events() -> Vec<CallbackEventInfo> {
    let Some(manager) = (unsafe { manager_ref() }) else {
        return Vec::new();
    };

    let events = unsafe { manager.events() };
    let mut result = Vec::with_capacity(events.len());

    for event in events {
        result.push(CallbackEventInfo {
            desc_addr: event as *const CallbackEventDesc as usize,
            name: event.name_string(),
            event_id: event.event_id,
            in_dispatch: event.in_dispatch,
            func_count: event.callback_count(),
            funcs_begin: event.funcs.begin_addr(),
            funcs_end: event.funcs.end_addr(),
            funcs_capacity: event.funcs.capacity as usize,
        });
    }

    result
}

/// Найти событие по ID.
pub fn find_event_by_id(event_id: i32) -> Option<CallbackEventInfo> {
    list_events().into_iter().find(|e| e.event_id == event_id)
}

/// Найти событие по имени (без учёта регистра).
pub fn find_event_by_name(name: &str) -> Option<CallbackEventInfo> {
    list_events()
        .into_iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
}

/// Прочитать все callback'и конкретного события.
pub fn get_functions_for_event(event_id: i32) -> Vec<CallbackFunctionInfo> {
    let Some(manager) = (unsafe { manager_ref() }) else {
        return Vec::new();
    };

    let events = unsafe { manager.events() };
    let Some(event) = events.iter().find(|e| e.event_id == event_id) else {
        return Vec::new();
    };

    let funcs = unsafe { event.callbacks() };
    let mut result = Vec::with_capacity(funcs.len());

    for entry in funcs {
        result.push(CallbackFunctionInfo {
            entry_addr: entry as *const CallbackFunctionEntry as usize,
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

// =============================================================================
//  Дамп в лог
// =============================================================================

/// Вывести в лог ключевые lifecycle-события.
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

    logger::info("[callbacks] ключевые события:");

    for id in ids {
        if let Some(event) = find_event_by_id(id) {
            logger::info(&format!(
                "  \"{}\" id={} dispatch={} funcs={} addr=0x{:X}",
                event.name, event.event_id, event.in_dispatch, event.func_count, event.desc_addr,
            ));
        }
    }
}

/// Полный дамп реестра — все события + все callback'и.
pub fn dump_registry() {
    let Some(manager_ptr) = get_manager_ptr() else {
        logger::warn("[callbacks] GameCallbackManager = NULL");
        return;
    };

    let Some(manager) = (unsafe { manager_ref() }) else {
        logger::error("[callbacks] не удалось прочитать GameCallbackManager");
        return;
    };

    let events = unsafe { manager.events() };
    if events.is_empty() {
        logger::warn("[callbacks] реестр событий пуст или невалиден");
        return;
    }

    logger::info(&format!(
        "[callbacks] manager=0x{:X} events={} pending={} {:?}",
        manager_ptr,
        manager.entries.len(),
        manager.pending.len(),
        manager.entries,
    ));

    for (i, event) in events.iter().enumerate() {
        let funcs = unsafe { event.callbacks() };

        logger::info(&format!(
            "[callbacks][{i:02}] id={} dispatch={} funcs={} \"{}\" addr=0x{:X}",
            event.event_id,
            event.in_dispatch,
            funcs.len(),
            event.name_string(),
            event as *const CallbackEventDesc as usize,
        ));

        for (j, func) in funcs.iter().enumerate() {
            logger::debug(&format!(
                "  [{j:02}] obj=0x{:X} fn=0x{:X} prio={} flags=0x{:02X} mask=0x{:X} \
                 float={} int={} addr=0x{:X}",
                func.callback_object as usize,
                func.callback_function as usize,
                func.priority,
                func.flags,
                func.config_mask,
                func.float_param,
                func.int_param,
                func as *const CallbackFunctionEntry as usize,
            ));
        }
    }
}