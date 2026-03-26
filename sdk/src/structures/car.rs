//! Engine-level сущности машин: C_Car и C_CarVehicle.
//!
//! C_Car — базовый класс машины (припаркованные, статические).
//! C_CarVehicle — расширенный класс (управляемый транспорт, 5 vtable).
//!
//! Структуры восстановлены из IDA Pro decompile vtable C_Car.
//! Подтверждены compile-time ассертами на смещения полей.

use super::entity::CEntity;
use crate::macros::assert_field_offsets;
use std::ffi::{CStr, c_void};

// =============================================================================
//  C_Car — базовый класс машины
// =============================================================================

/// Engine-level сущность машины (C_Car).
///
/// Множественное наследование — 5 vtable.
/// Размер аллокации: 0x1258 байт.
///
/// VTables:
/// - +0x00: primary (0x141850030)
/// - +0xE0: sub-vtable 1 (0x141850298)
/// - +0x1E0: sub-vtable 2 (0x141850478)
/// - +0x1E8: sub-vtable 3 (0x1418504C0)
/// - +0x1F8: sub-vtable 4 (0x1418504E0)
/// - +0x210: sub-vtable 5 (0x1418504F0)
#[repr(C)]
pub struct CCar {
    _pad_000: [u8; 0x88],

    /// Attached ops begin (вектор операций).
    pub attached_ops_begin: *mut c_void,        // +0x88
    /// Attached ops end.
    pub attached_ops_end: *mut c_void,          // +0x90
    /// Attached ops capacity.
    pub attached_ops_cap: *mut c_void,          // +0x98

    /// Entity subtype (0x36, 0x37, 0x3A для разных C_Car).
    pub entity_subtype: u32,                    // +0xA0

    _pad_0a4: [u8; 0xB0 - 0xA4],

    /// Pending dispatch begin.
    pub pending_dispatch_begin: *mut c_void,    // +0xB0
    /// Pending dispatch end.
    pub pending_dispatch_end: *mut c_void,      // +0xB8

    _pad_0c0: [u8; 0xC8 - 0xC0],

    /// Records begin (вектор записей).
    pub records_begin: *mut c_void,             // +0xC8
    /// Records end.
    pub records_end: *mut c_void,               // +0xD0
    /// Records capacity.
    pub records_cap: *mut c_void,               // +0xD8

    /// Physics sub-vtable pointer (+0xE0).
    pub physics_sub_vtable: *const c_void,      // +0xE0

    _pad_0e8: [u8; 0x270 - 0xE8],

    /// World matrix 4x4 (row-major, f32[16]).
    /// Подтверждено: IDA decompile C_Car vtable.
    pub world_matrix: [f32; 16],                // +0x270

    _pad_2b0: [u8; 0x2F0 - 0x2B0],

    /// Self-reference (= this). Подтверждено 3 образцами runtime.
    pub self_ref: *mut CCar,                    // +0x2F0

    _pad_2f8: [u8; 0xED8 - 0x2F8],

    /// Physics body pointer.
    pub physics_body: *mut c_void,              // +0xED8

    _pad_ee0: [u8; 0xF10 - 0xEE0],

    /// Behavior component pointer.
    pub behavior: *mut c_void,                  // +0xF10

    _pad_f18: [u8; 0xF30 - 0xF18],

    /// Car flags (u64).
    pub car_flags: u64,                         // +0xF30

    _pad_f38: [u8; 0xF48 - 0xF38],

    /// Template resource pointer.
    pub template_resource: *mut c_void,         // +0xF48

    _pad_f50: [u8; 0xF88 - 0xF50],

    /// Variant index (u32).
    pub variant_index: u32,                     // +0xF88

    _pad_f8c: [u8; 0x11EC - 0xF8C],

    /// Pos committed flag (u8).
    pub pos_committed: u8,                      // +0x11EC

    _pad_11ed: [u8; 0x1210 - 0x11ED],

    /// Collision body pointer.
    pub collision_body: *mut c_void,            // +0x1210

    /// Collision body refcount (i32).
    pub collision_body_refcount: i32,           // +0x1218
}

impl CCar {
    /// Получить позицию из встроенной world matrix.
    ///
    /// Подтверждено decompile `CCar_GetPos`:
    /// - +0x27C = X
    /// - +0x28C = Y
    /// - +0x29C = Z
    ///
    /// Если `world_matrix` начинается с +0x270, то это элементы:
    /// - [3], [7], [11]
    pub fn get_pos(&self) -> (f32, f32, f32) {
        (
            self.world_matrix[3],
            self.world_matrix[7],
            self.world_matrix[11],
        )
    }

