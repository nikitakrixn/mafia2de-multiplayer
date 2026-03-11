//! Менеджер таблиц данных (.tbl файлы).
//!
//! Игра загружает все таблицы из /tables/ при старте.
//! Каждая таблица содержит параметры: оружие, машины,
//! материалы, полицейские нарушения и т.д.

use std::ffi::c_void;
use crate::macros::assert_field_offsets;

/// Менеджер таблиц данных.
///
/// Конструктор: `functions::tables::CONSTRUCTOR`
/// Загружает все .tbl файлы из /tables/ при инициализации.
///
/// Поле `initialized` инвертировано: 0 = да, 1 = нет.
#[repr(C)]
pub struct TableManager {
    /// Флаг инициализации (инвертирован: 0 = готов).
    pub initialized: u8,                        // +0x00
    _pad_001: [u8; 0x38 - 0x01],
    /// /tables/police_offences.tbl
    pub police_offences_table: *mut c_void,     // +0x38
    /// /tables/weapons.tbl — параметры оружия, макс. патроны.
    pub weapons_table: *mut c_void,             // +0x40
    _pad_048: [u8; 0x50 - 0x48],
    /// /tables/attack_params.tbl
    pub attack_params_table: *mut c_void,       // +0x50
    _pad_058: [u8; 0x60 - 0x58],
    /// /tables/vehicles.tbl — параметры машин.
    pub vehicles_table: *mut c_void,            // +0x60
    _pad_068: [u8; 0xB8 - 0x68],
    /// /tables/phobj_sounds.tbl
    pub phobj_sounds_table: *mut c_void,        // +0xB8
    _pad_0c0: [u8; 0xC8 - 0xC0],
    /// /tables/materials_physics.tbl
    pub materials_physics_table: *mut c_void,   // +0xC8
    /// /tables/materials_shots.tbl
    pub materials_shots_table: *mut c_void,     // +0xD0
    /// /tables/music.tbl
    pub music_table: *mut c_void,               // +0xD8
    /// /tables/glassbreaking.tbl
    pub glassbreaking_table: *mut c_void,       // +0xE0
    /// /tables/glassmattemplates.tbl
    pub glassmattemplates_table: *mut c_void,   // +0xE8
    _pad_0f0: [u8; 0xF8 - 0xF0],
    /// /tables/human_dmgzones.tbl — зоны урона по телу.
    pub human_dmgzones_table: *mut c_void,      // +0xF8
    /// /tables/pinups_galleries.tbl
    pub pinups_galleries_table: *mut c_void,    // +0x100
    /// /tables/pinups.tbl
    pub pinups_table: *mut c_void,              // +0x108
    _pad_110: [u8; 0x150 - 0x110],
    /// /tables/rambo_actions.tbl
    pub rambo_actions_table: *mut c_void,       // +0x150
}

assert_field_offsets!(TableManager {
    weapons_table       == 0x40,
    vehicles_table      == 0x60,
    rambo_actions_table == 0x150,
});