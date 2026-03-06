use std::ffi::c_void;
use super::Inventory;

/// Игрок (наследует Entity).
///
/// Проверенные смещения:
/// - `+0xE8` — `Inventory*`
///
/// Не определено:
/// - `current_vehicle` — требуется реверс
/// - `position` — требуется реверс
/// - `health` — требуется реверс
#[repr(C)]
pub struct Player {
    pub vtable: *const c_void,          // +0x00
    _pad_008: [u8; 0xE8 - 0x08],
    pub inventory: *mut Inventory,      // +0xE8
    // Далее — не определено
}

const _: () = {
    assert!(std::mem::offset_of!(Player, inventory) == 0xE8);
};