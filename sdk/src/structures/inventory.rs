//! Инвентарь и система денег.
//!
//! Цепочка до денег:
//! `CHuman -> Inventory -> slots[5] -> vector[0] -> wallet -> inner -> container -> value`
//!
//! Это длинная цепочка указателей, но она подтверждена
//! по IDA и runtime-проверкам. Каждый уровень может быть NULL
//! на ранних стадиях инициализации.

use crate::macros::assert_field_offsets;
use std::ffi::c_void;

/// Инвентарь игрока.
///
/// Содержит массив слотов (std::vector<InventorySlot*>):
/// - слоты 0-1 — неизвестно
/// - слоты 2-3 — оружие
/// - слот 4 — боеприпасы
/// - слот 5 — деньги
#[repr(C)]
pub struct Inventory {
    pub vtable: *const c_void, // +0x00
    _pad_008: [u8; 0x24 - 0x08],
    /// Тип инвентаря. Для игрока значение ≠ 16.
    /// Значение 16 — NPC (влияет на HUD popup).
    pub inv_type: u8, // +0x24
    _pad_025: [u8; 0x50 - 0x25],
    /// Начало массива указателей на слоты.
    pub slots_start: *mut InventorySlot, // +0x50
    /// Конец массива (slots_end - slots_start) / 8 = количество слотов.
    pub slots_end: *mut InventorySlot, // +0x58
    _pad_060: [u8; 0xE8 - 0x60],
    /// Указатель на подсистему оружия.
    pub weapons: *mut c_void, // +0xE8
}

/// Один слот инвентаря.
///
/// Внутри — std::vector указателей на предметы.
/// Для денежного слота (index=5) вектор содержит один элемент.
#[repr(C)]
pub struct InventorySlot {
    pub vtable: *const c_void, // +0x00
    _pad_008: [u8; 0x18 - 0x08],
    /// Указатель на данные предмета в слоте.
    pub data: *mut InventoryData, // +0x18
}

/// Данные элемента инвентаря.
#[repr(C)]
pub struct InventoryData {
    pub vtable: *const c_void, // +0x00
    _pad_008: [u8; 0x10 - 0x08],
    /// Указатель на значение (для денег — MoneyValue).
    pub value_ptr: *mut c_void, // +0x10
}

/// Хранилище суммы денег (последний уровень цепочки).
///
/// Значение хранится в центах: $600.00 = 60000.
#[repr(C)]
pub struct MoneyValue {
    pub vtable: *const c_void, // +0x00
    _pad_008: [u8; 0x10 - 0x08],
    /// Сумма в центах (i32, но реально движок пишет i64).
    pub amount: i32, // +0x10
}

assert_field_offsets!(Inventory {
    inv_type    == 0x24,
    slots_start == 0x50,
    slots_end   == 0x58,
    weapons     == 0xE8,
});

assert_field_offsets!(InventorySlot {
    data == 0x18,
});

assert_field_offsets!(InventoryData {
    value_ptr == 0x10,
});

assert_field_offsets!(MoneyValue {
    amount == 0x10,
});
