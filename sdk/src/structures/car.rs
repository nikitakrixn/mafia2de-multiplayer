//! Сущности транспорта: C_Car и C_CarVehicle.
//!
//! ```text
//! C_Entity (0x78)
//!   └─ C_EntityPos (spatial pure virtuals)
//!        └─ C_Actor (0xA8, frame_node, owner, components)
//!             └─ C_ActorVehicle (VT: 0x1418CFE50, +seats, +enter/leave)
//!                  │  Конструктор: M2DE_CActorVehicle_Construct (0x140C0EB10)
//!                  │  Зануляет 7 полей: +0xA8..+0xD8
//!                  │
//!                  └─ C_Car (VT: 0x141850030, ft=0x12)
//!                       │  Конструктор: M2DE_CCar_Construct (0x1400EE6C0)
//!                       │
//!                       │  Множественное наследование (5 MI sub-vtable):
//!                       ├─ +0x0E0: C_Vehicle / C_PhThingDeform MI
//!                       │          (VT: 0x141850298)
//!                       │          Методы: OnSavePart, CreatePart, DeletePart,
//!                       │          InitRigidBody, DeformChangeStep1/2, JointBreak...
//!                       ├─ +0x1E0: Physics contact handler
//!                       │          (VT: 0x141850478)
//!                       ├─ +0x1E8: Simulation callback
//!                       │          (VT: 0x1418504C0)
//!                       ├─ +0x1F8: Joint break handler
//!                       │          (VT: 0x1418504E0)
//!                       └─ +0x210: Activated callback
//!                                  (VT: 0x1418504F0)
//! ```
//!
//! ## Важно: C_ActorVehicle ≠ C_ActorDeform!
//!
//! C_Car получает деформацию через `C_Vehicle` MI, а не через `C_ActorDeform`.
//!
//! ## C_CarVehicle ft=0x70

use super::entity::CEntity;
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

// =============================================================================
//  C_Car — сущность машины (ft=0x12, ~0x1258 байт)
// =============================================================================

/// Engine-level сущность машины (C_Car).
///
/// **Размер аллокации: ~0x1258 байт**.
/// **Runtime count в FreeRide: 41 entity**.
///
/// ## Цепочка конструктора
///
/// ```text
/// C_Car::C_Car()
///   1. C_ActorVehicle::C_ActorVehicle(this)     — Actor + seats
///   2. C_Vehicle::C_Vehicle(this + 0xE0)        — MI subobject at +0xE0
///   3. *this = VT_CCar                          — primary vtable
///   4. *(this+0x0E0) = VT_CCar_Vehicle          — Vehicle/PhThingDeform MI
///   5. *(this+0x1E0) = VT_CCar_PhysicsContact   — contact handler MI
///   6. *(this+0x1E8) = VT_CCar_SimCallback       — simulation MI
///   7. *(this+0x1F8) = VT_CCar_JointBreak        — joint break MI
///   8. *(this+0x210) = VT_CCar_Activated          — activated MI
///   9. C_Entity::SetType(this, 0x12)
/// ```
///
/// ## Vtable слоты primary
///
/// | Слоты | Метод | Источник |
/// |:----:|:------|:---------|
/// | 3 | `Init(S_EntityInitProps*)` | C_Car override |
/// | 4-8 | `GameInit/Done/Restore/OnActivate/OnDeactivate` | C_Car overrides |
/// | 12/15 | `GameSave/GameLoad` | C_Car override |
/// | 19 | `IsSafeSpawn(C_Vector&, float, C_Entity*)` | C_Car specific |
/// | 22 | `RecvMessage(C_EntityMessage*)` | C_Car: 825B router |
/// | 24-25 | `GetCsInterface / GetCreateCsInterface` | C_Car override |
/// | 29-31 | `UpdateIdleFX / UpdateIdleRC / Update` | C_Car specific |
/// | 32-34 | `SetPos / SetDir / SetRot` | C_Car: physics sync |
/// | 48-49 | `EnableUnderwaterDetection / GetUnderwaterStatus` | C_Car (MAC slot[49/50]) |
/// | 50-71 | Seat management (C_ActorVehicle) | Clear/AddSeat/Lock/Unlock/... |
/// | 73-77 | `CameraPos/Dir, IsModelRendered, IsInOptimPlayerRange` | C_Car specific |
/// | 80 | `SetHorn(bool, bool)` | C_Car specific |
/// | 87-88 | `SetSPZText / GetSPZText` | Номерной знак |
/// | 89-90 | `OnSimulationFinished / OnRegisterToTick` | Физика |
/// | 91-92 | `OnTyreCrash / OnTyreOff` | Повреждение шин |
/// | 93-94 | `DoorChangeState / HoodAndTrunkChangeState` | Двери/капот |
/// | 95 | `DeformPartEnergyChanged` | Деформация |
/// | 98-99 | `OnContactStart / OnContactTouch` | Контакты |
/// | 100 | `VehicleTransform` | Трансформация |
/// | 101-102 | `OnSimulationStep / ChangeCollElementsCount` | Физика |
/// | 103-104 | `GetInitData / DropPart` | Деформация |
/// | 105-108 | `IsValidSound/Effect/Blood/SpawnContact` | Фильтры контактов |
/// | 110-111 | `Register2AI / UnregisterFromAI` | Регистрация в ИИ |
/// | 112 | `AddCollisionContact` | Запись столкновения |
#[repr(C)]
pub struct CCar {
    _pad_000: [u8; 0x88],

