use std::ffi::c_void;

/// Глобальный менеджер callback/event системы игры.
///
/// Глобал:
/// `crate::addresses::globals::GAME_CALLBACK_MANAGER`
///
/// Основные обязанности:
/// - хранит все зарегистрированные типы событий (`CallbackEventDesc`)
/// - хранит callback functions, подписанные на эти события
/// - хранит deferred/pending операции add/remove на время dispatch
/// - публикует указатель на текущий `DispatchContext`
///
/// Внутри:
/// - `std::vector<CallbackEventDesc>` по `+0x08/+0x10/+0x18`
/// - `std::vector<PendingFunctionOp>` по `+0x20/+0x28/+0x30`
/// - `+0x38` — указатель на активный `DispatchContext` во время callback dispatch
///
/// Подтверждено по:
/// - `sub_1403A08F0`  — RegisterEventType
/// - `sub_1403A06D0`  — RegisterFunction
/// - `sub_1403A15E0`  — FireEventById
/// - `sub_1403A16A0`  — DispatchSingleEventByIndex
/// - `sub_1403A1A00`  — DispatchEventsInternal
/// - `sub_1403A55A0`  — UnregisterFunction
/// - `sub_1403A57E0`  — UnregisterFunctionsByObject
/// - `sub_1403A64F0`  — SetFunctionFlags
/// - `sub_1403B4AD0`  — FlushPendingFunctionOps
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GameCallbackManager {
    pub vtable: *const c_void,                    // +0x00

    /// std::vector<EventDesc>::begin
    pub entries_begin: *mut CallbackEventDesc,    // +0x08
    /// std::vector<EventDesc>::end
    pub entries_end: *mut CallbackEventDesc,      // +0x10
    /// std::vector<EventDesc>::capacity
    pub entries_capacity: *mut CallbackEventDesc, // +0x18

    /// std::vector<PendingFunctionOp>::begin
    ///
    /// Используется для отложенных операций регистрации/изменения callback'ов,
    /// когда система уже находится внутри dispatch.
    pub pending_begin: *mut PendingFunctionOp,    // +0x20
    /// std::vector<PendingFunctionOp>::end
    pub pending_end: *mut PendingFunctionOp,      // +0x28
    /// std::vector<PendingFunctionOp>::capacity
    pub pending_capacity: *mut PendingFunctionOp, // +0x30

    /// Текущий активный context во время dispatch.
    ///
    /// Во время простоя обычно NULL.
    /// Во время dispatch указывает на локальный `DispatchContext`.
    pub current_dispatch_ctx: *mut DispatchContext, // +0x38
}

/// Один зарегистрированный тип события.
///
/// Примеры:
/// - "Game Tick"
/// - "Game Tick Always"
/// - "Game Render"
///
/// Размер: 0x40.
///
/// Подтверждено по `sub_1403A08F0` и `sub_1403A1A00`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallbackEventDesc {
    /// Имя события, null-terminated, максимум 0x1F символов.
    pub name: [u8; 0x20],                         // +0x00

    /// Уникальный ID события.
    ///
    /// Примеры:
    /// - 3  = Game Tick
    /// - 5  = Game Tick Always
    /// - 7  = Game Render
    pub event_id: i32,                            // +0x20

    /// Временный reentrancy/busy lock для события.
    ///
    /// ВАЖНО:
    /// это не "тип" и не постоянный mode события.
    /// Поле выставляется на время dispatch и сбрасывается после него.
    ///
    /// Наблюдение:
    /// - 0 = idle
    /// - 1 = event is currently being dispatched
    ///
    /// Подтверждено:
    /// - `sub_1403A16A0` выставляет это поле в 1 перед вызовом callback'ов
    /// - после dispatch поле снова сбрасывается в 0
    pub in_dispatch: i32,                         // +0x24

    /// std::vector<CallbackFunctionEntry>::begin
    pub funcs_begin: *mut CallbackFunctionEntry,  // +0x28
    /// std::vector<CallbackFunctionEntry>::end
    pub funcs_end: *mut CallbackFunctionEntry,    // +0x30
    /// std::vector<CallbackFunctionEntry>::capacity
    pub funcs_capacity: *mut CallbackFunctionEntry,// +0x38
}

