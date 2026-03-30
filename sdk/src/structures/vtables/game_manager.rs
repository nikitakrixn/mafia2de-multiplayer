//! Типизированная VTable для `C_Game` (GameManager).
//!
//! VTable расположена в `.rdata` по адресу `off_14186F450` и содержит
//! **25 виртуальных методов**.
//!
//! Класс идентифицируется строкой `"C_Game"` через слот `[2]`.
//!
//! ## Что уже подтверждено
//!
//! По коду и строкам внутри функций уверенно установлены:
//! - загрузка/выгрузка мира (`Open`, `Close`),
//! - инициализация/деинициализация (`GameInit`, `GameDone`),
//! - глобальный tick,
//! - работа с 4 entity-слотами,
//! - inline MissionManager,
//! - удаление/активация entity.
//!
//! ## Карта слотов
//!
//! | Слот | Функция | Назначение |
//! |:----:|:--------|:-----------|
//! | 0 | `Dtor` | Деструктор `C_Game` |
//! | 1 | `GetModuleID` | Возвращает `7` |
//! | 2 | `GetClassName` | Возвращает `"C_Game"` |
//! | 3 | NOP | Заглушка |
//! | 4 | `GetFixedTimeStep` | Возвращает `0.005f` |
//! | 5 | `Open` | Загрузка игрового мира |
//! | 6 | `Close` | Выгрузка игрового мира |
//! | 7 | `IsLoaded` | Проверка бита loaded |
//! | 8 | `GameInit` | Пост-загрузочная инициализация |
//! | 9 | `GameDone` | Предвыгрузочная деинициализация |
//! | 10 | `IsInitialized` | Проверка бита initialized |
//! | 11 | `IsFlagBit3` | Проверка бита 3 |
//! | 12 | NOP | Заглушка |
//! | 13 | `SetSuspended` | Установка/сброс бита suspended |
//! | 14 | `IsSuspended` | Проверка бита suspended |
//! | 15 | `Tick` | Главный цикл обновления |
//! | 16 | `InvalidateEntity` | Очистка entity-слотов |
//! | 17 | `GetTickCounter` | Возвращает `game_tick_counter` |
//! | 18 | `GetMissionManager` | Возвращает `this + 0x58` |
//! | 19 | `SetFlagBit4` | Устанавливает bit 4 |
//! | 20 | `GetEntityFromIndex` | Чтение `entity_slots[i]` |
//! | 21 | `SetEntityAtIndex` | Запись `entity_slots[i]` |
//! | 22 | `GetGamePhase` | Возвращает байт по `this + 0x48` |
//! | 23 | `ActivateEntity` | Добавление entity в таблицы |
//! | 24 | `OnEntityDeleted` | Удаление entity из таблиц |

use std::ffi::{c_char, c_void};

// -----------------------------------------------------------------------------
// ABI-safe aliases
// -----------------------------------------------------------------------------

type FnDtor = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnThisU32 = unsafe extern "system" fn(this: *const c_void) -> u32;
type FnThisBool = unsafe extern "system" fn(this: *const c_void) -> bool;
type FnThisF32 = unsafe extern "system" fn(this: *const c_void) -> f32;
type FnThisVoid = unsafe extern "system" fn(this: *mut c_void);
type FnThisConstVoid = unsafe extern "system" fn(this: *const c_void) -> *mut c_void;
type FnThisU8 = unsafe extern "system" fn(this: *const c_void) -> u8;
type FnOpen = unsafe extern "system" fn(this: *mut c_void, game_name: *const c_char) -> bool;
type FnClose = unsafe extern "system" fn(this: *mut c_void) -> bool;
type FnSetBool = unsafe extern "system" fn(this: *mut c_void, value: bool);
type FnTick = unsafe extern "system" fn(this: *mut c_void, context: *const c_void);
type FnInvalidateEntity = unsafe extern "system" fn(this: *mut c_void, entity: *const c_void);
type FnGetEntityFromIndex =
    unsafe extern "system" fn(this: *const c_void, index: u32) -> *mut c_void;