    /// Есть ли активная физика (physics_body != NULL).
    pub fn has_physics(&self) -> bool {
        !self.physics_body.is_null()
    }

    /// Установлен ли dirty-флаг.
    ///
    /// Подтверждено `CCar_SetPos` / `CCar_SetRotation`:
    /// `car_flags |= 0x1000`.
    pub fn is_dirty(&self) -> bool {
        (self.car_flags & 0x1000) != 0
    }

    /// Количество записей в records-векторе.
    ///
    /// Подтверждено:
    /// - slot[67] = `(end - begin) / 24`
    /// - slot[68] = `begin + 24 * index`
    pub fn record_count(&self) -> usize {
        let begin = self.records_begin as usize;
        let end = self.records_end as usize;
        if end > begin {
            (end - begin) / 24
        } else {
            0
        }
    }

    /// Валиден ли self_ref (указывает на себя).
    pub fn has_valid_self_ref(&self) -> bool {
        let self_addr = self as *const CCar as usize;
        let ref_addr = self.self_ref as usize;
        ref_addr != 0 && ref_addr == self_addr
    }

    /// Получить указатель на embedded damage subobject (`car + 0xE0`).
    ///
    /// ВАЖНО:
    /// это overlay на inline-подобъект внутри `C_Car`,
    /// а НЕ отдельная аллокация.
    pub fn damage_sub1_ptr(&self) -> *const CCarDamageSub1 {
        (&self.physics_sub_vtable as *const *const c_void) as *const CCarDamageSub1
    }

    /// Это двухдверный кузов?
    ///
    /// Runtime confirmed: subtype `0x3D` имеет 2 двери в `group_a`.
    pub fn is_two_door_body(&self) -> bool {
        self.entity_subtype == 0x3D
    }

    /// Это один из подтверждённых четырёхдверных кузовов?
    pub fn is_four_door_body(&self) -> bool {
        matches!(self.entity_subtype, 0x38 | 0x3A | 0x3B | 0x40)
    }
}

assert_field_offsets!(CCar {
    attached_ops_begin       == 0x88,
    pending_dispatch_begin   == 0xB0,
    records_begin            == 0xC8,
    physics_sub_vtable       == 0xE0,
    world_matrix             == 0x270,
    self_ref                 == 0x2F0,
    physics_body             == 0xED8,
    behavior                 == 0xF10,
    car_flags                == 0xF30,
    template_resource        == 0xF48,
    variant_index            == 0xF88,
    pos_committed            == 0x11EC,
    collision_body           == 0x1210,
    collision_body_refcount  == 0x1218,
});

// =============================================================================
//  C_CarVehicle — управляемый транспорт
// =============================================================================

/// Управляемый транспорт (C_CarVehicle).
///
/// Расширяет C_Car, добавляет 4 дополнительных sub-vtable,
/// physics params, SDS-имена и extended params.
///
/// Размер аллокации: 0x2F0 байт.
///
/// VTables (5 штук):
/// - +0x00: primary (0x1418EAAC8)
/// - +0xA8: sub-vtable 1
/// - +0xB0: sub-vtable 2
/// - +0xB8: sub-vtable 3
/// - +0xC0: sub-vtable 4
#[repr(C)]
pub struct CCarVehicle {
    _pad_000: [u8; 0xA0],

    /// Entity subtype (=3 для C_CarVehicle).
    pub entity_subtype: u32,                    // +0xA0

    _pad_0a4: [u8; 0xA8 - 0xA4],

    /// Sub-vtable 1.
    pub sub_vtable_1: *const c_void,            // +0xA8
    /// Sub-vtable 2.
    pub sub_vtable_2: *const c_void,            // +0xB0
    /// Sub-vtable 3.
    pub sub_vtable_3: *const c_void,            // +0xB8
    /// Sub-vtable 4.
    pub sub_vtable_4: *const c_void,            // +0xC0

    _pad_0c8: [u8; 0xD0 - 0xC8],

    /// Physics params (0x44 байта, inline).
    pub physics_params: [u8; 0x44],             // +0xD0

    _pad_114: [u8; 0x118 - 0x114],

    /// SDS name 1 (cloth slot, 32 байта: u8 flag + char[31]).
    pub sds_name_1: [u8; 32],                   // +0x118
    /// SDS name 2 (body slot, 32 байта).
    pub sds_name_2: [u8; 32],                   // +0x138
    /// SDS name 3 (look slot, 32 байта).
    pub sds_name_3: [u8; 32],                   // +0x158