    /// Attached ops begin (вектор операций).
    pub attached_ops_begin: *mut c_void, // +0x88
    /// Attached ops end.
    pub attached_ops_end: *mut c_void, // +0x90
    /// Attached ops capacity.
    pub attached_ops_cap: *mut c_void, // +0x98

    /// Подтип сущности (0x36, 0x37, 0x3A для разных кузовов C_Car).
    pub entity_subtype: u32, // +0xA0

    _pad_0a4: [u8; 0xB0 - 0xA4],

    /// Pending dispatch begin.
    pub pending_dispatch_begin: *mut c_void, // +0xB0
    /// Pending dispatch end.
    pub pending_dispatch_end: *mut c_void, // +0xB8

    _pad_0c0: [u8; 0xC8 - 0xC0],

    /// Records begin (вектор записей, stride=24).
    /// Vtable[67] `GetRecordCount`: `(end - begin) / 24`.
    /// Vtable[68] `GetRecordByIndex`: `begin + 24 * index`.
    pub records_begin: *mut c_void, // +0xC8
    /// Records end.
    pub records_end: *mut c_void, // +0xD0
    /// Records capacity.
    pub records_cap: *mut c_void, // +0xD8

    /// MI sub-vtable: C_Vehicle / C_PhThingDeform.
    ///
    /// VT: `M2DE_VT_CCar_Vehicle_PhThingDeform` (0x141850298).
    ///
    /// Содержит методы деформации и физики:
    /// OnSavePart, OnLoadPart, CreatePart, DeletePart,
    /// InitRigidBody, InitJoint, DoneRigidBody, DoneJoint,
    /// ChangeJoint, ChangeRigid, UpdateVolumesMassAndCOM,
    /// RemoveVolumes, DeformChangeStep1/2, JointBreak, etc.
    pub vehicle_mi_vtable: *const c_void, // +0xE0

    _pad_0e8: [u8; 0x1E0 - 0xE8],

    /// MI sub-vtable 2: Physics contact handler.
    /// VT: 0x141850478.
    /// Содержит OnContactStart thunks.
    pub physics_contact_vtable: *const c_void, // +0x1E0

    /// MI sub-vtable 3: Simulation callback.
    /// VT: 0x1418504C0.
    /// Содержит OnSimulationStep/Finished thunks.
    pub simulation_callback_vtable: *const c_void, // +0x1E8

    _pad_1f0: [u8; 0x1F8 - 0x1F0],

    /// MI sub-vtable 4: Joint break handler.
    /// VT: 0x1418504E0.
    /// Содержит OnUniversalRBJointBreak.
    pub joint_break_vtable: *const c_void, // +0x1F8

    _pad_200: [u8; 0x210 - 0x200],

    /// MI sub-vtable 5: Activated callback.
    /// VT: 0x1418504F0.
    /// Содержит OnActivated.
    pub activated_vtable: *const c_void, // +0x210

