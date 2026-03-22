//! Чтение callback/event реестра движка.
//!
//! Этот модуль НЕ вмешивается в callback-систему игры.
//! Он только читает уже зарегистрированные event descriptor'ы
//! и callback entry. Полезно для:
//! - диагностики (какие события зарегистрированы, сколько callback'ов)
//! - проверки гипотез реверса (правильные ли ID событий)
//! - поиска хороших точек для hook'ов

use common::logger;

use crate::{
    addresses, memory,
    structures::{CallbackEventDesc, CallbackFunctionEntry, GameCallbackManager},
};

use super::base;

// =============================================================================
//  Типы для удобного доступа
// =============================================================================

/// Информация об одном зарегистрированном событии.
///
/// Это "дружественная" копия данных — можно хранить,
/// передавать между потоками, логировать.
#[derive(Debug, Clone)]
pub struct CallbackEventInfo {
    /// Адрес descriptor'а в памяти игры.
    pub desc_addr: usize,
    /// Имя события ("Game Tick", "Game Render" и т.д.).
    pub name: String,
    /// Уникальный ID события.
    pub event_id: i32,
    /// 1 = событие сейчас dispatch'ится.
    pub in_dispatch: i32,
    /// Количество подписанных callback'ов.
    pub func_count: usize,
    /// Начало массива callback'ов.
    pub funcs_begin: usize,
    /// Конец массива callback'ов.
    pub funcs_end: usize,
    /// Ёмкость массива callback'ов.
    pub funcs_capacity: usize,
}

/// Информация об одном callback внутри события.
#[derive(Debug, Clone)]
pub struct CallbackFunctionInfo {
    /// Адрес entry в памяти игры.
    pub entry_addr: usize,
    /// Объект-получатель (RCX при вызове).
    pub callback_object: usize,
    /// Адрес функции callback'а.
    pub callback_function: usize,
    /// Приоритет (чем меньше — тем раньше).
    pub priority: i32,
    /// Runtime-флаги (bit0 = active).
    pub flags: u8,
    /// Конфигурационная маска.
    pub config_mask: i32,
    /// Float-параметр callback'а.
    pub float_param: f32,
    /// Integer-параметр callback'а.
    pub int_param: i32,
    /// Зарезервировано.
    pub reserved: i32,
}

// =============================================================================
//  Чтение реестра
// =============================================================================

/// Указатель на глобальный GameCallbackManager.
pub fn get_manager_ptr() -> Option<usize> {
    unsafe { memory::read_ptr(base() + addresses::globals::GAME_CALLBACK_MANAGER) }
}

/// Прочитать список всех зарегистрированных событий.
///
/// Возвращает пустой вектор если менеджер не инициализирован.
/// Обычно в игре 39 событий.
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

    // Базовая проверка валидности вектора
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

        let Some(entry) = (unsafe { memory::read_value::<CallbackFunctionEntry>(entry_addr) })
        else {
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

// =============================================================================
//  Дамп в лог
// =============================================================================

/// Вывести в лог ключевые lifecycle-события.
///
/// Полезно при старте — сразу видно что callback-система работает
/// и сколько подписчиков у каждого события.
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
///
/// Выводит МНОГО текста. Использовать для глубокой отладки.
pub fn dump_registry() {
    let Some(manager_ptr) = get_manager_ptr() else {
        logger::warn("[callbacks] GameCallbackManager = NULL");
        return;
    };

    let events = list_events();
    if events.is_empty() {
        logger::warn("[callbacks] реестр событий пуст или невалиден");
        return;
    }

    let Some(manager) = (unsafe { memory::read_value::<GameCallbackManager>(manager_ptr) }) else {
        logger::error("[callbacks] не удалось прочитать GameCallbackManager");
        return;
    };

    logger::info(&format!(
        "[callbacks] manager=0x{:X} begin=0x{:X} end=0x{:X} cap=0x{:X} событий={}",
        manager_ptr,
        manager.entries_begin as usize,
        manager.entries_end as usize,
        manager.entries_capacity as usize,
        events.len(),
    ));

    for (i, event) in events.iter().enumerate() {
        logger::info(&format!(
            "[callbacks][{i:02}] id={} dispatch={} funcs={} \"{}\" addr=0x{:X}",
            event.event_id, event.in_dispatch, event.func_count, event.name, event.desc_addr,
        ));

        // Для каждого события — его callback'и (debug-уровень, чтобы не засорять)
        for (j, func) in get_functions_for_event(event.event_id).iter().enumerate() {
            logger::debug(&format!(
                "  [{j:02}] obj=0x{:X} fn=0x{:X} prio={} flags=0x{:02X} mask=0x{:X} \
                 float={} int={} addr=0x{:X}",
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

// =============================================================================
//  Вспомогательные функции
// =============================================================================

/// Прочитать null-terminated строку из фиксированного буфера.
fn fixed_c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}
