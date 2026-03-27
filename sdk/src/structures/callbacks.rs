//! Callback/event система движка.
//!
//! GameCallbackManager хранит зарегистрированные типы событий
//! и подписанные на них callback-функции. Через эту систему
//! проходят все lifecycle-события: тики, рендер, загрузка,
//! пауза, активация/деактивация окна.
//!
//! Подтверждено по:
//! - sub_1403A08F0 — RegisterEventType
//! - sub_1403A06D0 — RegisterFunction
//! - sub_1403A15E0 — FireEventById
//! - sub_1403A16A0 — DispatchSingleEventByIndex
//! - sub_1403A1A00 — DispatchEventsInternal
//! - sub_1403A55A0 — UnregisterFunction
//! - sub_1403A57E0 — UnregisterFunctionsByObject
//! - sub_1403A64F0 — SetFunctionFlags
//! - sub_1403B4AD0 — FlushPendingFunctionOps

use crate::macros::{assert_field_offsets, assert_layout};
use std::ffi::c_void;

use super::std_vector::StdVector;

/// Глобальный менеджер callback/event системы.
///
/// Глобал: `addresses::globals::GAME_CALLBACK_MANAGER`
///
/// Внутри:
/// - `std::vector<CallbackEventDesc>` — зарегистрированные события
/// - `std::vector<PendingFunctionOp>` — отложенные операции
/// - указатель на текущий DispatchContext
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GameCallbackManager {
    pub vtable: *const c_void, // +0x00

    /// Зарегистрированные события (`std::vector<CallbackEventDesc>`).
    pub entries: StdVector<CallbackEventDesc>, // +0x08

    /// Отложенные add/remove операции (`std::vector<PendingFunctionOp>`).
    pub pending: StdVector<PendingFunctionOp>, // +0x20

    /// Активный контекст во время dispatch (NULL в покое).
    pub current_dispatch_ctx: *mut DispatchContext, // +0x38
}

/// Один зарегистрированный тип события.
///
/// Примеры: "Game Tick" (id=3), "Game Tick Always" (id=5),
/// "Game Render" (id=7), "Mission After Open" (id=10).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallbackEventDesc {
    /// Имя события, null-terminated, до 0x1F символов.
    pub name: [u8; 0x20], // +0x00

    /// Уникальный ID события (3, 5, 7, ...).
    pub event_id: i32, // +0x20

    /// Reentrancy lock: 1 = событие сейчас dispatch'ится.
    /// Сбрасывается в 0 после завершения dispatch.
    pub in_dispatch: i32, // +0x24

    /// Подписанные callback'и (`std::vector<CallbackFunctionEntry>`).
    pub funcs: StdVector<CallbackFunctionEntry>, // +0x28
}

impl CallbackEventDesc {
    /// Прочитать имя как Rust-строку.
    pub fn name_string(&self) -> String {
        let end = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.name.len());
        String::from_utf8_lossy(&self.name[..end]).into_owned()
    }

    /// Количество подписанных callback'ов.
    #[inline]
    pub fn callback_count(&self) -> usize {
        self.funcs.len()
    }

    /// Слайс callback'ов.
    ///
    /// # Safety
    ///
    /// Game thread only.
    #[inline]
    pub unsafe fn callbacks(&self) -> &[CallbackFunctionEntry] {
        unsafe { self.funcs.as_slice() }
    }

    /// Событие сейчас dispatch'ится.
    #[inline]
    pub fn is_dispatching(&self) -> bool {
        self.in_dispatch != 0
    }
}

/// Один зарегистрированный callback внутри события.
///
/// Prototype callback'а:
/// `void __fastcall fn(void* callback_object, DispatchContext* ctx)`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallbackFunctionEntry {
    /// Первый аргумент callback'а (RCX при вызове).
    pub callback_object: *const c_void, // +0x00

    /// Указатель на функцию callback'а.
    pub callback_function: *const c_void, // +0x08

    /// Приоритет: чем меньше, тем раньше вызывается.
    pub priority: i32, // +0x10

    /// Runtime-флаги:
    /// - bit0 = active/enabled
    /// - bit1, bit2 — управление dispatch chain'ом
    /// Dispatcher очищает bit1+bit2 перед вызовом: `flags &= 0xF9`.
    pub flags: u8, // +0x14
    pub _pad_015: [u8; 3], // +0x15

    /// Конфигурационная маска (0x40F00000, 0x7500000 и т.д.).
    pub config_mask: i32, // +0x18

    /// Float-параметр (0.005, 0.003, -1.0 и т.д.).
    pub float_param: f32, // +0x1C

    /// Дополнительный integer-параметр.
    pub int_param: i32, // +0x20

    /// Не расшифровано.
    pub reserved: i32, // +0x24
}

impl CallbackFunctionEntry {
    /// Callback активен (bit 0).
    #[inline]
    pub fn is_active(&self) -> bool {
        (self.flags & 0x01) != 0
    }
}

/// Отложенная операция над callback-системой.
///
/// Создаётся когда нужно добавить/убрать callback,
/// а система уже внутри dispatch. Записывается через sub_1403A0A90.
/// Применяется через FlushPendingFunctionOps (sub_1403B4AD0).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PendingFunctionOp {
    /// Целевой event_id.
    pub event_id: i32, // +0x00
    pub _pad_004: i32, // +0x04

    pub callback_object: *const c_void,   // +0x08
    pub callback_function: *const c_void, // +0x10
    pub priority: i32,                    // +0x18

    /// Код операции:
    /// - ≠ 0 -> добавить callback
    /// - 0 -> убрать callback
    pub op_code: u8, // +0x1C
    pub _pad_01d: [u8; 3], // +0x1D

    pub float_param: f32, // +0x20
    pub config_mask: i32, // +0x24
    pub int_param: i32,   // +0x28
    pub _pad_02c: i32,    // +0x2C
}