    _pad_218: [u8; 0x270 - 0x218],

    /// Матрица мира 4x4 (row-major, f32[16]).
    ///
    /// Vtable[36] `GetPos()` читает:
    /// - +0x27C = X (matrix[3])
    /// - +0x28C = Y (matrix[7])
    /// - +0x29C = Z (matrix[11])
    pub world_matrix: [f32; 16], // +0x270

    _pad_2b0: [u8; 0x2F0 - 0x2B0],

    /// Self-reference (`= this`). Подтверждено 3 образцами runtime.
    pub self_ref: *mut CCar, // +0x2F0

    _pad_2f8: [u8; 0xED8 - 0x2F8],

    /// Physics body pointer.
    /// Vtable[43] `GetCameraPoint`: вызывает `physics_body->vfunc[9]`.
    pub physics_body: *mut c_void, // +0xED8

    _pad_ee0: [u8; 0xF10 - 0xEE0],

    /// Behavior component pointer.
    pub behavior: *mut c_void, // +0xF10

    _pad_f18: [u8; 0xF30 - 0xF18],

    /// Car flags (u64).
    /// Vtable[32] `SetPos`: ORs `0x1000` (dirty flag).
    pub car_flags: u64, // +0xF30

    _pad_f38: [u8; 0xF48 - 0xF38],

    /// Template resource pointer.
    /// Записывается в vtable[3] `Init`.
    pub template_resource: *mut c_void, // +0xF48

    _pad_f50: [u8; 0xF88 - 0xF50],

    /// Variant index (u32).
    /// Vtable[73] `SetVariant` (в M2DE-specific слотах).
    pub variant_index: u32, // +0xF88

    _pad_f8c: [u8; 0x11EC - 0xF8C],

    /// Pos committed flag (u8).
    pub pos_committed: u8, // +0x11EC

    _pad_11ed: [u8; 0x1210 - 0x11ED],

    /// Collision body pointer.
    /// Vtable[48] `EnableUnderwaterDetection`: refcounted.
    pub collision_body: *mut c_void, // +0x1210

    /// Collision body refcount (i32).
    pub collision_body_refcount: i32, // +0x1218
}

impl CCar {
    /// Получить позицию из встроенной world matrix.
    ///
    /// M2DE: vtable[36] читает matrix[3], matrix[7], matrix[11].
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

    /// Установлен ли dirty-флаг (0x1000 в car_flags).
    pub fn is_dirty(&self) -> bool {
        (self.car_flags & 0x1000) != 0
    }

    /// Количество записей в records-векторе.
    pub fn record_count(&self) -> usize {
        let begin = self.records_begin as usize;
        let end = self.records_end as usize;
        if end > begin { (end - begin) / 24 } else { 0 }
    }

    /// Валиден ли self_ref (указывает на себя).
    pub fn has_valid_self_ref(&self) -> bool {
        let self_addr = self as *const CCar as usize;
        let ref_addr = self.self_ref as usize;
        ref_addr != 0 && ref_addr == self_addr
    }

    /// Указатель на inline damage subobject (car + 0xE0).
    pub fn damage_sub1_ptr(&self) -> *const CCarDamageSub1 {
        (&self.vehicle_mi_vtable as *const *const c_void) as *const CCarDamageSub1
    }

