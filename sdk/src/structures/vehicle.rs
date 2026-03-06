use std::ffi::c_void;

/// High-level Vehicle (Lua API wrapper).
///
/// **НЕ путать с `C_Car`!**
/// - `Vehicle` — высокоуровневая обёртка (для гаража: 184 байт аллокации)
/// - `C_Car` — низкоуровневая сущность движка
///
/// Состояния (`+0x08` для standalone Vehicle):
/// - 6: Initial (только создан)
/// - 5: Deferred (создан, не заспавнен)
/// - 1: Active (полностью инициализирован, C_Car существует)
#[repr(C)]
pub struct Vehicle {
    pub vtable: *const c_void,              // +0x000
    _pad_008: [u8; 0xE0 - 0x08],
    pub spawn_data: *mut c_void,            // +0x0E0 (224)
    _pad_0e8: [u8; 0x360 - 0xE8],
    pub speed: u64,                         // +0x360 (864)
    pub speed_related: u32,                 // +0x368 (872)
    _pad_36c: [u8; 0x388 - 0x36C],
    pub anim_param1: f32,                   // +0x388 (904)
    _pad_38c: [u8; 0x394 - 0x38C],
    pub anim_param2: f32,                   // +0x394 (916)
    _pad_398: [u8; 0x1248 - 0x398],
    pub spawn_timestamp: u64,               // +0x1248 (4680)
    _pad_1250: [u8; 0x1288 - 0x1250],
    pub min_spawn_time: f32,                // +0x1288 (4744)
    pub max_spawn_time: f32,                // +0x128C (4748)
    _pad_1290: [u8; 0x12AC - 0x1290],
    pub spawn_progress: f32,                // +0x12AC (4780) 0.0–1.0
    _pad_12b0: [u8; 0x12CC - 0x12B0],
    pub spawn_speed_multiplier: f32,        // +0x12CC (4812)
}

/// Reference-counted обёртка над Vehicle* (32 байта).
///
/// Используется в `C_Garage` для управления временем жизни.
/// VTable: `vtables::garage::VEHICLE_WRAPPER`
#[repr(C)]
pub struct VehicleWrapper {
    pub vtable: *const c_void,              // +0x00
    pub refcount: i32,                      // +0x08 (COM-style)
    _pad_0c: [u8; 0x18 - 0x0C],
    pub vehicle: *mut Vehicle,              // +0x18
}

const _: () = {
    assert!(std::mem::size_of::<VehicleWrapper>() == 32);
    assert!(std::mem::offset_of!(VehicleWrapper, refcount) == 0x08);
    assert!(std::mem::offset_of!(VehicleWrapper, vehicle) == 0x18);
    assert!(std::mem::offset_of!(Vehicle, spawn_data) == 0xE0);
    assert!(std::mem::offset_of!(Vehicle, speed) == 0x360);
    assert!(std::mem::offset_of!(Vehicle, anim_param1) == 0x388);
    assert!(std::mem::offset_of!(Vehicle, spawn_progress) == 0x12AC);
    assert!(std::mem::offset_of!(Vehicle, spawn_speed_multiplier) == 0x12CC);
};