impl CallbackEventDesc {
    pub fn name_string(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(self.name.len());
        String::from_utf8_lossy(&self.name[..end]).into_owned()
    }
}

/// Один зарегистрированный callback внутри конкретного события.
///
/// Размер: 0x28.
///
/// Подтверждено по `sub_1403A06D0` и `sub_1403A1A00`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallbackFunctionEntry {
    /// Первый аргумент callback'а.
    ///
    /// Во время реального вызова:
    /// `RCX = callback_object`
    pub callback_object: *const c_void,           // +0x00

    /// Указатель на функцию callback'а.
    ///
    /// Во время реального вызова:
    /// `call [callback_function]`
    ///
    /// Предполагаемый prototype:
    /// `void __fastcall fn(void* callback_object, DispatchContext* ctx)`
    pub callback_function: *const c_void,         // +0x08

    /// Приоритет callback'а.
    ///
    /// Чем меньше число, тем раньше вызывается callback
    /// в merged dispatch списке.
    pub priority: i32,                            // +0x10

    /// Runtime flags callback'а.
    ///
    /// Подтверждено:
    /// - bit0 = active/enabled
    ///
    /// Вероятно:
    /// - bit1 = continue / re-dispatch chaining flag
    /// - bit2 = abort current dispatch
    ///
    /// Dispatcher перед вызовом callback'а делает:
    /// `flags &= 0xF9`
    /// то есть очищает bit1 и bit2.
    ///
    /// После вызова callback может выставить их обратно.
    pub flags: u8,                                // +0x14
    pub _pad_015: [u8; 3],                        // +0x15

    /// Конфигурационная маска/флаги регистрации.
    ///
    /// Часто содержит значения вроде `0x40F00000`, `0x7500000` и т.д.
    pub config_mask: i32,                         // +0x18

    /// Float-параметр callback'а.
    ///
    /// Часто встречаются:
    /// - `0.005`
    /// - `0.003`
    /// - `0.0001`
    /// - `-1.0`
    pub float_param: f32,                         // +0x1C

    /// Дополнительный integer parameter.
    ///
    /// Иногда 0, иногда небольшие управляющие значения.
    pub int_param: i32,                           // +0x20

    /// Пока не расшифровано.
    pub reserved: i32,                            // +0x24
}

/// Отложенная операция над callback-системой.
///
/// Размер: 0x30.
///
/// Используется, когда система уже находится внутри dispatch и нельзя
/// безопасно модифицировать список callback'ов напрямую.
///
/// Заполняется через `sub_1403A0A90`.
///
/// Это НЕ "immediate entry", а наоборот deferred/pending operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PendingFunctionOp {
    /// Целевой event_id.
    pub event_id: i32,                            // +0x00
    pub _pad_004: i32,                            // +0x04

    /// callback_object будущей записи.
    pub callback_object: *const c_void,           // +0x08

    /// callback_function будущей записи.
    pub callback_function: *const c_void,         // +0x10

    /// priority будущей записи.
    pub priority: i32,                            // +0x18

    /// Код отложенной операции.
    ///
    /// Подтверждено по `sub_1403B4AD0`:
    /// - `op_code != 0` → deferred register / add path
    /// - `op_code == 0` → deferred unregister / remove path
    ///
    /// То есть это не просто "flags", а именно discriminator типа операции.
    pub op_code: u8,                              // +0x1C
    pub _pad_01d: [u8; 3],                        // +0x1D

    /// float_param будущей записи.
    pub float_param: f32,                         // +0x20

    /// config_mask будущей записи.
    pub config_mask: i32,                         // +0x24

    /// int_param будущей записи.
    pub int_param: i32,                           // +0x28

    pub _pad_02c: i32,                            // +0x2C
}

