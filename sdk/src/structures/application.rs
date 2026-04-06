//! Структура `C_Application` — главный объект игрового модуля.
//!
//! M2DE singleton: `g_M2DE_pGameModule`.
//!
//! ## Иерархия
//!
//! ```text
//! C_TickedModule
//!   └─ C_Application
//! ```
//!
//! ## Layout (из `M2DE_CApplication_Constructor` @ `0x1400EDFA0`)
//!
//! ```text
//! +0x000  vtable              -> M2DE_VT_CApplication (5 слотов)
//! +0x008  module_id           i32  (init = -1)
//! +0x010  game_name_str       C_String (ue::sys::utils::C_String)
//! +0x018  game_name_buf       char  (init = 0, заполняется в Start)
//! +0x098  tick_counter        u32  (init = 0)
//! +0x09C  reload_flag         u8   (init = 0)
//! +0x09D  unk_9d              u8   (init = 1)
//! +0x0A0  save_cb_0           ModuleObject (size 0x18)
//! +0x0B0  save_cb_1           ModuleObject (size 0x0C)
//! +0x0C0  save_cb_2           ModuleObject (size 0)
//! +0x0D0  mission_number      i64  (init = -1)
//! +0x0D8  mission_part        u32  (init = 0)
//! +0x0E0  dlc_mission_data    *DLCMissionPackData (64B)
//! +0x0E8  sys_notify_vtable   *vtable (C_SysNotificationCallback)
//! +0x0F0  unk_f0              u8   (init = 0)
//! +0x0F8  unk_f8              *vtable
//! +0x100  sentinel            i32  (init = -1)
//! ```

use super::vtables::application::CApplicationVTable;
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

/// Главный объект игрового модуля (`C_Application`).
///
/// Конструктор: `M2DE_CApplication_Constructor` (`0x1400EDFA0`).
/// Vtable: `M2DE_VT_CApplication` (`0x1418538F8`).
#[repr(C)]
pub struct CApplication {
    /// `+0x000` VTable -> `M2DE_VT_CApplication`.
    pub vtable: *const CApplicationVTable,

    /// `+0x008` Module ID (init = -1).
    ///
    /// `*(_DWORD *)(a1 + 8) = -1` в конструкторе.
    pub module_id: i32,

    _pad_00c: u32,

    /// `+0x010` `ue::sys::utils::C_String` — строковый объект имени игры.
    _game_name_str: [u8; 8],

    /// `+0x018` Буфер имени карты для Tick/Reload.
    ///
    /// Заполняется в `M2DE_CApplication_Start` из SDS line `'game'`.
    /// Читается в `M2DE_CApplication_Tick` для вызова `C_Game::Open`.
    /// `*(_BYTE *)(a1 + 24) = 0` в конструкторе.
    pub game_name_buf: [u8; 128],

    /// `+0x098` Счётчик/результат последнего тика.
    ///
    /// `*(_DWORD *)(a1 + 152) = 1` при успешном reload в `Tick`.
    /// `*(_DWORD *)(a1 + 152) = 2` при паузе в `GameTick_Hook`.
    pub tick_counter: u32,

    /// `+0x09C` Флаг паузы из `GameTick_Hook`.
    ///
    /// Если установлен: `C_Game::STATE_PAUSED |= 4`, `tick_counter = 2`.
    /// `*(_BYTE *)(a1 + 156) = 0` в конструкторе.
    pub reload_flag: u8,

    /// `+0x09D` Неизвестный флаг (init = 1).
    ///
    /// `*(_BYTE *)(a1 + 157) = 1` в конструкторе.
    pub unk_9d: u8,

    _pad_09e: [u8; 2],

    /// `+0x0A0` SaveLoadCallback #0 (size 0x18). Vtable: `off_141853890`.
    ///
    /// Регистрируется в `RegGameSaveCb` с типами 0, 1, 4, 8, 9, 10.
    _save_cb_0: [u8; 0x10],

    /// `+0x0B0` SaveLoadCallback #1 (size 0x0C). Vtable: `off_1418538A8`.
    ///
    /// Регистрируется в `RegGameSaveCb` с типом 3.
    _save_cb_1: [u8; 0x10],

    /// `+0x0C0` SaveLoadCallback #2 (size 0). Vtable: `off_1418538C0`.
    ///
    /// Регистрируется в `RegGameSaveCb` с типом 1.
    _save_cb_2: [u8; 0x10],

    /// `+0x0D0` Номер текущей миссии (init = -1).
    ///
    /// `*(_QWORD *)(a1 + 208) = -1` в конструкторе.
    pub mission_number: i64,

    /// `+0x0D8` Часть текущей миссии (init = 0).
    ///
    /// `*(_DWORD *)(a1 + 216) = 0` в конструкторе.
    pub mission_part: u32,

    _pad_0dc: u32,

    /// `+0x0E0` Указатель на `DLCMissionPackData` (64B).
    ///
    /// `*(_QWORD *)(a1 + 224) = sub_1400F1D90(alloc(64))` в конструкторе.
    pub dlc_mission_data: *mut c_void,

    /// `+0x0E8` `C_SysNotificationCallback` vtable ptr.
    ///
    /// `*(_QWORD *)(a1 + 232) = off_1418538D8` в конструкторе.
    _sys_notify_vtable: *const c_void,

    /// `+0x0F0` Флаг (init = 0).
    ///
    /// `*(_BYTE *)(a1 + 240) = 0` в конструкторе.
    pub unk_f0: u8,

    _pad_0f1: [u8; 7],

    /// `+0x0F8` Vtable ptr.
    ///
    /// `*(_QWORD *)(a1 + 248) = off_1418538E8` в конструкторе.
    _unk_f8_vtable: *const c_void,

    /// `+0x100` Sentinel (init = -1).
    ///
    /// `*(_DWORD *)(a1 + 256) = -1` в конструкторе.
    pub sentinel: i32,

    _pad_104: u32,
}

assert_field_offsets!(CApplication {
    vtable           == 0x000,
    module_id        == 0x008,
    game_name_buf    == 0x018,
    tick_counter     == 0x098,
    reload_flag      == 0x09C,
    unk_9d           == 0x09D,
    mission_number   == 0x0D0,
    mission_part     == 0x0D8,
    dlc_mission_data == 0x0E0,
    unk_f0           == 0x0F0,
    sentinel         == 0x100,
});

impl CApplication {
    /// Имя текущей карты (из буфера Tick/Reload).
    #[inline]
    pub fn game_name(&self) -> Option<&str> {
        let buf = &self.game_name_buf;
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        if len == 0 {
            return None;
        }
        std::str::from_utf8(&buf[..len]).ok()
    }

    /// Reload был успешным.
    #[inline]
    pub fn is_reload_success(&self) -> bool {
        self.reload_flag != 0
    }

    /// Миссия загружена (mission_number != -1).
    #[inline]
    pub fn has_mission(&self) -> bool {
        self.mission_number != -1
    }
}
