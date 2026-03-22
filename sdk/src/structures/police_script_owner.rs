//! Police-script owner singleton.
//!
//! Это НЕ полноценная entity-структура, а маленький singleton owner object,
//! который живёт в глобале `M2DE_g_PoliceScriptOwner`.
//!
//! Источники:
//! - `M2DE_PoliceScriptOwner_GetOrCreate` (`0x1400B3A50`)
//! - `M2DE_PoliceScriptOwner_Init` (`0x140EAC480`)
//! - `M2DE_PoliceScriptOwner_AtexitShutdown` (`0x1400B3250`)
//! - `M2DE_PoliceScriptOwner_InitChild` (`0x140EC4330`)
//! - `M2DE_PoliceScriptOwner_DestroyChild` (`0x140EC4220`)
//!
//! Confirmed:
//! - owner alloc size = `0x18`
//! - `+0x10` = active child ptr
//! - `+0x00` = root/sentinel ptr to `0x30`-byte self-linked node object
//! - `+0x08` = state/count-like field, init = 0

use std::ffi::c_void;

use crate::macros::assert_field_offsets;

/// Singleton owner object для police-script child path.
#[repr(C)]
#[allow(non_snake_case)]
pub struct PoliceScriptOwner {
    /// Root/sentinel pointer.
    ///
    /// В `Init` выделяется блок `0x30` байт и заполняется self-links:
    /// - `[0x00] = self`
    /// - `[0x08] = self`
    /// - `[0x10] = self`
    /// - `word [0x18] = 0x0101`
    pub root_or_sentinel: *mut c_void, // +0x00

    /// Count/state-like field.
    ///
    /// Init:
    /// - zero
    ///
    /// Shutdown:
    /// - reset to zero before owner free
    ///
    /// Точный смысл пока не завершён.
    pub count_or_state: usize, // +0x08

    /// Активный police-script child.
    ///
    /// InitChild:
    /// - создаёт child
    /// - сохраняет его сюда
    ///
    /// DestroyChild:
    /// - деактивирует и удаляет child
    /// - пишет NULL
    pub active_child: *mut c_void, // +0x10
}

assert_field_offsets!(PoliceScriptOwner {
    root_or_sentinel == 0x00,
    count_or_state   == 0x08,
    active_child     == 0x10,
});

/// Root/sentinel node object, выделяемый owner'ом.
///
/// Layout пока частично условный, но self-link pattern подтверждён.
#[repr(C)]
#[allow(non_snake_case)]
pub struct PoliceScriptOwnerNode {
    pub link_00: *mut c_void,           // +0x00
    pub link_08: *mut c_void,           // +0x08
    pub child_or_link_10: *mut c_void,  // +0x10
    pub flags_18: u16,                  // +0x18
    pub flags_1A: u16,                  // +0x1A
    pub _unknown_1C: [u8; 0x30 - 0x1C], // +0x1C..+0x2F
}

assert_field_offsets!(PoliceScriptOwnerNode {
    link_00          == 0x00,
    link_08          == 0x08,
    child_or_link_10 == 0x10,
    flags_18         == 0x18,
});

// =============================================================================
//  Documentation block
// =============================================================================
///
/// ```text
/// owner size = 0x18
///   +0x00 root/sentinel
///   +0x08 count/state
///   +0x10 active_child
///
/// root/sentinel size = 0x30
///   self-linked at +0x00 / +0x08 / +0x10
///   word +0x18 = 0x0101
///
/// owner lifecycle:
///   GetOrCreate -> Init -> store global -> atexit(Shutdown)
///   Dispatch(code=6) -> InitChild
///   Dispatch(code=2) -> DestroyChild
/// ```

pub const _POLICE_SCRIPT_OWNER_DOC: () = ();