type FnSetEntityAtIndex =
    unsafe extern "system" fn(this: *mut c_void, entity: *mut c_void, index: u32);
type FnActivateEntity =
    unsafe extern "system" fn(this: *mut c_void, entity: *mut c_void, activate: bool);
type FnOnEntityDeleted =
    unsafe extern "system" fn(this: *mut c_void, entity: *mut c_void);

/// VTable для `C_Game`.
///
/// Все слоты в этом описании соответствуют реальному layout в `.rdata`.
#[repr(C)]
pub struct CGameVTable {
    // =========================================================================
    //  Жизненный цикл (0–4)
    // =========================================================================

    /// `[0]` Деструктор `C_Game`.
    ///
    /// Очищает:
    /// - pending-векторы,
    /// - обе hash table entity,
    /// - inline MissionManager,
    /// - служебные буферы и хвостовые контейнеры.
    pub dtor: FnDtor,

    /// `[1]` Возвращает module id = `7`.
    pub get_module_id: FnThisU32,

    /// `[2]` Возвращает строку `"C_Game"`.
    pub get_class_name: unsafe extern "system" fn(this: *const c_void) -> *const c_char,

    /// `[3]` Заглушка.
    pub _slot_03_nop: usize,

    /// `[4]` Возвращает фиксированный шаг симуляции: `0.005f`.
    ///
    /// Это соответствует 200 Гц.
    pub get_fixed_time_step: FnThisF32,

    // =========================================================================
    //  Загрузка / выгрузка мира (5–7)
    // =========================================================================

    /// `[5]` Загрузка игрового мира.
    ///
    /// Основные действия:
    /// - парсит `/games/{name}.bin`,
    /// - загружает SDS-ресурсы,
    /// - настраивает пути `sds_path_*`,
    /// - подготавливает ScriptContext,
    /// - устанавливает флаг loaded (bit 1).
    pub open: FnOpen,

    /// `[6]` Выгрузка игрового мира.
    ///
    /// Основные действия:
    /// - обнуляет `entity_slots`,
    /// - очищает ScriptContext,
    /// - очищает PlayerModelManager,
    /// - запускает выгрузку SDS,
    /// - сбрасывает флаг loaded (bit 1).
    pub close: FnClose,

    /// `[7]` Проверка состояния loaded.
    ///
    /// Эквивалентно: `(game_state_flags & 2) != 0`.
    pub is_loaded: FnThisBool,

    // =========================================================================
    //  Инициализация / деинициализация (8–12)
    // =========================================================================

    /// `[8]` Пост-загрузочная инициализация мира.
    ///
    /// Основные действия:
    /// - спавнит player model,
    /// - активирует entity из базы,
    /// - сбрасывает `game_tick_counter`,
    /// - устанавливает бит initialized (bit 0).
    pub game_init: FnThisVoid,

    /// `[9]` Деинициализация перед выгрузкой мира.
    ///
    /// Основные действия:
    /// - деактивирует entity,
    /// - очищает `entity_slots`,
    /// - сбрасывает initialized (bit 0).
    pub game_done: FnThisVoid,

    /// `[10]` Проверка состояния initialized.
    ///
    /// Эквивалентно: `(game_state_flags & 1) != 0`.
    pub is_initialized: FnThisBool,

    /// `[11]` Проверка бита 3 в `game_state_flags`.
    pub is_flag_bit3: FnThisBool,

    /// `[12]` Заглушка.
    ///
    /// В текущем DE-билде логика не реализована.
    pub _slot_12_nop: usize,

    // =========================================================================
    //  Управление состоянием (13–14)
    // =========================================================================

    /// `[13]` Установить / снять suspended-флаг.
    ///
    /// Управляет bit 2 в `game_state_flags`.
    pub set_suspended: FnSetBool,

    /// `[14]` Проверка suspended-флага.
    ///
    /// Эквивалентно: `(game_state_flags & 4) != 0`.
    /// Важно: это НЕ флаг обычной pause menu.
    /// В логах при событии Game Paused (34) значение остаётся false.
    pub is_suspended: FnThisBool,

