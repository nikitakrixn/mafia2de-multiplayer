use std::ffi::c_void;

/// Менеджер таблиц данных.
///
/// Конструктор: `functions::tables::CONSTRUCTOR`
/// Загружает все .tbl файлы из /tables/.
#[repr(C)]
pub struct TableManager {
    pub initialized: u8,                        // +0x00 (инвертирован)
    _pad_001: [u8; 0x38 - 0x01],
    pub police_offences_table: *mut c_void,     // +0x38 (56)
    pub weapons_table: *mut c_void,             // +0x40 (64)
    _pad_048: [u8; 0x50 - 0x48],
    pub attack_params_table: *mut c_void,       // +0x50 (80)
    _pad_058: [u8; 0x60 - 0x58],
    pub vehicles_table: *mut c_void,            // +0x60 (96)
    _pad_068: [u8; 0xB8 - 0x68],
    pub phobj_sounds_table: *mut c_void,        // +0xB8 (184)
    _pad_0c0: [u8; 0xC8 - 0xC0],
    pub materials_physics_table: *mut c_void,   // +0xC8 (200)
    pub materials_shots_table: *mut c_void,     // +0xD0 (208)
    pub music_table: *mut c_void,               // +0xD8 (216)
    pub glassbreaking_table: *mut c_void,       // +0xE0 (224)
    pub glassmattemplates_table: *mut c_void,   // +0xE8 (232)
    _pad_0f0: [u8; 0xF8 - 0xF0],
    pub human_dmgzones_table: *mut c_void,      // +0xF8 (248)
    pub pinups_galleries_table: *mut c_void,    // +0x100 (256)
    pub pinups_table: *mut c_void,              // +0x108 (264)
    _pad_110: [u8; 0x150 - 0x110],
    pub rambo_actions_table: *mut c_void,       // +0x150 (336)
}

const _: () = {
    assert!(std::mem::offset_of!(TableManager, vehicles_table) == 0x60);
    assert!(std::mem::offset_of!(TableManager, weapons_table) == 0x40);
    assert!(std::mem::offset_of!(TableManager, rambo_actions_table) == 0x150);
};