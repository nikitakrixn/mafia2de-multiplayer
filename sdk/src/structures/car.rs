use std::ffi::c_void;

/// Engine-level сущность машины (отличается от `Vehicle`!).
///
/// Множественное наследование — 3 vtable.
/// VTables: `vtables::car::{MAIN, BASE_2, BASE_3}`
#[repr(C)]
pub struct CCar {
    pub vtable: *const c_void,              // +0x00
    _pad_008: [u8; 0x38 - 0x08],
    pub important_data: *mut CarData,       // +0x38 (56)
}

/// Основные данные машины (по `C_Car::important_data`).
///
/// Содержит физику, рендеринг, bounding box.
#[repr(C)]
pub struct CarData {
    _pad_000: [u8; 0x270],
    pub size_info: *mut c_void,             // +0x270 (624)
    _pad_278: [u8; 0x2F8 - 0x278],
    pub init_info: *mut c_void,             // +0x2F8 (760)
    _pad_300: [u8; 0x328 - 0x300],
    pub bbox_min_x: f64,                    // +0x328 (808)
    pub bbox_min_y: i32,                    // +0x330 (816)
    pub bbox_max_x: f64,                    // +0x334 (820)
    pub bbox_max_y: i32,                    // +0x33C (828)
}

const _: () = {
    assert!(std::mem::offset_of!(CCar, important_data) == 0x38);
    assert!(std::mem::offset_of!(CarData, size_info) == 0x270);
    assert!(std::mem::offset_of!(CarData, bbox_min_x) == 0x328);
};