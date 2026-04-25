//! Структура `C_ActorsPack` — inline подобъект `C_Game` (`+0x58`).
//!
//! Управляет набором actor entity для текущей карты.
//! Загружается из `.bin` файла через `Open`, выгружается через `Close`/`CloseTick`.
//!
//! ## Иерархия
//!
//! ```text
//! I_ActorsPack  (vtable M2DE_VT_IActorsPack  @ 0x14186D9C8)
//!   └─ C_ActorsPack (vtable M2DE_VT_CActorsPack @ 0x14186EFB8)
//! ```
//!
//! ## Layout (0x128 байт)
//!
//! ```text
//! +0x000  vtable              -> M2DE_VT_CActorsPack (6 слотов)
//! +0x008  flags               u8   (init=0)
//! +0x088  state_flags         u32  (bit0=game_inited, bit1=opened)
//! +0x090  entities_begin      *ptr (вектор entity, begin)
//! +0x098  entities_end        *ptr (вектор entity, end)
//! +0x0A0  entities_cap        *ptr (вектор entity, capacity)
//! +0x0A8  scene_callback      *ptr (-> off_141C2EC30, перезаписывается C_ActorsPack)
//! +0x0B0  close_counter       i32  (init=-1, итератор при CloseTick)
//! +0x0B8  bin_data            *ptr (буфер из ParseFromBinDataInit, init=0)
//! +0x0C0..+0x11F  [поля ParseFromBinDataInit]
//! +0x120  close_counter_2     i32  (init=-1)
//! ```
//!
//! ## VTable слоты
//!
//! | Слот | Функция | Описание |
//! |:----:|:--------|:---------|
//! | 0 | `dtor` | Деструктор |
//! | 1 | `open` | Загрузка из `.bin` файла |
//! | 2 | `close` | Выгрузка (вызывает CloseTick с force=true) |
//! | 3 | `close_tick_init` | Инициализация итератора закрытия |
//! | 4 | `close_tick` | Пошаговая выгрузка entity |
//! | 5 | `find_parent_in_scene` | Поиск frame в сцене по имени |

use crate::macros::assert_field_offsets;
use super::vtables::actors_pack::CActorsPackVTable;
use std::ffi::{c_char, c_void};

/// Inline подобъект `C_ActorsPack` внутри `C_Game` (`+0x58`).
///
/// Конструктор: `M2DE_CActorsPack_Constructor` (`0x1403D10D0`).
/// Vtable: `M2DE_VT_CActorsPack` (`0x14186EFB8`).
/// Размер: **0x128 байт**.
#[repr(C)]
pub struct CActorsPack {
    /// `+0x000` VTable -> `M2DE_VT_CActorsPack`.
    pub vtable: *const CActorsPackVTable,

    /// `+0x008` Флаг (init=0).
    ///
    /// `*(_BYTE *)(a1 + 8) = 0` в `I_ActorsPack_Constructor`.
    pub flags: u8,

    _pad_009: [u8; 0x7F],

    /// `+0x088` Флаги состояния.
    ///
    /// - bit 0 = game_inited (`GameDone` сбрасывает)
    /// - bit 1 = opened (`Open` устанавливает, `CloseTick` сбрасывает)
    ///
    /// `*(_DWORD *)(a1 + 136) = 0` в `I_ActorsPack_Constructor`.
    pub state_flags: u32,

    _pad_08c: u32,

    /// `+0x090` Начало вектора entity.
    ///
    /// `*(_QWORD *)(a1 + 144) = 0` в `I_ActorsPack_Constructor`.
    pub entities_begin: *mut c_void,

    /// `+0x098` Конец вектора entity.
    ///
    /// `*(_QWORD *)(a1 + 152) = 0` в `I_ActorsPack_Constructor`.
    pub entities_end: *mut c_void,

    /// `+0x0A0` Конец выделенной памяти вектора.
    ///
    /// `*(_QWORD *)(a1 + 160) = 0` в `I_ActorsPack_Constructor`.
    pub entities_cap: *mut c_void,