    // =========================================================================
    //  Основной update (15–16)
    // =========================================================================

    /// `[15]` Главный игровой тик.
    ///
    /// Основные действия:
    /// - увеличивает `game_tick_counter`,
    /// - обходит entity из hash table,
    /// - обновляет streaming / deferred entity logic,
    /// - в конце обрабатывает pending add/remove vectors.
    pub tick: FnTick,

    /// `[16]` Инвалидация entity в `entity_slots`.
    ///
    /// Если переданный entity совпадает с одним из 4 слотов,
    /// соответствующий слот обнуляется.
    pub invalidate_entity: FnInvalidateEntity,

    // =========================================================================
    //  Запросы (17–22)
    // =========================================================================

    /// `[17]` Получить глобальный счётчик тиков.
    ///
    /// Эквивалентно чтению `game_tick_counter` по `+0x0C`.
    pub get_tick_counter: FnThisU32,

    /// `[18]` Получить указатель на inline MissionManager.
    ///
    /// Фактически возвращает `this + 0x58`.
    pub get_mission_manager: FnThisConstVoid,

    /// `[19]` Установить bit 4 в `game_state_flags`.
    pub set_flag_bit4: FnThisVoid,

    /// `[20]` Получить entity по индексу слота.
    ///
    /// Эквивалентно: `entity_slots[index]`.
    pub get_entity_from_index: FnGetEntityFromIndex,

    /// `[21]` Записать entity в slot по индексу.
    ///
    /// Эквивалентно: `entity_slots[index] = entity`.
    pub set_entity_at_index: FnSetEntityAtIndex,

    /// `[22]` Возвращает внутренний байтовый state по `this + 0x48`.
    ///
    /// Рабочее имя пока условное (`game_phase`).
    /// В обычной игре и в меню наблюдается значение `0`.
    pub get_game_phase: FnThisU8,

    // =========================================================================
    //  Работа с entity tables (23–24)
    // =========================================================================

    /// `[23]` Активировать entity.
    ///
    /// Если `tick_in_progress == 0`:
    /// - entity добавляется напрямую в hash table
    ///
    /// Если `tick_in_progress != 0`:
    /// - операция откладывается в `pending_add_entities`
    pub activate_entity: FnActivateEntity,

    /// `[24]` Обработка удаления entity.
    ///
    /// Если `tick_in_progress == 0`:
    /// - entity удаляется напрямую из hash table
    ///
    /// Если `tick_in_progress != 0`:
    /// - операция откладывается в `pending_remove_entities`
    pub on_entity_deleted: FnOnEntityDeleted,
}

const _: () = {
    assert!(std::mem::size_of::<CGameVTable>() == 25 * 8);

    assert!(std::mem::offset_of!(CGameVTable, dtor) == 0);
    assert!(std::mem::offset_of!(CGameVTable, get_class_name) == 2 * 8);
    assert!(std::mem::offset_of!(CGameVTable, open) == 5 * 8);
    assert!(std::mem::offset_of!(CGameVTable, close) == 6 * 8);
    assert!(std::mem::offset_of!(CGameVTable, is_loaded) == 7 * 8);
    assert!(std::mem::offset_of!(CGameVTable, game_init) == 8 * 8);
    assert!(std::mem::offset_of!(CGameVTable, game_done) == 9 * 8);
    assert!(std::mem::offset_of!(CGameVTable, is_initialized) == 10 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_suspended) == 13 * 8);
    assert!(std::mem::offset_of!(CGameVTable, tick) == 15 * 8);
    assert!(std::mem::offset_of!(CGameVTable, invalidate_entity) == 16 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_tick_counter) == 17 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_mission_manager) == 18 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_flag_bit4) == 19 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_entity_from_index) == 20 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_entity_at_index) == 21 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_game_phase) == 22 * 8);
    assert!(std::mem::offset_of!(CGameVTable, activate_entity) == 23 * 8);
    assert!(std::mem::offset_of!(CGameVTable, on_entity_deleted) == 24 * 8);
};