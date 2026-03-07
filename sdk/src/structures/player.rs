use std::ffi::c_void;
use super::Inventory;

/// Игрок (наследует Entity).
#[repr(C)]
pub struct CHuman {
    pub vtable: *const c_void,          // +0x00
    _pad_008: [u8; 0x78 - 0x08],
    pub frame_node: *mut c_void,        // +0x78 (transform node)
    _pad_080: [u8; 0xE8 - 0x80],
    pub inventory: *mut Inventory,      // +0xE8
    pub control_component: *mut c_void, // +0xF0
}

const _: () = {
    assert!(std::mem::offset_of!(CHuman, frame_node) == 0x78);
    assert!(std::mem::offset_of!(CHuman, inventory) == 0xE8);
    assert!(std::mem::offset_of!(CHuman, control_component) == 0xF0);
};