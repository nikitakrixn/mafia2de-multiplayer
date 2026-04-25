//! Структура `C_Mission` — синглтон менеджера миссий.
//!
//! Отдельный глобальный объект `g_M2DE_pMission` (288B).
//!
//! ## Иерархия
//!
//! ```text
//! C_TickedModule
//!   └─ I_Mission (abstract, vtable M2DE_VT_IMission_Abstract)
//!       └─ C_Mission (vtable M2DE_VT_CMission @ 0x14186EFE8)
//! ```
//!
//! ## Layout (0x120 = 288 байт)
//!
//! ```text
//! +0x000  vtable          -> M2DE_VT_CMission (14 слотов)
//! +0x008  state_flags     u32  (bit0=game_inited, bit1=opened)
//! +0x010  scene_ptr       *ptr (текущая сцена)
//! +0x018  game_name_buf   char[0x104] (имя текущей миссии)
//! +0x11C  unk_11c         u16
//! ```
//!
//! ## Флаги состояния (`state_flags`, +0x08)
//!
//! | Бит | Назначение |
//! |:---:|:-----------|
//! | 0 | game_inited — `Init` вызван |
//! | 1 | opened — `Open` вызван |

use crate::macros::assert_field_offsets;
use super::vtables::mission::CMissionVTable;
use std::ffi::{c_char, c_void};

/// Синглтон менеджера миссий (`C_Mission`).
///
/// Глобальный указатель: `g_M2DE_pMission` (`0x141CAF778`).
/// Конструктор: `M2DE_CMission_Constructor` (`0x1403D1920`).
/// Vtable: `M2DE_VT_CMission` (`0x14186EFE8`).
/// Размер: **0x120 байт**.
#[repr(C)]
pub struct CMission {
    /// `+0x000` VTable -> `M2DE_VT_CMission`.
    pub vtable: *const CMissionVTable,

    /// `+0x008` Флаги состояния (init=0).
    ///
    /// | Бит | Назначение |
    /// |:---:|:-----------|
    /// | 0 | game_inited |
    /// | 1 | opened |
    pub state_flags: u32,

    _pad_00c: u32,

    /// `+0x010` Указатель на текущую сцену (init=0).
    pub scene_ptr: *mut c_void,

    /// `+0x018` Буфер имени текущей миссии (init=0).
    ///
    /// Заполняется в `Open`, очищается в `Close`.
    pub game_name_buf: [u8; 0x104],

    /// `+0x11C` Неизвестное поле (init=0).
    ///
    /// Читается/пишется через vtable слоты `[6]`/`[7]`.
    pub unk_11c: u16,

    _pad_11e: [u8; 2],
}

assert_field_offsets!(CMission {
    vtable          == 0x000,
    state_flags     == 0x008,
    scene_ptr       == 0x010,
    game_name_buf   == 0x018,
    unk_11c         == 0x11C,
});

const _: () = {
    assert!(std::mem::size_of::<CMission>() == 0x120);
};

impl CMission {
    /// Миссия инициализирована (bit 0).
    #[inline]
    pub fn is_game_inited(&self) -> bool {
        self.state_flags & 1 != 0
    }

    /// Миссия открыта (bit 1).
    #[inline]
    pub fn is_opened(&self) -> bool {
        self.state_flags & 2 != 0
    }

    /// Имя текущей миссии из `game_name_buf`.
    #[inline]
    pub fn mission_name(&self) -> Option<&str> {
        let len = self.game_name_buf.iter().position(|&b| b == 0)?;
        if len == 0 { return None; }
        std::str::from_utf8(&self.game_name_buf[..len]).ok()
    }

    /// Загрузить миссию (vtable slot `[11]`).
    ///
    /// Fires `MISSION_BEFORE_OPEN` (9) и `MISSION_AFTER_OPEN` (10).
    ///
    /// # Safety
    /// Объект должен быть валиден, `name` — корректная C-строка.
    #[inline]
    pub unsafe fn vtbl_open(&mut self, name: *const c_char) -> bool {
        unsafe { ((*self.vtable).open)(self as *mut _ as *mut _, name) }
    }

    /// Выгрузить миссию (vtable slot `[12]`).
    ///
    /// Fires `MISSION_BEFORE_CLOSE` (11) и `MISSION_AFTER_CLOSE` (12).
    ///
    /// # Safety
    /// Объект должен быть валиден.
    #[inline]
    pub unsafe fn vtbl_close(&mut self) -> bool {
        unsafe { ((*self.vtable).close)(self as *mut _ as *mut _) }
    }

    /// Инициализировать — создаёт сцену (vtable slot `[8]`).
    ///
    /// # Safety
    /// Объект должен быть валиден.
    #[inline]
    pub unsafe fn vtbl_init(&mut self) -> bool {
        unsafe { ((*self.vtable).init)(self as *mut _ as *mut _) }
    }

    /// Деинициализировать — уничтожает сцену (vtable slot `[9]`).
    ///
    /// # Safety
    /// Объект должен быть валиден.
    #[inline]
    pub unsafe fn vtbl_done(&mut self) -> bool {
        unsafe { ((*self.vtable).done)(self as *mut _ as *mut _) }
    }

    /// Получить `unk_11c` (vtable slot `[7]`).
    #[inline]
    pub unsafe fn vtbl_get_mission_part(&self) -> u16 {
        unsafe { ((*self.vtable).get_mission_part)(self as *const _ as *const _) }
    }

    /// Установить `unk_11c` (vtable slot `[6]`).
    #[inline]
    pub unsafe fn vtbl_set_mission_part(&mut self, value: u16) {
        unsafe { ((*self.vtable).set_mission_part)(self as *mut _ as *mut _, value) }
    }
}
