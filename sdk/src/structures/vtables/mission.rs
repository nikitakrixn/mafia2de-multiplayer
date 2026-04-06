//! VTable `C_Mission` — 14 слотов.
//!
//! Адрес в `.rdata`: `M2DE_VT_CMission` (`0x14186EFE8`).
//! Идентификатор класса: строка `"C_Mission"` через слот `[2]`.
//!
//! ## Карта слотов
//!
//! | Слот | Имя | Описание |
//! |:----:|:----|:---------|
//! | 0 | `dtor` | Деструктор |
//! | 1 | `get_module_id` | Возвращает `7` |
//! | 2 | `get_class_name` | Возвращает `"C_Mission"` |
//! | 3 | `static_register` | Заглушка |
//! | 4 | `get_fixed_time_step` | Возвращает `0.005f` |
//! | 5 | `get_game_name_buf` | Возвращает `&game_name_buf` (`this + 0x18`) |
//! | 6 | `set_mission_part` | Записывает `unk_11c` (`+0x11C`) |
//! | 7 | `get_mission_part` | Читает `unk_11c` (`+0x11C`) |
//! | 8 | `init` | Создаёт сцену через GfxDevice |
//! | 9 | `done` | Уничтожает сцену |
//! | 10 | `is_game_inited` | `state_flags & 1` |
//! | 11 | `open` | Загрузка миссии, fires events 9+10 |
//! | 12 | `close` | Выгрузка миссии, fires events 11+12 |
//! | 13 | `is_opened` | `state_flags & 2` |

use std::ffi::{c_char, c_void};

type FnDtor     = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnU32      = unsafe extern "system" fn(this: *const c_void) -> u32;
type FnF32      = unsafe extern "system" fn(this: *const c_void) -> f32;
type FnBool     = unsafe extern "system" fn(this: *const c_void) -> bool;
type FnVoidPtr  = unsafe extern "system" fn(this: *const c_void) -> *mut c_void;
type FnSetU16   = unsafe extern "system" fn(this: *mut c_void, value: u16);
type FnGetU16   = unsafe extern "system" fn(this: *const c_void) -> u16;
type FnOpen     = unsafe extern "system" fn(this: *mut c_void, name: *const c_char) -> bool;
type FnClose    = unsafe extern "system" fn(this: *mut c_void) -> bool;

/// VTable `C_Mission` — `M2DE_VT_CMission` @ `0x14186EFE8`.
#[repr(C)]
pub struct CMissionVTable {
    /// `[0]` Деструктор `C_Mission`.
    pub dtor: FnDtor,

    /// `[1]` Возвращает module id = `7`.
    pub get_module_id: FnU32,

    /// `[2]` Возвращает строку `"C_Mission"`.
    pub get_class_name: unsafe extern "system" fn(this: *const c_void) -> *const c_char,

    /// `[3]` Заглушка (`nullsub`).
    pub static_register: unsafe extern "system" fn(this: *mut c_void),

    /// `[4]` Фиксированный шаг симуляции: `0.005f` (200 Гц).
    pub get_fixed_time_step: FnF32,

    /// `[5]` Возвращает указатель на `game_name_buf` (`this + 0x18`).
    ///
    /// `return a1 + 24` в M2DE.
    pub get_game_name_buf: FnVoidPtr,

    /// `[6]` Записывает `unk_11c` (`+0x11C`).
    ///
    /// `*(_WORD *)(a1 + 284) = a2` в M2DE.
    pub set_mission_part: FnSetU16,

    /// `[7]` Читает `unk_11c` (`+0x11C`).
    ///
    /// `return *(u16*)(a1 + 284)` в M2DE.
    pub get_mission_part: FnGetU16,

    /// `[8]` Инициализация — создаёт сцену через `GfxDevice`.
    ///
    /// Устанавливает `scene_ptr (+0x10)`, `state_flags |= 1`.
    pub init: FnBool,

    /// `[9]` Деинициализация — уничтожает сцену.
    ///
    /// Если `is_opened`: fires events 11+12, очищает сцену.
    /// Освобождает `scene_ptr`, сбрасывает `state_flags & ~1`.
    pub done: FnBool,

    /// `[10]` Проверка: game инициализирован.
    ///
    /// Эквивалентно `(state_flags & 1) != 0`.
    pub is_game_inited: FnBool,

    /// `[11]` Загрузка миссии.
    ///
    /// Fires `MISSION_BEFORE_OPEN` (9), копирует имя в `game_name_buf (+0x18)`,
    /// fires `MISSION_AFTER_OPEN` (10), устанавливает `state_flags |= 2`.
    pub open: FnOpen,

    /// `[12]` Выгрузка миссии.
    ///
    /// Fires `MISSION_BEFORE_CLOSE` (11), очищает сцену,
    /// fires `MISSION_AFTER_CLOSE` (12), сбрасывает `state_flags & ~2`,
    /// очищает `game_name_buf`.
    pub close: FnClose,

    /// `[13]` Проверка: миссия открыта.
    ///
    /// Эквивалентно `(state_flags & 2) != 0`.
    pub is_opened: FnBool,
}

const _: () = {
    assert!(std::mem::size_of::<CMissionVTable>() == 14 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, dtor)             == 0 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, get_module_id)    == 1 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, get_class_name)   == 2 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, get_fixed_time_step) == 4 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, get_game_name_buf) == 5 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, set_mission_part) == 6 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, get_mission_part) == 7 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, init)             == 8 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, done)             == 9 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, is_game_inited)   == 10 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, open)             == 11 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, close)            == 12 * 8);
    assert!(std::mem::offset_of!(CMissionVTable, is_opened)        == 13 * 8);
};
