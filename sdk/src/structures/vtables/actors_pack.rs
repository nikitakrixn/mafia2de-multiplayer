//! VTable `C_ActorsPack` — 6 слотов.
//!
//! Адрес в `.rdata`: `M2DE_VT_CActorsPack` (`0x14186EFB8`).
//!
//! ## Карта слотов
//!
//! | Слот | Имя | Описание |
//! |:----:|:----|:---------|
//! | 0 | `dtor` | Деструктор |
//! | 1 | `open` | Загрузка из `.bin` файла |
//! | 2 | `close` | Выгрузка (force=true) |
//! | 3 | `close_tick_init` | Инициализация итератора закрытия |
//! | 4 | `close_tick` | Пошаговая выгрузка entity |
//! | 5 | `find_parent_in_scene` | Поиск frame в сцене по имени |

use std::ffi::{c_char, c_void};

type FnDtor  = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnBool  = unsafe extern "system" fn(this: *mut c_void) -> bool;
type FnOpen  = unsafe extern "system" fn(this: *mut c_void, path: *const c_char) -> bool;
type FnCloseTick = unsafe extern "system" fn(this: *mut c_void, timer: u64, force: bool) -> bool;
type FnFindParent = unsafe extern "system" fn(this: *mut c_void, name: *const c_char) -> *mut c_void;

/// VTable `C_ActorsPack` - `M2DE_VT_CActorsPack` @ `0x14186EFB8`.
#[repr(C)]
pub struct CActorsPackVTable {
    /// `[0]` Деструктор `C_ActorsPack`.
    ///
    /// Вызывает `CloseTick(force=true)` затем `I_ActorsPack::~I_ActorsPack`.
    pub dtor: FnDtor,

    /// `[1]` Загрузка из `.bin` файла.
    ///
    /// Читает заголовок (magic `0x12345678`), вызывает `ParseFromBinDataInit`.
    /// Устанавливает `state_flags |= 2` (opened).
    pub open: FnOpen,

    /// `[2]` Выгрузка.
    ///
    /// Вызывает `CloseTickInit` затем `CloseTick(0, force=true)`.
    pub close: FnBool,

    /// `[3]` Инициализация итератора закрытия.
    ///
    /// `close_counter = (entities_end - entities_begin) / 8 - 1`.
    pub close_tick_init: FnBool,

    /// `[4]` Пошаговая выгрузка entity.
    ///
    /// Итерирует entity в обратном порядке, вызывает `C_Entity::Release`.
    /// При `force=true` — выгружает всё за один вызов.
    /// После завершения: освобождает `bin_data`, сбрасывает `state_flags & ~2`.
    pub close_tick: FnCloseTick,

    /// `[5]` Поиск frame в сцене по имени.
    ///
    /// Ищет в векторе entity по FNV-1a hash имени.
    /// Fallback: `I_Mission::GetScene()->FindFrame(name)`.
    pub find_parent_in_scene: FnFindParent,
}

const _: () = {
    assert!(std::mem::size_of::<CActorsPackVTable>() == 6 * 8);
    assert!(std::mem::offset_of!(CActorsPackVTable, dtor)                == 0 * 8);
    assert!(std::mem::offset_of!(CActorsPackVTable, open)                == 1 * 8);
    assert!(std::mem::offset_of!(CActorsPackVTable, close)               == 2 * 8);
    assert!(std::mem::offset_of!(CActorsPackVTable, close_tick_init)     == 3 * 8);
    assert!(std::mem::offset_of!(CActorsPackVTable, close_tick)          == 4 * 8);
    assert!(std::mem::offset_of!(CActorsPackVTable, find_parent_in_scene) == 5 * 8);
};