/// Контекст dispatch'а, передаваемый callback'у вторым аргументом.
///
/// Предполагаемый runtime prototype callback'а:
///
/// `void __fastcall Callback(void* callback_object, DispatchContext* ctx)`
///
/// ВАЖНО:
/// структура всё ещё восстановлена частично.
/// Названия полей отражают наблюдаемое использование в dispatcher'е,
/// но не гарантируют полное оригинальное назначение.
///
/// Основной источник:
/// - `sub_1403A16A0`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DispatchContext {
    /// VTable/iface dispatch context.
    pub vtable: *const c_void,                    // +0x00

    /// Указатель на временный список `CallbackFunctionEntry*`,
    /// собранный dispatcher'ом для одного fired event.
    pub sorted_callback_list: *mut c_void,        // +0x08

    /// Пользовательский/внешний аргумент dispatcher'а.
    ///
    /// Передаётся в `sub_1403A1A00` пятым аргументом.
    pub user_data: *mut c_void,                   // +0x10

    /// Runtime counters, используемые dispatcher'ом во время обхода callback'ов.
    ///
    /// Точная семантика пары ещё не до конца подтверждена,
    /// но это не "постоянные" поля, а рабочее состояние dispatch'а.
    pub current_index: u32,                       // +0x18
    pub call_depth: u32,                          // +0x1C

    /// Указатель на текущий обрабатываемый `CallbackFunctionEntry`.
    ///
    /// Устанавливается перед вызовом callback'а и очищается после него.
    pub current_function_entry: *mut CallbackFunctionEntry, // +0x20

    /// Пока не расшифровано. Инициализируется нулём.
    pub reserved0: u32,                           // +0x28
    pub reserved1: u32,                           // +0x2C

    /// Таймер/профилировщик dispatch'а.
    ///
    /// Может быть NULL.
    pub timer: *mut DispatchTimer,                // +0x30

    /// Дополнительный интерфейс / fallback object для callback dispatch path.
    ///
    /// Если внешний interface pointer не передан, dispatcher подставляет
    /// локальный fallback object.
    pub callback_iface: *mut c_void,              // +0x38
}

/// Таймер / статистика dispatch'а.
///
/// Восстановлен частично по `sub_1403A1A00`.
///
/// Эта структура уже полезна как layout, но её семантика пока не считается
/// полностью финальной. Поля названы по наблюдаемому использованию.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DispatchTimer {
    /// Базовый накопитель.
    pub base_value: f32,                          // +0x00

    /// Активен ли таймер в текущем dispatch.
    pub is_active: u8,                            // +0x04
    pub _pad_005: [u8; 3],                        // +0x05

    /// Счётчик вызовов/кадров.
    pub frame_count: u32,                         // +0x08

    /// Не расшифровано.
    pub unknown_0c: u32,                          // +0x0C

    /// Последняя разница QueryPerformanceCounter.
    pub perf_delta: i64,                          // +0x10

    /// Базовое значение QueryPerformanceCounter.
    pub perf_base: i64,                           // +0x18

    /// Не расшифровано.
    pub unknown_20: u32,                          // +0x20

    /// Накопительные float-поля, которые dispatcher перераспределяет
    /// по dt/cap группам.
    pub accum_add: f32,                           // +0x24
    pub accum_sub: f32,                           // +0x28

    /// Не расшифровано.
    pub unknown_2c: u32,                          // +0x2C

    pub dt_level0: f32,                           // +0x30
    pub dt_level1: f32,                           // +0x34
    pub dt_level2: f32,                           // +0x38
    pub dt_level3: f32,                           // +0x3C
    pub dt_level4: f32,                           // +0x40

    pub cap0: f32,                                // +0x44
    pub cap1: f32,                                // +0x48
    pub cap2: f32,                                // +0x4C
    pub cap3: f32,                                // +0x50
    pub cap4: f32,                                // +0x54

    pub prev_cap0: f32,                           // +0x58
    pub prev_cap1: f32,                           // +0x5C
    pub prev_cap2: f32,                           // +0x60
    pub prev_cap3: f32,                           // +0x64
    pub prev_cap4: f32,                           // +0x68
}

