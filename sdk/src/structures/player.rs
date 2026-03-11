//! Структура игрока (C_Human).

use std::ffi::c_void;
use super::Inventory;
use crate::macros::assert_field_offsets;

/// Игрок (наследует Entity).
///
/// Движок использует C_Human для всех humanoid-сущностей.
/// Для локального игрока указатель берётся из GameManager.
///
/// Основные поля:
/// - `frame_node` — transform node с матрицей 4x4 (позиция мира)
/// - `inventory` — инвентарь с оружием и деньгами
/// - `control_component` — управление (блокировка, стиль)
#[repr(C)]
pub struct CHuman {
    pub vtable: *const c_void,          // +0x00
    _pad_008: [u8; 0x78 - 0x08],
    /// Frame/transform node — хранит мировую позицию.
    /// Координаты: frame+0x64 (X), frame+0x74 (Y), frame+0x84 (Z).
    pub frame_node: *mut c_void,        // +0x78
    _pad_080: [u8; 0xE8 - 0x80],
    /// Инвентарь игрока (оружие, деньги, предметы).
    pub inventory: *mut Inventory,      // +0xE8
    /// Компонент управления (блокировка ввода, стиль боя).
    pub control_component: *mut c_void, // +0xF0
}

assert_field_offsets!(CHuman {
    frame_node        == 0x78,
    inventory         == 0xE8,
    control_component == 0xF0,
});