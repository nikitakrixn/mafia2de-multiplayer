//! VTable `C_Game` — типизированное описание 25 виртуальных методов.
//!
//! Адрес в `.rdata`: `M2DE_VT_CGame` (`0x14186F450`).
//! Идентификатор класса: строка `"C_Game"` через слот `[2]`.
//!
//! ## Карта слотов
//!
//! | Слот | Имя | Описание |
//! |:----:|:----|:---------|
//! | 0 | `dtor` | Деструктор |
//! | 1 | `get_module_id` | Возвращает `7` |
//! | 2 | `get_class_name` | Возвращает `"C_Game"` |
//! | 3 | *(nop)* | Заглушка |
//! | 4 | `get_fixed_time_step` | Возвращает `0.005f` (200 Гц) |
//! | 5 | `open` | Загрузка мира из `.bin` |
//! | 6 | `close` | Выгрузка мира |
//! | 7 | `is_loaded` | Проверка bit 1 |
//! | 8 | `game_init` | Пост-загрузочная инициализация |
//! | 9 | `game_done` | Деинициализация перед выгрузкой |
//! | 10 | `is_initialized` | Проверка bit 0 |
//! | 11 | `is_flag_bit3` | Проверка bit 3 |
//! | 12 | *(nop)* | Заглушка |
//! | 13 | `set_suspended` | Управление bit 2 |
//! | 14 | `is_suspended` | Проверка bit 2 |
//! | 15 | `tick` | Главный игровой тик |
//! | 16 | `invalidate_entity` | Обнуление entity-слота |
//! | 17 | `get_tick_counter` | Чтение `game_tick_counter` |
//! | 18 | `get_actors_pack` | Возвращает `this + 0x58` |
//! | 19 | `set_flag_bit4` | Установка bit 4 |
//! | 20 | `get_entity_from_index` | Чтение `entity_slots[i]` |
//! | 21 | `set_entity_at_index` | Запись `entity_slots[i]` |
//! | 22 | `get_game_phase` | Байтовый state по `this + 0x48` |
//! | 23 | `activate_entity` | Добавление entity в таблицы |
//! | 24 | `on_entity_deleted` | Удаление entity из таблиц |

use std::ffi::{c_char, c_void};

// -----------------------------------------------------------------------------
// Типы функций
// -----------------------------------------------------------------------------

type FnDtor = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnThisU32 = unsafe extern "system" fn(this: *const c_void) -> u32;
type FnThisBool = unsafe extern "system" fn(this: *const c_void) -> bool;
type FnThisF32 = unsafe extern "system" fn(this: *const c_void) -> f32;
type FnThisU8 = unsafe extern "system" fn(this: *const c_void) -> u8;
type FnThisVoid = unsafe extern "system" fn(this: *mut c_void);
type FnThisConstVoid = unsafe extern "system" fn(this: *const c_void) -> *mut c_void;
type FnOpen = unsafe extern "system" fn(this: *mut c_void, name: *const c_char) -> bool;
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
type FnOnEntityDeleted = unsafe extern "system" fn(this: *mut c_void, entity: *mut c_void);

/// VTable `C_Game` — 25 слотов, `M2DE_VT_CGame` @ `0x14186F450`.
#[repr(C)]
pub struct CGameVTable {
    // =========================================================================
    //  Жизненный цикл (0–4)
    // =========================================================================
    /// `[0]` Деструктор `C_Game`.
    pub dtor: FnDtor,

    /// `[1]` Возвращает module id = `7`.
    pub get_module_id: FnThisU32,

    /// `[2]` Возвращает строку `"C_Game"`.
    pub get_class_name: unsafe extern "system" fn(this: *const c_void) -> *const c_char,

    /// `[3]` Заглушка (`nullsub`).
    pub _slot_03_nop: usize,

    /// `[4]` Фиксированный шаг симуляции: `0.005f` (200 Гц).
    pub get_fixed_time_step: FnThisF32,

    // =========================================================================
    //  Загрузка / выгрузка мира (5–7)
    // =========================================================================
    /// `[5]` Загрузка игрового мира.
    ///
    /// - Парсит `/games/{name}.bin`
    /// - Загружает SDS-ресурсы
    /// - Заполняет `sds_path_*`
    /// - Устанавливает bit 1 (`loaded`)
    pub open: FnOpen,

