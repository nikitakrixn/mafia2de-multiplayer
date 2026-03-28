//! Типизированная VTable C_Car (114 слотов primary + MI sub-vtables).
//!
//! ## Адреса в .rdata
//!
//! | Vtable | RVA | Описание |
//! |:-------|:----|:---------|
//! | C_Car primary | `0x141850030` | 114 слотов |
//! | C_ActorVehicle | `0x1418CFE50` | Промежуточный (50 слотов Actor) |
//! | Vehicle/PhThingDeform MI | `0x141850298` | +0xE0, ~38 слотов |
//! | Physics contact MI | `0x141850478` | +0x1E0, ~9 слотов |
//! | Simulation MI | `0x1418504C0` | +0x1E8, ~4 слота |
//! | Joint break MI | `0x1418504E0` | +0x1F8, ~3 слота |
//! | Activated MI | `0x1418504F0` | +0x210, ~3 слота |
//!
//! ## Зоны слотов primary vtable
//!
//! | Cлоты | Источник | Назначение |
//! |:-----------|:---------|:-----------|
//! | 0–2 | C_Entity | Жизненный цикл |
//! | 3–16 | C_Entity / C_Car | Инициализация, сериализация |
//! | 17–23 | C_Entity / C_Car | Сообщения, события |
//! | 24–31 | C_Car | CsInterface, UpdateIdle, Update |
//! | 32–39 | C_Car / C_Actor | Позиция, вращение (physics sync) |
//! | 41–47 | C_EntityPos / C_Actor | PRS, CameraPoint, Frame, IsDead |
//! | 48–49 | C_Car | Underwater detection |
//! | 50–71 | C_ActorVehicle | Seat management |
//! | 72–112 | C_Car | Камера, физика, деформация, ИИ |

use crate::types::Vec3;
use std::ffi::c_void;

/// VTable C_Car — 114 слотов (primary interface).
///
/// Наследование: C_Entity -> C_EntityPos -> C_Actor -> C_ActorVehicle -> C_Car.
#[repr(C)]
pub struct CCarVTable {
    // ====================================================================
    //  Жизненный цикл (0–2)
    // ====================================================================
    /// [0] `~C_Car()`. C_Car::~C_Car().
    pub dtor: usize,
    /// [1] Внутренний (MSVC artifact).
    pub _slot_01: usize,
    /// [2] `GetFrameNode()`. `return *(this + 0x78)`.
    pub get_frame_node: unsafe extern "C" fn(this: *const c_void) -> *mut c_void,

    // ====================================================================
    //  Инициализация и сериализация (3–16)
    // ====================================================================
    /// [3] `Init(S_EntityInitProps*)`. C_Car::Init.
    pub init: usize,
    /// [4] `GameInit()`. C_Car::GameInit.
    pub game_init: usize,
    /// [5] `GameDone()`. C_Car::GameDone.
    pub game_done: usize,
    /// [6] `GameRestore()`. C_Car::GameRestore.
    pub game_restore: usize,
    /// [7] `OnActivate()`. C_Car::OnActivate.
    pub on_activate: unsafe extern "C" fn(this: *mut c_void),
    /// [8] `OnDeactivate()`. C_Car::OnDeactivate.
    pub on_deactivate: unsafe extern "C" fn(this: *mut c_void),
    /// [9] `IsActive() const`.
    pub is_active: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [10] Заглушка.
    pub _slot_10: usize,
    /// [11] `GameSavePrerequisite(C_BitStream*)`.
    pub game_save_prerequisite: usize,
    /// [12] `GameSave(C_BitStream*)`. C_Car::GameSave.
    pub game_save: usize,
    /// [13] `GameSaveDependencies(C_BitStream*)`.
    pub game_save_deps: usize,
    /// [14] `GameLoadPrerequisite(C_BitStream*)`.
    pub game_load_prerequisite: usize,
    /// [15] `GameLoad(C_BitStream*)`. C_Car::GameLoad.
    pub game_load: usize,
    /// [16] `GameLoadDependencies(C_BitStream*)`.
    pub game_load_deps: usize,

