//! Типизированные VTable структуры движка.
//!
//! Каждый слот — `unsafe extern "C" fn(...)` с правильной сигнатурой.
//! Неизвестные слоты — `usize` заглушки.
//!
//! VTable не создаются в Rust — они уже лежат в `.rdata` секции игры.
//! Мы только описываем layout чтобы компилятор сам вычислял смещения.

use std::ffi::c_void;
use crate::types::Vec3;

/// VTable `C_Player` / `C_Human` — 83 слота (до `process_damage`).
///
/// Включает слоты от Entity (0-31), Actor spatial (32-49),
/// Human/Player methods (50-82).
///
/// Один и тот же тип используется для Player и NPC —
/// реальная vtable различается, но layout слотов совпадает.
///
/// # Использование
///
/// ```ignore
/// let entity = &*player_ptr;
/// let vt = &*(entity.actor.base.vtable as *const CPlayerVTable);
/// let mut pos = Vec3::ZERO;
/// (vt.get_pos)(player_ptr as *const c_void, &mut pos);
/// ```
#[repr(C)]
pub struct CPlayerVTable {
    // =========================================================================
    //  Entity base (slots 0-31)
    // =========================================================================

    /// [0] Scalar deleting destructor.
    pub dtor: unsafe extern "C" fn(this: *mut c_void, flags: u8),

    /// [1]
    pub _slot_01: usize,

    /// [2] GetFrameNode → returns `*(this+0x78)`.
    pub get_frame_node: unsafe extern "C" fn(this: *const c_void) -> *mut c_void,

    /// [3..31] Entity infrastructure slots.
    pub _slots_03_31: [usize; 29],

    // =========================================================================
    //  Actor spatial interface (slots 32-49)
    // =========================================================================

    /// [32] SetPos — обновляет physics + cache + dirty flags.
    pub set_pos: unsafe extern "C" fn(this: *mut c_void, pos: *const Vec3),

    /// [33]
    pub _slot_33: usize,

    /// [34] SetRotationQuat — принимает `[f32; 4]` (x,y,z,w).
    pub set_rotation_quat: unsafe extern "C" fn(this: *mut c_void, quat: *const [f32; 4]),

    /// [35] SetDir.
    pub set_dir: unsafe extern "C" fn(this: *mut c_void, dir: *const Vec3),

    /// [36] GetPos — основной getter позиции.
    pub get_pos: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,

    /// [37] GetDir — direction из locomotion controller.
    pub get_dir: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,

    /// [38] GetRotation — кватернион.
    pub get_rotation: unsafe extern "C" fn(this: *const c_void, out: *mut [f32; 4]) -> *mut [f32; 4],

    /// [39] GetBoundRadius.
    pub get_bound_radius: unsafe extern "C" fn(this: *const c_void) -> f32,

    /// [40..42]
    pub _slots_40_42: [usize; 3],

    /// [43] GetHeadPos — `GetPos() + Vec3(0, 0, 2.0)`.
    pub get_head_pos: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,

    /// [44..46]
    pub _slots_44_46: [usize; 3],

    /// [47] IsDeath — `return *(u8*)(this + 0x161)`.
    pub is_death: unsafe extern "C" fn(this: *const c_void) -> bool,

    /// [48..49]
    pub _slots_48_49: [usize; 2],

    // =========================================================================
    //  Human / Player methods (slots 50-82)
    // =========================================================================

    /// [50..67]
    pub _slots_50_67: [usize; 18],

    /// [68] GetVelocity — из locomotion controller.
    ///
    /// Подтверждено: vtable\[68\], slot\[25\] locomotion.
    pub get_velocity: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,

    /// [69..74]
    pub _slots_69_74: [usize; 6],

    /// [75] SetMovementSpeed — пишет `+0x294` AND `+0x298`.
    pub set_movement_speed: unsafe extern "C" fn(this: *mut c_void, speed: f32),

    /// [76] GetMovementSpeedCurrent — `return *(float*)(this+0x298)`.
    pub get_movement_speed_current: unsafe extern "C" fn(this: *const c_void) -> f32,

    /// [77] SetMovementSpeedTarget — пишет только `+0x294`.
    pub set_movement_speed_target: unsafe extern "C" fn(this: *mut c_void, speed: f32),

    /// [78..81]
    pub _slots_78_81: [usize; 4],

    /// [82] ProcessDamage.
    pub process_damage: unsafe extern "C" fn(this: *mut c_void, msg: *const c_void) -> u8,
}

// =============================================================================
//  Compile-time проверки
// =============================================================================

const _: () = {
    // 83 слота × 8 байт = 664 байт
    assert!(std::mem::size_of::<CPlayerVTable>() == 83 * 8);

    // Ключевые смещения (slot_index × 8)
    assert!(std::mem::offset_of!(CPlayerVTable, get_pos) == 36 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, set_pos) == 32 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, get_dir) == 37 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, is_death) == 47 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, get_head_pos) == 43 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, get_velocity) == 68 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, set_movement_speed) == 75 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, get_movement_speed_current) == 76 * 8);
    assert!(std::mem::offset_of!(CPlayerVTable, process_damage) == 82 * 8);
};