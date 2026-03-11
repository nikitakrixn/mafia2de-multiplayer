//! Engine-level сущность машины (C_Car).
//!
//! Низкоуровневый объект движка: физика, рендер, столкновения.
//! Не путать с Vehicle (Lua-обёртка для гаража/спавна).

use std::ffi::c_void;
use crate::macros::assert_field_offsets;

/// Engine-level сущность машины.
///
/// Множественное наследование — 3 vtable.
/// VTables: `vtables::car::{MAIN, BASE_2, BASE_3}`
#[repr(C)]
pub struct CCar {
    pub vtable: *const c_void,              // +0x00
    _pad_008: [u8; 0x38 - 0x08],
    /// Указатель на основные данные машины (физика, bbox).
    pub important_data: *mut CarData,       // +0x38
}

/// Основные данные машины (по C_Car::important_data).
///
/// Содержит физику, рендеринг, bounding box.
///
/// ВАЖНО: область bbox (+0x328..+0x340) содержит f64 по адресу 0x334,
/// что не кратно 8. Это значит что в игре эта структура либо packed,
/// либо типы полей не совсем такие как в IDA. Храним bbox как сырые
/// байты с типизированными аксессорами — так layout гарантированно
/// совпадает с памятью игры.
#[repr(C)]
pub struct CarData {
    _pad_000: [u8; 0x270],
    /// Информация о размерах модели.
    pub size_info: *mut c_void,             // +0x270
    _pad_278: [u8; 0x2F8 - 0x278],
    /// Параметры инициализации (из SDS/ресурсов).
    pub init_info: *mut c_void,             // +0x2F8
    _pad_300: [u8; 0x328 - 0x300],

    /// Bounding box — сырые байты области +0x328..+0x340.
    ///
    /// Внутренняя раскладка (из IDA, не проверена на 100%):
    ///   +0x00 (0x328): f64 bbox_min_x
    ///   +0x08 (0x330): i32 bbox_min_y
    ///   +0x0C (0x334): f64 bbox_max_x  ← не выровнен по 8!
    ///   +0x14 (0x33C): i32 bbox_max_y
    ///
    /// Из-за невыровненного f64 нельзя представить как поля repr(C).
    /// Используй аксессоры ниже для типизированного чтения.
    pub bbox_raw: [u8; 0x340 - 0x328],     // +0x328, 24 байта
}

impl CarData {
    /// Bounding box: минимум X (f64 по +0x328).
    pub fn bbox_min_x(&self) -> f64 {
        let bytes: [u8; 8] = self.bbox_raw[0x00..0x08].try_into().unwrap();
        f64::from_le_bytes(bytes)
    }

    /// Bounding box: минимум Y (i32 по +0x330).
    pub fn bbox_min_y(&self) -> i32 {
        let bytes: [u8; 4] = self.bbox_raw[0x08..0x0C].try_into().unwrap();
        i32::from_le_bytes(bytes)
    }

    /// Bounding box: максимум X (f64 по +0x334, невыровненный!).
    pub fn bbox_max_x(&self) -> f64 {
        let bytes: [u8; 8] = self.bbox_raw[0x0C..0x14].try_into().unwrap();
        f64::from_le_bytes(bytes)
    }

    /// Bounding box: максимум Y (i32 по +0x33C).
    pub fn bbox_max_y(&self) -> i32 {
        let bytes: [u8; 4] = self.bbox_raw[0x14..0x18].try_into().unwrap();
        i32::from_le_bytes(bytes)
    }
}

assert_field_offsets!(CCar {
    important_data == 0x38,
});

assert_field_offsets!(CarData {
    size_info == 0x270,
    init_info == 0x2F8,
    bbox_raw  == 0x328,
});