    // ====================================================================
    //  Сообщения и события (17–31)
    // ====================================================================
    /// [17] `EntityInvalidate(C_Entity*)`.
    pub entity_invalidate: usize,
    /// [18] `AddOnEvent(uint, uint, uint)`.
    pub add_on_event: usize,
    /// [19] `IsSafeSpawn(C_Vector&, float, C_Entity*)`. C_Car::IsSafeSpawn.
    pub is_safe_spawn: usize,
    /// [20] `GetLineIntersection(...)`. C_Car::GetLineIntersection.
    pub get_line_intersection: usize,
    /// [21] `HideAreaAction(bool)`.
    pub hide_area_action: usize,
    /// [22] `RecvMessage(C_EntityMessage*)`. C_Car::RecvMessage. 825B.
    pub recv_message: usize,
    /// [23] `InvalidateRelation(C_Entity*)`.
    pub invalidate_relation: usize,
    /// [24] `GetCsInterface() const`. C_Car::GetCsInterface.
    pub get_cs_interface: usize,
    /// [25] `GetCreateCsInterface()`. C_Car::GetCreateCsInterface.
    pub get_create_cs_interface: usize,
    /// [26] Заглушка.
    pub _slot_26: usize,
    /// [27–28] Заглушки.
    pub _slots_27_28: [usize; 2],
    /// [29] `UpdateIdleFX(float)`. C_Car::UpdateIdleFX.
    pub update_idle_fx: usize,
    /// [30] `UpdateIdleRC(float)`. C_Car::UpdateIdleRC.
    pub update_idle_rc: usize,
    /// [31] `Update(float)`. C_Car::Update. Главный тик.
    pub update: usize,

    // ====================================================================
    //  Пространственный интерфейс (32–39)
    // ====================================================================
    /// [32] `SetPos(C_Vector const&)`. C_Car::SetPos. Physics sync.
    pub set_pos: unsafe extern "C" fn(this: *mut c_void, pos: *const Vec3),
    /// [33] `SetDir(C_Vector const&)`. C_Car::SetDir.
    pub set_dir: unsafe extern "C" fn(this: *mut c_void, dir: *const Vec3),
    /// [34] `SetRot(C_Quat const&)`. C_Car::SetRot.
    pub set_rot: unsafe extern "C" fn(this: *mut c_void, quat: *const [f32; 4]),
    /// [35] `SetScale(float)`. Inherited from C_Actor.
    pub set_scale: unsafe extern "C" fn(this: *mut c_void, scale: f32),
    /// [36] `GetPos()`. Читает из embedded world_matrix.
    pub get_pos: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,
    /// [37] `GetDir()`. Inherited from C_Actor.
    pub get_dir: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,
    /// [38] `GetRot()`. Inherited from C_Actor.
    pub get_rot: unsafe extern "C" fn(this: *const c_void, out: *mut [f32; 4]) -> *mut [f32; 4],
    /// [39] `GetScale()`. Inherited from C_Actor.
    pub get_scale: unsafe extern "C" fn(this: *const c_void) -> f32,

    // ====================================================================
    //  PRS / Frame / Underwater (40–49)
    // ====================================================================
    /// [40] Заглушка.
    pub _slot_40: usize,
    /// [41] `GameSavePRS(C_BitStream*)`.
    pub game_save_prs: usize,
    /// [42] `GameLoadPRS(C_BitStream*)`.
    pub game_load_prs: usize,
    /// [43] `GetCameraPoint()`. через physics_body->vfunc[9].
    pub get_camera_point: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,
    /// [44] `SetFrame(C_Frame*)`.
    pub set_frame: usize,
    /// [45] `SetOwner(C_Entity*)`. C_Actor stub.
    pub set_owner: usize,
    /// [46] Заглушка.
    pub _slot_46: usize,
    /// [47] `IsDead()`. C_Actor base — returns false.
    pub is_dead: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [48] `EnableUnderwaterDetection(bool)`. Refcounted +0x1210.
    pub enable_underwater_detection: usize,
    /// [49] `GetUnderwaterStatus()`.
    pub get_underwater_status: unsafe extern "C" fn(this: *const c_void) -> u32,