    /// Extended params (0x30 байт).
    pub extended_params: [u8; 0x30],            // +0x178

    _pad_1a8: [u8; 0x1A8 - 0x1A8],

    /// Global subsystem pointer.
    pub global_subsystem: *mut c_void,          // +0x1A8
}

impl CCarVehicle {
    /// Получить SDS name 1 как строку.
    ///
    /// Подтверждено decompile `InitFromTemplate`:
    /// `strncpy(this+0x118, template+72, 0x1F); this[0x137]=0`
    ///
    /// То есть это обычный `char[32]`, а НЕ `{flag + char[31]}`.
    pub fn get_sds_name_1(&self) -> Option<&str> {
        CStr::from_bytes_until_nul(&self.sds_name_1).ok()?.to_str().ok()
    }

    /// Получить SDS name 2 как строку.
    pub fn get_sds_name_2(&self) -> Option<&str> {
        CStr::from_bytes_until_nul(&self.sds_name_2).ok()?.to_str().ok()
    }

    /// Получить SDS name 3 как строку.
    pub fn get_sds_name_3(&self) -> Option<&str> {
        CStr::from_bytes_until_nul(&self.sds_name_3).ok()?.to_str().ok()
    }
}

assert_field_offsets!(CCarVehicle {
    entity_subtype   == 0xA0,
    sub_vtable_1     == 0xA8,
    sub_vtable_2     == 0xB0,
    sub_vtable_3     == 0xB8,
    sub_vtable_4     == 0xC0,
    physics_params   == 0xD0,
    sds_name_1       == 0x118,
    sds_name_2       == 0x138,
    sds_name_3       == 0x158,
    extended_params  == 0x178,
    global_subsystem == 0x1A8,
});

// =============================================================================
//  CCarDamageSub1 — overlay для car+0xE0 (damage subobject)
// =============================================================================

/// Damage subobject машины (overlay на car+0xE0).
///
/// Не является отдельной аллокацией — это inline-часть CCar начиная с +0xE0.
/// Все смещения относительно начала этого subobject (т.е. car+0xE0 = base).
///
/// Восстановлено из IDA Pro decompile CCarDamageSub1 vtable.
#[repr(C)]
pub struct CCarDamageSub1 {
    /// Vtable pointer (+0x00 от subobject base = car+0xE0).
    pub vtable: *const c_void,                  // +0x00

    _pad_008: [u8; 0x30 - 0x08],

    /// Parts table begin (вектор crash-part записей).
    pub parts_table_begin: *mut c_void,         // +0x30
    /// Parts table end.
    pub parts_table_end: *mut c_void,           // +0x38

    _pad_040: [u8; 0x60 - 0x40],

    /// Active refs begin.
    pub active_refs_begin: *mut c_void,         // +0x60
    /// Active refs end.
    pub active_refs_end: *mut c_void,           // +0x68

    _pad_070: [u8; 0x6B0 - 0x70],

    /// Group A begin.
    pub group_a_begin: *mut c_void,             // +0x6B0
    /// Group A end.
    pub group_a_end: *mut c_void,               // +0x6B8

    _pad_6c0: [u8; 0x6C8 - 0x6C0],

    /// Links begin.
    pub links_begin: *mut c_void,               // +0x6C8
    /// Links end.
    pub links_end: *mut c_void,                 // +0x6D0

    _pad_6d8: [u8; 0x6E0 - 0x6D8],

    /// Group B begin.
    pub group_b_begin: *mut c_void,             // +0x6E0
    /// Group B end.
    pub group_b_end: *mut c_void,               // +0x6E8

    _pad_6f0: [u8; 0x710 - 0x6F0],

    /// Group C begin.
    pub group_c_begin: *mut c_void,             // +0x710
    /// Group C end.
    pub group_c_end: *mut c_void,               // +0x718

    _pad_720: [u8; 0x740 - 0x720],

    /// Group D begin.
    pub group_d_begin: *mut c_void,             // +0x740
    /// Group D end.
    pub group_d_end: *mut c_void,               // +0x748

    _pad_750: [u8; 0x758 - 0x750],

    /// FX group begin.
    pub fx_group_begin: *mut c_void,            // +0x758
    /// FX group end.
    pub fx_group_end: *mut c_void,              // +0x760

    _pad_768: [u8; 0x8A0 - 0x768],