    /// `[6]` Выгрузка игрового мира.
    ///
    /// - Обнуляет `entity_slots`
    /// - Очищает ScriptContext и PlayerModelManager
    /// - Запускает выгрузку SDS
    /// - Сбрасывает bit 1 (`loaded`)
    pub close: FnClose,

    /// `[7]` Проверка: мир загружен.
    ///
    /// Эквивалентно `(game_state_flags & 2) != 0`.
    pub is_loaded: FnThisBool,

    // =========================================================================
    //  Инициализация / деинициализация (8–12)
    // =========================================================================
    /// `[8]` Пост-загрузочная инициализация.
    ///
    /// - Спавнит player model
    /// - Активирует entity из базы
    /// - Сбрасывает `game_tick_counter`
    /// - Устанавливает bit 0 (`initialized`)
    pub game_init: FnThisVoid,

    /// `[9]` Деинициализация перед выгрузкой.
    ///
    /// - Деактивирует entity
    /// - Очищает `entity_slots`
    /// - Сбрасывает bit 0 (`initialized`)
    pub game_done: FnThisVoid,

    /// `[10]` Проверка: мир инициализирован.
    ///
    /// Эквивалентно `(game_state_flags & 1) != 0`.
    pub is_initialized: FnThisBool,

    /// `[11]` Проверка bit 3 в `game_state_flags`.
    pub is_flag_bit3: FnThisBool,

    /// `[12]` Заглушка.
    pub _slot_12_nop: usize,

    // =========================================================================
    //  Управление состоянием (13–14)
    // =========================================================================
    /// `[13]` Установить / снять suspended-флаг (bit 2).
    pub set_suspended: FnSetBool,

    /// `[14]` Проверка suspended-флага.
    ///
    /// Эквивалентно `(game_state_flags & 4) != 0`.
    pub is_suspended: FnThisBool,

    // =========================================================================
    //  Основной update (15–16)
    // =========================================================================
    /// `[15]` Главный игровой тик.
    ///
    /// - Увеличивает `game_tick_counter`
    /// - Обходит entity из hash table
    /// - Обрабатывает pending add/remove в конце
    pub tick: FnTick,

    /// `[16]` Инвалидация entity-слота.
    ///
    /// Если переданный entity совпадает с одним из 4 слотов — обнуляет его.
    pub invalidate_entity: FnInvalidateEntity,

    // =========================================================================
    //  Запросы (17–22)
    // =========================================================================
    /// `[17]` Чтение `game_tick_counter` (`+0x0C`).
    pub get_tick_counter: FnThisU32,

    /// `[18]` Указатель на inline `C_ActorsPack`.
    ///
    /// Фактически возвращает `this + 0x58`.
    pub get_actors_pack: FnThisConstVoid,

    /// `[19]` Установить bit 4 в `game_state_flags`.
    pub set_flag_bit4: FnThisVoid,

    /// `[20]` Чтение `entity_slots[index]`.
    pub get_entity_from_index: FnGetEntityFromIndex,

    /// `[21]` Запись `entity_slots[index] = entity`.
    pub set_entity_at_index: FnSetEntityAtIndex,

    /// `[22]` Игровая фаза — читает `this + 0x48`.
    ///
    /// Эквивалентно прямому чтению поля `game_phase`.
    pub get_game_phase: FnThisU8,

    // =========================================================================
    //  Работа с entity tables (23–24)
    // =========================================================================
    /// `[23]` Активировать entity.
    ///
    /// - `tick_in_progress == 0` -> добавляет напрямую в hash table
    /// - `tick_in_progress != 0` -> откладывает в `pending_add_entities`
    pub activate_entity: FnActivateEntity,

    /// `[24]` Обработка удаления entity.
    ///
    /// - `tick_in_progress == 0` -> удаляет напрямую из hash table
    /// - `tick_in_progress != 0` -> откладывает в `pending_remove_entities`
    pub on_entity_deleted: FnOnEntityDeleted,
}

const _: () = {
    assert!(std::mem::size_of::<CGameVTable>() == 25 * 8);

    assert!(std::mem::offset_of!(CGameVTable, dtor) == 0 * 8);
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
    assert!(std::mem::offset_of!(CGameVTable, get_actors_pack) == 18 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_flag_bit4) == 19 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_entity_from_index) == 20 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_entity_at_index) == 21 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_game_phase) == 22 * 8);
    assert!(std::mem::offset_of!(CGameVTable, activate_entity) == 23 * 8);
    assert!(std::mem::offset_of!(CGameVTable, on_entity_deleted) == 24 * 8);
};