    // ====================================================================
    //  Seat management — C_ActorVehicle (50–71)
    // ====================================================================
    /// [50] `Clear()`. C_ActorVehicle::Clear.
    pub av_clear: usize,
    /// [51] `AddSeat(S_BaseSeat*, bool)`.
    pub av_add_seat: usize,
    /// [52] `LockSeat(uint, uint, I_Human2*)`.
    pub av_lock_seat: usize,
    /// [53] `PreLockSeat(uint, uint, I_Human2*)`.
    pub av_pre_lock_seat: usize,
    /// [54] `UnlockSeat(uint, uint)`.
    pub av_unlock_seat: usize,
    /// [55] `IsSeatLocked(uint, uint, I_Human2*)`.
    pub av_is_seat_locked: usize,
    /// [56] `GetSeatCount()`.
    pub av_get_seat_count: unsafe extern "C" fn(this: *const c_void) -> u32,
    /// [57] `GetSeatCount(uint)`.
    pub av_get_seat_count_by_type: unsafe extern "C" fn(this: *const c_void, seat_type: u32) -> u32,
    /// [58] `GetHumanUsedSeatCount()`.
    pub av_get_human_used_seat_count: unsafe extern "C" fn(this: *const c_void) -> u32,
    /// [59] `GetSeat(uint, uint)`.
    pub av_get_seat_by_type_idx: usize,
    /// [60] `GetSeat(uint)`.
    pub av_get_seat_by_idx: usize,
    /// [61] `GetFreeSeatIndex(uint)`.
    pub av_get_free_seat_index: unsafe extern "C" fn(this: *const c_void, seat_type: u32) -> i32,
    /// [62] `GetMySeatIndex(uint, uint*, uint*, uint*)`.
    pub av_get_my_seat_index: usize,
    /// [63] `SetSeatStatus(uint, E_BaseSeatStatus)`. C_Car override.
    pub set_seat_status: usize,
    /// [64] `GetSeatStatus(uint, E_BaseSeatStatus&)`.
    pub av_get_seat_status: usize,
    /// [65] Заглушка.
    pub _slot_65: usize,
    /// [66] `AddEnterLeave(S_BusEnterLeave const&)`.
    pub av_add_enter_leave: usize,
    /// [67–68] Заглушки.
    pub _slots_67_68: [usize; 2],
    /// [69] `DisableActionSit(uint, uint, bool)`.
    pub av_disable_action_sit: usize,
    /// [70] `PreLockActionSit(uint, uint, I_Human2*)`.
    pub av_pre_lock_action_sit: usize,
    /// [71] `FindAction(uint, uint)`.
    pub av_find_action: usize,