    /// Event buckets begin.
    pub event_buckets_begin: *mut c_void,       // +0x8A0
    /// Event buckets end.
    pub event_buckets_end: *mut c_void,         // +0x8A8

    _pad_8b0: [u8; 0xAA8 - 0x8B0],

    /// Flags AA8 (u32).
    pub flags_aa8: u32,                         // +0xAA8

    _pad_aac: [u8; 0xAB0 - 0xAAC],             // +0xAAC..+0xAAF

    /// Flags AB0 (u64).
    pub flags_ab0: u64,                         // +0xAB0

    /// Flags AB8 (u64).
    pub flags_ab8: u64,                         // +0xAB8

    _pad_ac0: [u8; 0xAC8 - 0xAC0],             // +0xAC0..+0xAC7

    /// FX manager pointer (+0xAC8).
    pub fx_manager_ac8: *mut c_void,            // +0xAC8
}

impl CCarDamageSub1 {
    /// Количество crash-parts в parts_table.
    ///
    /// Это таблица указателей, stride = 8.
    pub fn parts_count(&self) -> usize {
        let begin = self.parts_table_begin as usize;
        let end = self.parts_table_end as usize;
        if end > begin {
            (end - begin) / 8
        } else {
            0
        }
    }

    /// Количество элементов в active_refs.
    ///
    /// Это вектор указателей, stride = 8.
    pub fn active_refs_count(&self) -> usize {
        let begin = self.active_refs_begin as usize;
        let end = self.active_refs_end as usize;
        if end > begin {
            (end - begin) / 8
        } else {
            0
        }
    }

    /// Количество индексов в group A.
    ///
    /// Подтверждено: это массив `u32`, stride = 4.
    pub fn group_a_count(&self) -> usize {
        let begin = self.group_a_begin as usize;
        let end = self.group_a_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество индексов в links.
    pub fn links_count(&self) -> usize {
        let begin = self.links_begin as usize;
        let end = self.links_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество индексов в group B.
    pub fn group_b_count(&self) -> usize {
        let begin = self.group_b_begin as usize;
        let end = self.group_b_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество индексов в group C.
    pub fn group_c_count(&self) -> usize {
        let begin = self.group_c_begin as usize;
        let end = self.group_c_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество индексов в group D.
    pub fn group_d_count(&self) -> usize {
        let begin = self.group_d_begin as usize;
        let end = self.group_d_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество индексов в fx_group.
    pub fn fx_group_count(&self) -> usize {
        let begin = self.fx_group_begin as usize;
        let end = self.fx_group_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество event buckets.
    ///
    /// Подтверждено runtime/decompile: stride = 0x260.
    pub fn event_bucket_count(&self) -> usize {
        let begin = self.event_buckets_begin as usize;
        let end = self.event_buckets_end as usize;
        if end > begin { (end - begin) / 0x260 } else { 0 }
    }
}

assert_field_offsets!(CCarDamageSub1 {
    vtable              == 0x00,
    parts_table_begin   == 0x30,
    parts_table_end     == 0x38,
    active_refs_begin   == 0x60,
    active_refs_end     == 0x68,
    group_a_begin       == 0x6B0,
    group_a_end         == 0x6B8,
    links_begin         == 0x6C8,
    links_end           == 0x6D0,
    group_b_begin       == 0x6E0,
    group_b_end         == 0x6E8,
    group_c_begin       == 0x710,
    group_c_end         == 0x718,
    group_d_begin       == 0x740,
    group_d_end         == 0x748,
    fx_group_begin      == 0x758,
    fx_group_end        == 0x760,
    event_buckets_begin == 0x8A0,
    event_buckets_end   == 0x8A8,
    flags_aa8           == 0xAA8,
    flags_ab0           == 0xAB0,
    flags_ab8           == 0xAB8,
    fx_manager_ac8      == 0xAC8,
});

// =============================================================================
//  Helpers для доступа к базовым классам
// =============================================================================

impl CCar {
    /// Доступ к CEntity (первые 0x78 байт).
    ///
    /// Безопасно: CEntity layout находится в начале CCar
    /// (CCar наследует CEntity → CActor в движке).
    pub fn as_entity(&self) -> &CEntity {
        unsafe { &*(self as *const CCar as *const CEntity) }
    }

    /// Packed table_id через CEntity.
    pub fn table_id(&self) -> u32 {
        self.as_entity().table_id
    }

    /// Factory type byte.
    pub fn factory_type(&self) -> u8 {
        self.as_entity().factory_type()
    }
}