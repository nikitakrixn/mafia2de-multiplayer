//! `C_SysInput` — низкоуровневый менеджер DirectInput-устройств.
//!
//! Singleton. Хранит RB-tree всех зарегистрированных устройств
//! (клавиатура, три варианта мыши, гамепад) и ставит их на паузу /
//! возобновляет, когда выше по стеку дёргается
//! `C_GameInputModule::PauseInput`.
//!
//! ## Singleton
//!
//! Указатель живёт по
//! [`globals::SYS_INPUT_INSTANCE`][crate::addresses::globals::SYS_INPUT_INSTANCE]
//! — это **указатель на объект**, а не сам объект (в отличие от
//! `C_GameInputModule`). Создаётся лениво в
//! `M2DE_C_SysInput_CreateInstance` (`0x14079FF00`).
//!
//! ## Layout (по `M2DE_C_SysInput_CreateInstance` @ `0x14079FF00`)
//!
//! ```text
//! +0x000  vtable               -> M2DE_VT_CSysInput (0x141895C00)
//! +0x008  devices_anchor       *Node       sentinel RB-tree устройств
//! +0x010  ?                    qword       (init = 0)
//! +0x018  ?                    qword       (init = 0)
//! +0x020  events_anchor        *Node       sentinel RB-tree (вероятно
//!                                            подписки на DI-события)
//! +0x028  ?
//! +0x030  ?
//! +0x038  ?
//! +0x040  ?
//! +0x048  ?
//! +0x050  m_b_paused           u8          выставляется в C_SysInput::Pause
//! +0x058  ?
//! +0x060  ?
//! +0x068  ?
//! +0x078  state_block          *void       40-байтовый под-объект
//!                                            (предположительно XInput state)
//! ```
//!
//! Полный размер 128 байт (`M2DE_GlobalAlloc(128)`). Не исследованные
//! ячейки оставлены как `_pad_*`, чтобы гарантировать корректные офсеты
//! без выдумывания типов.

use std::ffi::c_void;

use super::vtables::c_sys_input::CSysInputVTable;
use crate::macros::assert_field_offsets;

/// Узел RB-tree, в который C_SysInput складывает свои устройства и
/// подписки. Layout — стандартный 40-байтовый sentinel:
/// `{ left, right, parent, color/flags }`.
#[repr(C)]
pub struct CSysInputNode {
    pub left: *mut CSysInputNode,
    pub right: *mut CSysInputNode,
    pub parent: *mut CSysInputNode,
    /// `+0x18` `[u8 color, u8 is_sentinel, ..]`. В sentinel-узле
    /// записывается `0x0101` (`257`).
    pub color_flags: u16,
    _pad_1a: [u8; 6],
    /// `+0x20` Полезная нагрузка узла (для devices — `C_Device*`).
    pub data: *mut c_void,
}

/// Singleton низкоуровневого менеджера DI-устройств.
#[repr(C)]
pub struct CSysInput {
    /// `+0x000` VTable -> `M2DE_VT_CSysInput` (`0x141895C00`).
    pub vtable: *const CSysInputVTable,

    /// `+0x008` Sentinel-узел RB-tree всех зарегистрированных устройств.
    /// `*sentinel == sentinel` (циклическая ссылка) пока пусто.
    pub devices_anchor: *mut CSysInputNode,

    _pad_010: [u8; 0x10],

    /// `+0x020` Второй RB-tree (предположительно подписки на DI-события).
    pub events_anchor: *mut CSysInputNode,

    _pad_028: [u8; 0x28],

    /// `+0x050` Pause-флаг. Читается всеми Update-цепочками; когда `true`,
    /// устройства не обновляются.
    pub m_b_paused: u8,

    _pad_051: [u8; 7],

    _pad_058: [u8; 0x20],

    /// `+0x078` 40-байтовый под-объект (предположительно XInput-state).
    pub state_block: *mut c_void,
}

assert_field_offsets!(CSysInput {
    vtable         == 0x000,
    devices_anchor == 0x008,
    events_anchor  == 0x020,
    m_b_paused     == 0x050,
    state_block    == 0x078,
});

impl CSysInput {
    /// Поднят ли pause-флаг (мышь/клавиатура заморожены).
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.m_b_paused != 0
    }
}