    // ====================================================================
    //  C_Car specific (72–112)
    // ====================================================================
    /// [72] Заглушка.
    pub _slot_72: usize,
    /// [73] `CameraPos()`. C_Car::CameraPos.
    pub camera_pos: usize,
    /// [74] `CameraDir()`. C_Car::CameraDir.
    pub camera_dir: usize,
    /// [75] `IsModelRendered()`.
    pub is_model_rendered: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [76] `IsInOptimPlayerRange()`.
    pub is_in_optim_player_range: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [77] `IsVehicleInOpenSpace()`.
    pub is_vehicle_in_open_space: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [78] Заглушка.
    pub _slot_78: usize,
    /// [79] `OnBeforeLoad()`.
    pub on_before_load: usize,
    /// [80] `SetHorn(bool, bool)`.
    pub set_horn: unsafe extern "C" fn(this: *mut c_void, enabled: u8, honk_type: u8),
    /// [81] `SwitchSettingsTable(E_VehicleSettingsTable)`.
    pub switch_settings_table: usize,
    /// [82] `OnSaveOther(C_SaveLoadManager&)`.
    pub on_save_other: usize,
    /// [83] `OnLoadOther(C_SaveLoadManager&)`.
    pub on_load_other: usize,
    /// [84] `ExplodeFireFinish()`.
    pub explode_fire_finish: usize,
    /// [85] `AcceptObject(C_Frame*, bool&, bool&)`.
    pub accept_object: usize,
    /// [86] `SetVehicleDirty(float)`.
    pub set_vehicle_dirty: unsafe extern "C" fn(this: *mut c_void, amount: f32),
    /// [87] `SetSPZText(char const*, bool)`. Номерной знак.
    pub set_spz_text: unsafe extern "C" fn(this: *mut c_void, text: *const i8, update: u8),
    /// [88] `GetSPZText()`.
    pub get_spz_text: unsafe extern "C" fn(this: *const c_void) -> *const i8,
    /// [89] `OnSimulationFinished(float, I_IdleCallback const&)`.
    pub on_simulation_finished: usize,
    /// [90] `OnRegisterToTick(bool)`.
    pub on_register_to_tick: usize,
    /// [91] `OnTyreCrash(S_Wheel&)`.
    pub on_tyre_crash: usize,
    /// [92] `OnTyreOff(S_Wheel&)`.
    pub on_tyre_off: usize,
    /// [93] `DoorChangeState(E_DoorChangeState)`.
    pub door_change_state: usize,
    /// [94] `HoodAndTrunkChangeState(E_HoodAndTrunkChangeState)`.
    pub hood_and_trunk_change_state: usize,
    /// [95] `DeformPartEnergyChanged(C_DeformPart*, float)`.
    pub deform_part_energy_changed: usize,
    /// [96–97] Заглушки.
    pub _slots_96_97: [usize; 2],
    /// [98] `OnContactStart(S_ContactEventInfo const&)`.
    pub on_contact_start: usize,
    /// [99] `OnContactTouch(S_ContactEventInfo const&)`.
    pub on_contact_touch: usize,
    /// [100] `VehicleTransform()`.
    pub vehicle_transform: usize,
    /// [101] `OnSimulationStep(float, I_IdleCallback const&)`.
    pub on_simulation_step: usize,
    /// [102] `ChangeCollElementsCount(int, int)`.
    pub change_coll_elements_count: usize,
    /// [103] `GetInitData(uint64 const&)`.
    pub get_init_data: usize,
    /// [104] `DropPart(C_Deformation*, int, bool)`.
    pub drop_part: usize,
    /// [105] `IsValidSoundContact(CntPtr<C_RigidBody>)`.
    pub is_valid_sound_contact: usize,
    /// [106] `IsValidEffectContact(CntPtr<C_RigidBody>)`.
    pub is_valid_effect_contact: usize,
    /// [107] `IsValidBloodContact(CntPtr<C_RigidBody>)`.
    pub is_valid_blood_contact: usize,
    /// [108] `IsValidSpawnContact(CntPtr<C_RBElement>)`.
    pub is_valid_spawn_contact: usize,
    /// [109] Заглушка.
    pub _slot_109: usize,
    /// [110] `Register2AI()`. C_Car::Register2AI.
    pub register_2_ai: usize,
    /// [111] `UnregisterFromAI()`. C_Car::UnregisterFromAI.
    pub unregister_from_ai: usize,
    /// [112] `AddCollisionContact(...)`. C_Car::AddCollisionContact.
    pub add_collision_contact: usize,
    /// [113] Заглушка (последний слот).
    pub _slot_113: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CCarVTable>() == 114 * 8);

    assert!(std::mem::offset_of!(CCarVTable, get_frame_node) == 2 * 8);
    assert!(std::mem::offset_of!(CCarVTable, on_activate) == 7 * 8);
    assert!(std::mem::offset_of!(CCarVTable, is_active) == 9 * 8);
    assert!(std::mem::offset_of!(CCarVTable, set_pos) == 32 * 8);
    assert!(std::mem::offset_of!(CCarVTable, set_dir) == 33 * 8);
    assert!(std::mem::offset_of!(CCarVTable, set_rot) == 34 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_pos) == 36 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_dir) == 37 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_rot) == 38 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_scale) == 39 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_camera_point) == 43 * 8);
    assert!(std::mem::offset_of!(CCarVTable, is_dead) == 47 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_underwater_status) == 49 * 8);
    assert!(std::mem::offset_of!(CCarVTable, av_clear) == 50 * 8);
    assert!(std::mem::offset_of!(CCarVTable, av_get_seat_count) == 56 * 8);
    assert!(std::mem::offset_of!(CCarVTable, av_get_human_used_seat_count) == 58 * 8);
    assert!(std::mem::offset_of!(CCarVTable, av_get_free_seat_index) == 61 * 8);
    assert!(std::mem::offset_of!(CCarVTable, is_model_rendered) == 75 * 8);
    assert!(std::mem::offset_of!(CCarVTable, is_in_optim_player_range) == 76 * 8);
    assert!(std::mem::offset_of!(CCarVTable, is_vehicle_in_open_space) == 77 * 8);
    assert!(std::mem::offset_of!(CCarVTable, set_horn) == 80 * 8);
    assert!(std::mem::offset_of!(CCarVTable, set_vehicle_dirty) == 86 * 8);
    assert!(std::mem::offset_of!(CCarVTable, set_spz_text) == 87 * 8);
    assert!(std::mem::offset_of!(CCarVTable, get_spz_text) == 88 * 8);
    assert!(std::mem::offset_of!(CCarVTable, door_change_state) == 93 * 8);
    assert!(std::mem::offset_of!(CCarVTable, register_2_ai) == 110 * 8);
    assert!(std::mem::offset_of!(CCarVTable, add_collision_contact) == 112 * 8);
};