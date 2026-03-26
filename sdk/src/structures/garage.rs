//! Система гаража — хранение машин игрока.

use super::std_vector::StdVector;
use super::VehicleWrapper;
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

/// Базовый класс менеджера гаража.
#[repr(C)]
pub struct CGarageManager {
    pub vtable: *const c_void, // +0x00
}

/// Конкретная реализация гаража (наследует CGarageManager).
///
/// Хранит машины в `std::vector<VehicleWrapper*>`.
/// Максимум 34 машины — индексация через GarageVehicleId.
#[repr(C)]
pub struct CGarage {
    pub vtable: *const c_void, // +0x00
    /// Текущее количество машин в гараже.
    pub current_capacity: i32, // +0x08
    _pad_0c: [u8; 0x10 - 0x0C],
    /// Массив указателей на VehicleWrapper (`std::vector<VehicleWrapper*>`).
    pub vehicles: StdVector<*mut VehicleWrapper>, // +0x10
    _pad_028: [u8; 0x60 - 0x28],
    /// Индекс текущей выбранной машины.
    pub current_vehicle_index: i32, // +0x60
    /// Максимальное количество машин (обычно 34).
    pub max_vehicles: i32, // +0x64
}

assert_field_offsets!(CGarage {
    current_capacity      == 0x08,
    vehicles              == 0x10,
    current_vehicle_index == 0x60,
    max_vehicles          == 0x64,
});