/// Контекст dispatch'а — второй аргумент callback'а.
///
/// `void __fastcall Callback(void* callback_object, DispatchContext* ctx)`
///
/// Структура восстановлена частично по sub_1403A16A0.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DispatchContext {
    pub vtable: *const c_void, // +0x00

    /// Временный отсортированный список callback'ов для текущего события.
    pub sorted_callback_list: *mut c_void, // +0x08

    /// Пользовательские данные (5-й аргумент DispatchEventsInternal).
    pub user_data: *mut c_void, // +0x10

    /// Рабочие счётчики dispatch'а.
    pub current_index: u32, // +0x18
    pub call_depth: u32, // +0x1C

    /// Текущий обрабатываемый CallbackFunctionEntry.
    pub current_function_entry: *mut CallbackFunctionEntry, // +0x20

    pub reserved0: u32, // +0x28
    pub reserved1: u32, // +0x2C

    /// Таймер/профилировщик (может быть NULL).
    pub timer: *mut DispatchTimer, // +0x30

    /// Дополнительный интерфейс / fallback object.
    pub callback_iface: *mut c_void, // +0x38
}

/// Таймер / статистика dispatch'а.
///
/// Восстановлен частично по sub_1403A1A00.
/// Используется для профилирования callback-вызовов.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DispatchTimer {
    pub base_value: f32, // +0x00
    /// Активен ли таймер в текущем dispatch.
    pub is_active: u8, // +0x04
    pub _pad_005: [u8; 3], // +0x05
    /// Счётчик вызовов/кадров.
    pub frame_count: u32, // +0x08
    pub unknown_0c: u32, // +0x0C
    /// Разница QueryPerformanceCounter.
    pub perf_delta: i64, // +0x10
    /// Базовое значение QueryPerformanceCounter.
    pub perf_base: i64, // +0x18
    pub unknown_20: u32, // +0x20
    /// Накопительные float-поля для dt/cap групп.
    pub accum_add: f32, // +0x24
    pub accum_sub: f32,  // +0x28
    pub unknown_2c: u32, // +0x2C
    pub dt_level0: f32,  // +0x30
    pub dt_level1: f32,  // +0x34
    pub dt_level2: f32,  // +0x38
    pub dt_level3: f32,  // +0x3C
    pub dt_level4: f32,  // +0x40
    pub cap0: f32,       // +0x44
    pub cap1: f32,       // +0x48
    pub cap2: f32,       // +0x4C
    pub cap3: f32,       // +0x50
    pub cap4: f32,       // +0x54
    pub prev_cap0: f32,  // +0x58
    pub prev_cap1: f32,  // +0x5C
    pub prev_cap2: f32,  // +0x60
    pub prev_cap3: f32,  // +0x64
    pub prev_cap4: f32,  // +0x68
}

// =============================================================================
//  Compile-time проверки layout'ов
// =============================================================================

assert_layout!(GameCallbackManager, size = 0x40, {
    entries              == 0x08,
    pending              == 0x20,
    current_dispatch_ctx == 0x38,
});

assert_layout!(CallbackEventDesc, size = 0x40, {
    event_id    == 0x20,
    in_dispatch == 0x24,
    funcs       == 0x28,
});

assert_layout!(CallbackFunctionEntry, size = 0x28, {
    callback_object   == 0x00,
    callback_function == 0x08,
    priority          == 0x10,
    flags             == 0x14,
    config_mask       == 0x18,
    float_param       == 0x1C,
    int_param         == 0x20,
});

assert_layout!(PendingFunctionOp, size = 0x30, {
    event_id          == 0x00,
    callback_object   == 0x08,
    callback_function == 0x10,
    priority          == 0x18,
    op_code           == 0x1C,
    float_param       == 0x20,
    config_mask       == 0x24,
    int_param         == 0x28,
});

assert_layout!(DispatchContext, size = 0x40, {
    sorted_callback_list   == 0x08,
    user_data              == 0x10,
    current_index          == 0x18,
    current_function_entry == 0x20,
    timer                  == 0x30,
    callback_iface         == 0x38,
});

assert_field_offsets!(DispatchTimer {
    is_active   == 0x04,
    frame_count == 0x08,
    perf_delta  == 0x10,
    perf_base   == 0x18,
    accum_add   == 0x24,
    accum_sub   == 0x28,
    dt_level0   == 0x30,
    cap0        == 0x44,
    prev_cap0   == 0x58,
    prev_cap4   == 0x68,
});

impl GameCallbackManager {
    /// Количество зарегистрированных типов событий.
    ///
    /// В стандартной сессии FreeRide — 39 типов.
    #[inline]
    pub fn event_count(&self) -> usize {
        self.entries.len()
    }

    /// Слайс зарегистрированных событий.
    ///
    /// # Safety
    ///
    /// Вызывать из game thread. Массив может быть изменён движком.
    #[inline]
    pub unsafe fn events(&self) -> &[CallbackEventDesc] {
        unsafe { self.entries.as_slice() }
    }

    /// Сейчас идёт dispatch.
    #[inline]
    pub fn is_dispatching(&self) -> bool {
        !self.current_dispatch_ctx.is_null()
    }
}