const _: () = {
    assert!(std::mem::size_of::<GameCallbackManager>() == 0x40);
    assert!(std::mem::offset_of!(GameCallbackManager, entries_begin) == 0x08);
    assert!(std::mem::offset_of!(GameCallbackManager, pending_begin) == 0x20);
    assert!(std::mem::offset_of!(GameCallbackManager, current_dispatch_ctx) == 0x38);

    assert!(std::mem::size_of::<CallbackEventDesc>() == 0x40);
    assert!(std::mem::offset_of!(CallbackEventDesc, event_id) == 0x20);
    assert!(std::mem::offset_of!(CallbackEventDesc, in_dispatch) == 0x24);
    assert!(std::mem::offset_of!(CallbackEventDesc, funcs_begin) == 0x28);

    assert!(std::mem::size_of::<CallbackFunctionEntry>() == 0x28);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, callback_object) == 0x00);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, callback_function) == 0x08);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, priority) == 0x10);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, flags) == 0x14);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, config_mask) == 0x18);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, float_param) == 0x1C);
    assert!(std::mem::offset_of!(CallbackFunctionEntry, int_param) == 0x20);

    assert!(std::mem::size_of::<PendingFunctionOp>() == 0x30);
    assert!(std::mem::offset_of!(PendingFunctionOp, event_id) == 0x00);
    assert!(std::mem::offset_of!(PendingFunctionOp, callback_object) == 0x08);
    assert!(std::mem::offset_of!(PendingFunctionOp, callback_function) == 0x10);
    assert!(std::mem::offset_of!(PendingFunctionOp, priority) == 0x18);
    assert!(std::mem::offset_of!(PendingFunctionOp, op_code) == 0x1C);
    assert!(std::mem::offset_of!(PendingFunctionOp, float_param) == 0x20);
    assert!(std::mem::offset_of!(PendingFunctionOp, config_mask) == 0x24);
    assert!(std::mem::offset_of!(PendingFunctionOp, int_param) == 0x28);

    assert!(std::mem::size_of::<DispatchContext>() == 0x40);
    assert!(std::mem::offset_of!(DispatchContext, sorted_callback_list) == 0x08);
    assert!(std::mem::offset_of!(DispatchContext, user_data) == 0x10);
    assert!(std::mem::offset_of!(DispatchContext, current_index) == 0x18);
    assert!(std::mem::offset_of!(DispatchContext, current_function_entry) == 0x20);
    assert!(std::mem::offset_of!(DispatchContext, timer) == 0x30);
    assert!(std::mem::offset_of!(DispatchContext, callback_iface) == 0x38);

    assert!(std::mem::offset_of!(DispatchTimer, is_active) == 0x04);
    assert!(std::mem::offset_of!(DispatchTimer, frame_count) == 0x08);
    assert!(std::mem::offset_of!(DispatchTimer, perf_delta) == 0x10);
    assert!(std::mem::offset_of!(DispatchTimer, perf_base) == 0x18);
    assert!(std::mem::offset_of!(DispatchTimer, accum_add) == 0x24);
    assert!(std::mem::offset_of!(DispatchTimer, accum_sub) == 0x28);
    assert!(std::mem::offset_of!(DispatchTimer, dt_level0) == 0x30);
    assert!(std::mem::offset_of!(DispatchTimer, cap0) == 0x44);
    assert!(std::mem::offset_of!(DispatchTimer, prev_cap0) == 0x58);
    assert!(std::mem::offset_of!(DispatchTimer, prev_cap4) == 0x68);
};