    /// Доступ к CEntity (первые 0x78 байт).
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

assert_field_offsets!(CCar {
    attached_ops_begin       == 0x88,
    pending_dispatch_begin   == 0xB0,
    records_begin            == 0xC8,
    vehicle_mi_vtable        == 0xE0,
    physics_contact_vtable   == 0x1E0,
    simulation_callback_vtable == 0x1E8,
    joint_break_vtable       == 0x1F8,
    activated_vtable         == 0x210,
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
//  C_CarVehicle — управляемый транспорт ft=0x70
// =============================================================================

/// Управляемый транспорт (C_CarVehicle).
///
/// ## Конструктор (`M2DE_CCarVehicle_Construct`, 0x140DF3360)
///
/// ```text
/// 1. M2DE_CActor_Construct(this)
/// 2. Установка vtable + 4 sub-vtable
/// 3. Инициализация 6 transform слотов DefaultEntityTransform
/// 4. M2DE_InitContainer58_WithSentinel30(this+0x1D0)
/// 5. M2DE_SmartPtr_AssignAddRef(this+0x2A8, NULL)
/// 6. M2DE_Entity_SetTypeID(this, 0x70)
/// ```
///
/// ## Отличия от C_Car
///
/// | Характеристика | C_Car | C_CarVehicle |
/// |:---------------|:------|:-------------|
/// | Размер | ~0x1258 | 0x2F0 |
/// | Factory type | 0x12 | 0x70 |
/// | Базовый класс | C_ActorVehicle | C_Actor (без seats!) |
/// | Деформация | Через C_Vehicle MI | Нет |
/// | Inline строки | Нет | 3 × 0x20 (SDS names) |
/// | Runtime count | 41 (FreeRide) | 1 (FreeRide) |
#[repr(C)]
pub struct CCarVehicle {
    _pad_000: [u8; 0xA0],

    /// Entity subtype (=3 для C_CarVehicle).
    pub entity_subtype: u32, // +0xA0

    _pad_0a4: [u8; 0xA8 - 0xA4],

    /// Sub-vtable 1 (0x1418EAC60).
    pub sub_vtable_1: *const c_void, // +0xA8
    /// Sub-vtable 2 (0x1418EAC98).
    pub sub_vtable_2: *const c_void, // +0xB0
    /// Sub-vtable 3 (0x1418EACC8).
    pub sub_vtable_3: *const c_void, // +0xB8
    /// Sub-vtable 4 (0x1418EACD8).
    pub sub_vtable_4: *const c_void, // +0xC0

    _pad_0c8: [u8; 0xD0 - 0xC8],

    /// Transform слоты (6 × Vec3 = 0x48 байт, DefaultEntityTransform).
    pub transform_slots: [u8; 0x48], // +0xD0

    /// SDS name slot 1 (cloth, 32 байта)
    pub sds_cloth_name: [u8; 32], // +0x118
    /// SDS name slot 2 (body, 32 байта)
    pub sds_body_name: [u8; 32], // +0x138
    /// SDS name slot 3 (look, 32 байта)
    pub sds_look_name: [u8; 32], // +0x158

    _pad_178: [u8; 0x1A8 - 0x178],

    /// Ref ptr 1.
    pub ref_ptr_1: *mut c_void, // +0x1A8

    _pad_1b0: [u8; 0x2E0 - 0x1B0],

    /// Damping factor (f32 = 0.3).
    pub damping_factor: f32, // +0x2E0
}

impl CCarVehicle {
    /// Получить SDS cloth name как строку.
    pub fn get_cloth_name(&self) -> Option<&str> {
        std::ffi::CStr::from_bytes_until_nul(&self.sds_cloth_name)
            .ok()?
            .to_str()
            .ok()
    }

    /// Получить SDS body name как строку.
    pub fn get_body_name(&self) -> Option<&str> {
        std::ffi::CStr::from_bytes_until_nul(&self.sds_body_name)
            .ok()?
            .to_str()
            .ok()
    }

    /// Получить SDS look name как строку.
    pub fn get_look_name(&self) -> Option<&str> {
        std::ffi::CStr::from_bytes_until_nul(&self.sds_look_name)
            .ok()?
            .to_str()
            .ok()
    }
}

assert_field_offsets!(CCarVehicle {
    entity_subtype   == 0xA0,
    sub_vtable_1     == 0xA8,
    sub_vtable_2     == 0xB0,
    sub_vtable_3     == 0xB8,
    sub_vtable_4     == 0xC0,
    transform_slots  == 0xD0,
    sds_cloth_name   == 0x118,
    sds_body_name    == 0x138,
    sds_look_name    == 0x158,
    ref_ptr_1        == 0x1A8,
    damping_factor   == 0x2E0,
});

// =============================================================================
//  CCarDamageSub1 — overlay для car+0xE0 (damage subobject)
// =============================================================================

/// Damage subobject машины (overlay на car+0xE0).
///
/// Это inline-часть CCar, **не** отдельная аллокация.
///
/// Содержит данные деформации кузова (C_PhThingDeform):
/// - parts_table: таблица crash-частей
/// - group_a: двери (doors)
/// - links: стёкла / связанные элементы
/// - group_b: капоты/багажники (covers)
/// - group_c: выхлоп (exhaust)
/// - group_d: бамперы (bumpers)
/// - fx_group: эффекты столкновений / обломки
/// - event_buckets: dispatch-буферы событий деформации
#[repr(C)]
pub struct CCarDamageSub1 {
    /// Vtable pointer (+0x00 = car+0xE0).
    /// MAC: `ue::game::vehicle::C_Vehicle` vtable.
    pub vtable: *const c_void, // +0x00

    _pad_008: [u8; 0x30 - 0x08],

    /// Parts table begin (вектор crash-part указателей).
    pub parts_table_begin: *mut c_void, // +0x30
    /// Parts table end.
    pub parts_table_end: *mut c_void, // +0x38

    _pad_040: [u8; 0x60 - 0x40],

    /// Active refs begin.
    pub active_refs_begin: *mut c_void, // +0x60
    /// Active refs end.
    pub active_refs_end: *mut c_void, // +0x68

    _pad_070: [u8; 0x6B0 - 0x70],

    /// Group A begin (двери, stride=4).
    pub group_a_begin: *mut c_void, // +0x6B0
    /// Group A end.
    pub group_a_end: *mut c_void, // +0x6B8

    _pad_6c0: [u8; 0x6C8 - 0x6C0],

    /// Links begin (стёкла / связанные части, stride=4).
    pub links_begin: *mut c_void, // +0x6C8
    /// Links end.
    pub links_end: *mut c_void, // +0x6D0

    _pad_6d8: [u8; 0x6E0 - 0x6D8],

    /// Group B begin (капот/багажник, stride=4).
    pub group_b_begin: *mut c_void, // +0x6E0
    /// Group B end.
    pub group_b_end: *mut c_void, // +0x6E8

    _pad_6f0: [u8; 0x710 - 0x6F0],

    /// Group C begin (выхлоп, stride=4).
    pub group_c_begin: *mut c_void, // +0x710
    /// Group C end.
    pub group_c_end: *mut c_void, // +0x718

    _pad_720: [u8; 0x740 - 0x720],

    /// Group D begin (бамперы, stride=4).
    pub group_d_begin: *mut c_void, // +0x740
    /// Group D end.
    pub group_d_end: *mut c_void, // +0x748

    _pad_750: [u8; 0x758 - 0x750],

    /// FX group begin (эффекты столкновений, stride=4).
    pub fx_group_begin: *mut c_void, // +0x758
    /// FX group end.
    pub fx_group_end: *mut c_void, // +0x760

    _pad_768: [u8; 0x8A0 - 0x768],

    /// Event buckets begin (dispatch-буферы, stride=0x260).
    pub event_buckets_begin: *mut c_void, // +0x8A0
    /// Event buckets end.
    pub event_buckets_end: *mut c_void, // +0x8A8

    _pad_8b0: [u8; 0xAA8 - 0x8B0],

    /// Flags AA8 (u32).
    pub flags_aa8: u32, // +0xAA8
    _pad_aac: [u8; 0xAB0 - 0xAAC],
    /// Flags AB0 (u64).
    pub flags_ab0: u64, // +0xAB0
    /// Flags AB8 (u64).
    pub flags_ab8: u64, // +0xAB8
    _pad_ac0: [u8; 0xAC8 - 0xAC0],
    /// FX manager pointer.
    pub fx_manager_ac8: *mut c_void, // +0xAC8
}

impl CCarDamageSub1 {
    /// Количество crash-parts (stride = 8).
    pub fn parts_count(&self) -> usize {
        let begin = self.parts_table_begin as usize;
        let end = self.parts_table_end as usize;
        if end > begin { (end - begin) / 8 } else { 0 }
    }

    /// Количество дверей в group A (stride = 4).
    pub fn doors_count(&self) -> usize {
        let begin = self.group_a_begin as usize;
        let end = self.group_a_end as usize;
        if end > begin { (end - begin) / 4 } else { 0 }
    }

    /// Количество event buckets (stride = 0x260).
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