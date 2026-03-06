use std::ffi::c_void;
use super::VehicleWrapper;

/// Базовый класс менеджера гаража.
///
/// VTable: `vtables::garage::GARAGE_MANAGER`
/// Global: `globals::GARAGE_MANAGER`
///
/// VTable методы:
/// - `[0]` destructor
/// - `[1]` GetSize() → 7
/// - `[2]` GetClassName() → "C_GarageManager"
/// - `[3]` RegisterLuaAPI()
/// - `[4]` GetSomeFloat() → 0.005f
#[repr(C)]
pub struct CGarageManager {
    pub vtable: *const c_void,              // +0x00
}

/// Конкретная реализация гаража (наследует `CGarageManager`).
///
/// Хранит машины игрока с подсчётом ссылок.
/// Только 34 машины могут быть сохранены (см. `GarageVehicleId`).
#[repr(C)]
pub struct CGarage {
    pub vtable: *const c_void,              // +0x00 (от CGarageManager)
    pub current_capacity: i32,              // +0x08
    _pad_0c: [u8; 0x10 - 0x0C],
    pub vehicles_begin: *mut *mut VehicleWrapper,    // +0x10
    pub vehicles_end: *mut *mut VehicleWrapper,      // +0x18
    pub vehicles_capacity: *mut *mut VehicleWrapper,  // +0x20
    _pad_028: [u8; 0x60 - 0x28],
    pub current_vehicle_index: i32,         // +0x60 (96)
    pub max_vehicles: i32,                  // +0x64 (100)
}

const _: () = {
    assert!(std::mem::offset_of!(CGarage, current_capacity) == 0x08);
    assert!(std::mem::offset_of!(CGarage, vehicles_begin) == 0x10);
    assert!(std::mem::offset_of!(CGarage, vehicles_end) == 0x18);
    assert!(std::mem::offset_of!(CGarage, vehicles_capacity) == 0x20);
    assert!(std::mem::offset_of!(CGarage, current_vehicle_index) == 0x60);
    assert!(std::mem::offset_of!(CGarage, max_vehicles) == 0x64);
};