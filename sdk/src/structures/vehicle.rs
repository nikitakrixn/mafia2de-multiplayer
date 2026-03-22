//! Высокоуровневая обёртка транспорта (Vehicle) и ref-counted wrapper.
//!
//! НЕ путать с C_Car — это разные сущности:
//! - Vehicle — Lua-уровень, гараж, spawn system (184 байт аллокации)
//! - C_Car — engine-уровень, физика, рендер, столкновения

use crate::macros::{assert_field_offsets, assert_layout};
use std::ffi::c_void;

/// High-level Vehicle (Lua API wrapper).
///
/// Состояния (поле по +0x08 для standalone Vehicle):
/// - 6: Initial — только создан, ничего не загружено
/// - 5: Deferred — создан, ожидает загрузки модели
/// - 1: Active — полностью инициализирован, C_Car существует
#[repr(C)]
pub struct Vehicle {
    pub vtable: *const c_void, // +0x000
    _pad_008: [u8; 0xE0 - 0x08],
    /// Данные спавна (позиция, направление, параметры).
    pub spawn_data: *mut c_void, // +0x0E0
    _pad_0e8: [u8; 0x360 - 0xE8],
    /// Текущая скорость (внутреннее представление движка).
    pub speed: u64, // +0x360
    /// Вспомогательное поле скорости.
    pub speed_related: u32, // +0x368
    _pad_36c: [u8; 0x388 - 0x36C],
    /// Параметр анимации #1 (плавность разгона и т.п.).
    pub anim_param1: f32, // +0x388
    _pad_38c: [u8; 0x394 - 0x38C],
    /// Параметр анимации #2.
    pub anim_param2: f32, // +0x394
    _pad_398: [u8; 0x1248 - 0x398],
    /// Временная метка начала спавна.
    pub spawn_timestamp: u64, // +0x1248
    _pad_1250: [u8; 0x1288 - 0x1250],
    /// Минимальное время появления (секунды).
    pub min_spawn_time: f32, // +0x1288
    /// Максимальное время появления (секунды).
    pub max_spawn_time: f32, // +0x128C
    _pad_1290: [u8; 0x12AC - 0x1290],
    /// Прогресс спавна: 0.0 (начало) -> 1.0 (готово).
    pub spawn_progress: f32, // +0x12AC
    _pad_12b0: [u8; 0x12CC - 0x12B0],
    /// Множитель скорости спавна.
    pub spawn_speed_multiplier: f32, // +0x12CC
}

/// Reference-counted обёртка над Vehicle* (32 байта).
///
/// Используется в C_Garage для управления временем жизни.
/// COM-style refcount: AddRef/Release через vtable.
/// VTable: `vtables::garage::VEHICLE_WRAPPER`
#[repr(C)]
pub struct VehicleWrapper {
    pub vtable: *const c_void, // +0x00
    /// Счётчик ссылок (COM-style).
    /// При достижении 0 — Vehicle уничтожается.
    pub refcount: i32, // +0x08
    _pad_0c: [u8; 0x18 - 0x0C],
    /// Указатель на оборачиваемый Vehicle.
    pub vehicle: *mut Vehicle, // +0x18
}

assert_layout!(VehicleWrapper, size = 32, {
    refcount == 0x08,
    vehicle  == 0x18,
});

assert_field_offsets!(Vehicle {
    spawn_data             == 0xE0,
    speed                  == 0x360,
    anim_param1            == 0x388,
    anim_param2            == 0x394,
    spawn_timestamp        == 0x1248,
    min_spawn_time         == 0x1288,
    max_spawn_time         == 0x128C,
    spawn_progress         == 0x12AC,
    spawn_speed_multiplier == 0x12CC,
});
