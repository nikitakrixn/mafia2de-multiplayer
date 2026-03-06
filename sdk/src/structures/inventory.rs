use std::ffi::c_void;

/// Инвентарь игрока.
#[repr(C)]
pub struct Inventory {
    pub vtable: *const c_void,              // +0x00
    _pad_008: [u8; 0x24 - 0x08],
    pub inv_type: u8,                       // +0x24 (для игрока ≠ 16)
    _pad_025: [u8; 0x50 - 0x25],
    pub slots_start: *mut InventorySlot,    // +0x50
    pub slots_end: *mut InventorySlot,      // +0x58
    _pad_060: [u8; 0xE8 - 0x60],
    pub weapons: *mut c_void,               // +0xE8
}

/// Один слот инвентаря. Слот 5 = деньги.
#[repr(C)]
pub struct InventorySlot {
    pub vtable: *const c_void,              // +0x00
    _pad_008: [u8; 0x18 - 0x08],
    pub data: *mut InventoryData,           // +0x18
}

/// Данные элемента инвентаря.
#[repr(C)]
pub struct InventoryData {
    pub vtable: *const c_void,              // +0x00
    _pad_008: [u8; 0x10 - 0x08],
    pub value_ptr: *mut c_void,             // +0x10
}

/// Значение денег (за `InventoryData::value_ptr`).
#[repr(C)]
pub struct MoneyValue {
    pub vtable: *const c_void,              // +0x00
    _pad_008: [u8; 0x10 - 0x08],
    pub amount: i32,                        // +0x10
}

const _: () = {
    assert!(std::mem::offset_of!(Inventory, inv_type)    == 0x24);
    assert!(std::mem::offset_of!(Inventory, slots_start) == 0x50);
    assert!(std::mem::offset_of!(Inventory, slots_end)   == 0x58);
    assert!(std::mem::offset_of!(Inventory, weapons)     == 0xE8);
    assert!(std::mem::offset_of!(InventorySlot, data)    == 0x18);
    assert!(std::mem::offset_of!(InventoryData, value_ptr) == 0x10);
    assert!(std::mem::offset_of!(MoneyValue, amount)     == 0x10);
};