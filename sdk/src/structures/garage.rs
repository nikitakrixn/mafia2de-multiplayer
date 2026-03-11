//! Система гаража — хранение машин игрока.
//!
//! Гараж вмещает до 34 машин (см. GarageVehicleId).
//! Машины хранятся через VehicleWrapper с подсчётом ссылок.
//! На ранней стадии нужно реверсить, чтобы понять что точно хранится в гараже и как это работает.

use std::ffi::c_void;
use super::VehicleWrapper;
use crate::macros::assert_field_offsets;

/// Базовый класс менеджера гаража.
///
/// VTable: `vtables::garage::GARAGE_MANAGER`
/// Глобал: `globals::GARAGE_MANAGER`
///
/// VTable методы:
/// - [0] destructor
/// - [1] GetSize() → 7
/// - [2] GetClassName() → "C_GarageManager"
/// - [3] RegisterLuaAPI()
/// - [4] GetSomeFloat() → 0.005f
#[repr(C)]
pub struct CGarageManager {
    pub vtable: *const c_void,              // +0x00
}

/// Конкретная реализация гаража (наследует CGarageManager).
///
/// Хранит машины в std::vector<VehicleWrapper**>.
/// Максимум 34 машины — индексация через GarageVehicleId.
#[repr(C)]
pub struct CGarage {
    pub vtable: *const c_void,                          // +0x00
    /// Текущее количество машин в гараже.
    pub current_capacity: i32,                          // +0x08
    _pad_0c: [u8; 0x10 - 0x0C],
    /// Начало массива указателей на VehicleWrapper.
    pub vehicles_begin: *mut *mut VehicleWrapper,       // +0x10
    /// Конец массива (для подсчёта: (end - begin) / 8).
    pub vehicles_end: *mut *mut VehicleWrapper,         // +0x18
    /// Конец выделенной памяти массива.
    pub vehicles_capacity: *mut *mut VehicleWrapper,    // +0x20
    _pad_028: [u8; 0x60 - 0x28],
    /// Индекс текущей выбранной машины.
    pub current_vehicle_index: i32,                     // +0x60
    /// Максимальное количество машин (обычно 34).
    pub max_vehicles: i32,                              // +0x64
}

assert_field_offsets!(CGarage {
    current_capacity      == 0x08,
    vehicles_begin        == 0x10,
    vehicles_end          == 0x18,
    vehicles_capacity     == 0x20,
    current_vehicle_index == 0x60,
    max_vehicles          == 0x64,
});