    /// `+0x0A8` Указатель на scene callback.
    ///
    /// В `I_ActorsPack_Constructor` = 0, в `C_ActorsPack_Constructor` = `&off_141C2EC30`.
    pub scene_callback: *const c_void,

    /// `+0x0B0` Счётчик закрытия (итератор при `CloseTick`).
    ///
    /// `*(_DWORD *)(a1 + 176) = -1` в `I_ActorsPack_Constructor`.
    /// Устанавливается в `CloseTickInit` как `(entities_end - entities_begin) / 8 - 1`.
    pub close_counter: i32,

    _pad_0b4: u32,

    /// `+0x0B8` Буфер бинарных данных из `ParseFromBinDataInit`.
    ///
    /// `*(_QWORD *)(a1 + 184) = 0` в `C_ActorsPack_Constructor`.
    /// Освобождается в `CloseTick` после завершения.
    pub bin_data: *mut c_void,

    /// `+0x0C0..+0x11F` Поля `ParseFromBinDataInit` (неразобраны).
    ///
    /// Инициализируются нулями в `C_ActorsPack_Constructor`.
    _parse_fields: [u8; 0x60],

    /// `+0x120` Второй счётчик закрытия.
    ///
    /// `*(_DWORD *)(a1 + 288) = -1` в `C_ActorsPack_Constructor`.
    pub close_counter_2: i32,

    _pad_124: u32,
}

assert_field_offsets!(CActorsPack {
    vtable          == 0x000,
    flags           == 0x008,
    state_flags     == 0x088,
    entities_begin  == 0x090,
    entities_end    == 0x098,
    entities_cap    == 0x0A0,
    scene_callback  == 0x0A8,
    close_counter   == 0x0B0,
    bin_data        == 0x0B8,
    close_counter_2 == 0x120,
});

const _: () = {
    assert!(std::mem::size_of::<CActorsPack>() == 0x128);
};

impl CActorsPack {
    /// Пак открыт (bit 1 в `state_flags`).
    #[inline]
    pub fn is_opened(&self) -> bool {
        self.state_flags & 2 != 0
    }

    /// Game инициализирован (bit 0 в `state_flags`).
    #[inline]
    pub fn is_game_inited(&self) -> bool {
        self.state_flags & 1 != 0
    }

    /// Количество entity в паке.
    #[inline]
    pub fn entity_count(&self) -> usize {
        let begin = self.entities_begin as usize;
        let end = self.entities_end as usize;
        if end > begin { (end - begin) / 8 } else { 0 }
    }

    /// Открыть пак из файла (vtable slot `[1]`).
    ///
    /// # Safety
    /// Объект должен быть валиден, путь — корректная C-строка.
    #[inline]
    pub unsafe fn vtbl_open(&mut self, path: *const c_char) -> bool {
        unsafe { ((*self.vtable).open)(self as *mut _ as *mut _, path) }
    }

    /// Закрыть пак (vtable slot `[2]`).
    ///
    /// # Safety
    /// Объект должен быть валиден.
    #[inline]
    pub unsafe fn vtbl_close(&mut self) -> bool {
        unsafe { ((*self.vtable).close)(self as *mut _ as *mut _) }
    }

    /// Инициализировать итератор закрытия (vtable slot `[3]`).
    ///
    /// # Safety
    /// Объект должен быть валиден.
    #[inline]
    pub unsafe fn vtbl_close_tick_init(&mut self) -> bool {
        unsafe { ((*self.vtable).close_tick_init)(self as *mut _ as *mut _) }
    }

    /// Пошаговая выгрузка entity (vtable slot `[4]`).
    ///
    /// # Safety
    /// Объект должен быть валиден.
    #[inline]
    pub unsafe fn vtbl_close_tick(&mut self, timer: u64, force: bool) -> bool {
        unsafe { ((*self.vtable).close_tick)(self as *mut _ as *mut _, timer, force) }
    }
}
