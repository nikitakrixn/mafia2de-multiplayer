//! Типизированная VTable гуманоидов (110 слотов).
//!
//! VTable лежит в `.rdata` секции игры — мы описываем layout
//! для корректного вычисления смещений компилятором.
//!
//! ## Адреса в .rdata
//!
//! | Класс | VTable RVA | Слотов |
//! |:------|:-----------|:------:|
//! | C_Entity | `0x14186CAC8` | 32 |
//! | C_Actor | `0x14186D050` | 50 |
//! | I_Human2 (abstract) | `0x1418E2BD8` | 110 (purecall после 49) |
//! | C_Human2 (NPC) | `0x1418E5188` | 114 |
//! | C_Player2 | `0x14184C060` | 110 |
//!
//! ## Зоны слотов
//!
//! | Слоты | Источник | Назначение |
//! |:------|:---------|:-----------|
//! | 0–2 | C_Entity | Жизненный цикл |
//! | 3–16 | C_Entity | Инициализация, сериализация |
//! | 17–31 | C_Entity | Сообщения, события |
//! | 32–44 | C_EntityPos / C_Actor | Позиция, вращение, масштаб, PRS, frame |
//! | 45–49 | I_Human2 | Owner, IsDead, underwater |
//! | 50–54 | I_Human2 / C_Human2 | Tick, компонентные геттеры |
//! | 55–64 | C_Human2 | Управление моделями |
//! | 65–68 | C_Human2 | Запросы позиции/направления/скорости |
//! | 69–82 | C_Human2 | Геймплей: бой, прозрачность, урон |
//! | 83–109 | C_Human2 / C_Player2 | Управление игроком |

use crate::types::Vec3;
use std::ffi::c_void;

/// VTable гуманоидов — 110 слотов.
#[repr(C)]
pub struct CHumanVTable {
    // ====================================================================
    //  Жизненный цикл (0–2)
    // ====================================================================
    /// [0] Деструктор (scalar deleting).
    pub dtor: unsafe extern "C" fn(this: *mut c_void, flags: u8),
    /// [1] Внутренний. Все сущности возвращают 1.
    pub _slot_01: usize,
    /// [2] `return *(this + 0x78)`. Возвращает frame-узел.
    pub get_frame_node: unsafe extern "C" fn(this: *const c_void) -> *mut c_void,

    // ====================================================================
    //  Инициализация и сериализация (3–16)
    // ====================================================================
    /// [3] `Init(S_EntityInitProps const*)`. Инициализация из SDS.
    pub init: usize,
    /// [4] `GameInit()`.
    pub game_init: usize,
    /// [5] `GameDone()`.
    pub game_done: usize,
    /// [6] `GameRestore()`. Player: регистрация в PhysicsWorld.
    pub game_restore: usize,
    /// [7] `OnActivate()`.
    pub on_activate: usize,
    /// [8] `OnDeactivate()`.
    pub on_deactivate: usize,
    /// [9] `IsActive() const`.
    pub is_active: usize,
    /// [10] Заглушка (`return 0`).
    pub _slot_10: usize,
    /// [11] `GameSavePrerequisite(C_BitStream*)`.
    pub game_save_prerequisite: usize,
    /// [12] `GameSave(C_BitStream*)`.
    pub game_save: usize,
    /// [13] `GameSaveDependencies(C_BitStream*)`.
    pub game_save_deps: usize,
    /// [14] `GameLoadPrerequisite(C_BitStream*)`.
    pub game_load_prerequisite: usize,
    /// [15] `GameLoad(C_BitStream*)`.
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
    /// [19] Заглушка (`return 1`). В C_Car: `IsSafeSpawn`.
    pub _slot_19: usize,
    /// [20] `GetLineIntersection(...)`.
    pub get_line_intersection: usize,
    /// [21] `HideAreaAction(bool)`.
    pub hide_area_action: usize,
    /// [22] `RecvMessage(C_EntityMessage*)`. Главный роутер сообщений.
    pub recv_message: usize,
    /// [23] `InvalidateRelation(C_Entity*)`. Удаляет подписки на сообщения.
    pub invalidate_relation: usize,
    /// [24] `GetCsInterface() const`. Может вернуть NULL.
    pub get_cs_interface: usize,
    /// [25] `GetCreateCsInterface()`. Lazy-init.
    pub get_create_cs_interface: usize,
    /// [26–30] Заглушки. [31] `Update(float)`.
    pub _slots_26_31: [usize; 6],

    // ====================================================================
    //  Пространственный интерфейс (32–39)
    // ====================================================================
    /// [32] `SetPos(C_Vector const&)`.
    pub set_pos: unsafe extern "C" fn(this: *mut c_void, pos: *const Vec3),
    /// [33] `SetDir(C_Vector const&)`. Направление через frame.
    pub set_dir: usize,
    /// [34] `SetRot(C_Quat const&)`. Кватернион `[x,y,z,w]`.
    pub set_rot: unsafe extern "C" fn(this: *mut c_void, quat: *const [f32; 4]),
    /// [35] `SetScale(float)`.
    pub set_scale: unsafe extern "C" fn(this: *mut c_void, scale: f32),
    /// [36] `GetPos() const`.
    pub get_pos: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,
    /// [37] `GetDir() const`.
    pub get_dir: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,
    /// [38] `GetRot() const`.
    pub get_rot: unsafe extern "C" fn(this: *const c_void, out: *mut [f32; 4]) -> *mut [f32; 4],
    /// [39] `GetScale() const`. `sqrt(m00² + m11² + m22²)` диагонали матрицы.
    pub get_scale: unsafe extern "C" fn(this: *const c_void) -> f32,

    // ====================================================================
    //  PRS / Frame / Owner / Детекция (40–49)
    // ====================================================================
    /// [40] Заглушка (`return 0`).
    pub _slot_40: usize,
    /// [41] `GameSavePRS(C_BitStream*)`. Позиция + вращение + масштаб.
    pub game_save_prs: usize,
    /// [42] `GameLoadPRS(C_BitStream*)`. Вызывает SetPos + SetRot.
    pub game_load_prs: usize,
    /// [43] `GetCameraPoint()`. `GetPos() + (0, 0, 2.0)`.
    pub get_camera_point: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,
    /// [44] `SetFrame(C_Frame*)`. Замена frame-узла (+0x78).
    pub set_frame: usize,
    /// [45] `SetOwner(C_Entity*)`. Записывает vehicle -> +0x80.
    pub set_owner: usize,
    /// [46] Заглушка (`return 0`).
    pub _slot_46: usize,
    /// [47] `IsDead()`. `return *(u8*)(this + 0x161)`.
    pub is_dead: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [48] `EnableUnderwaterDetection(bool)`. Управляет +0x310.
    pub enable_underwater_detection: usize,
    /// [49] `GetUnderwaterStatus() const`.
    pub get_underwater_status: usize,

    // ====================================================================
    //  Тик обновления (50–54)
    // ====================================================================
    /// [50] `TickPrePhysics(float)`. Главный тик гуманоида.
    pub tick_pre_physics: usize,
    /// [51] `TickPostPhysics(float)`.
    pub tick_post_physics: usize,
    /// [52] Геттер компонента: `return *(qword*)(this + 0x2A8)`.
    pub get_component_2a8: usize,
    /// [53] Геттер компонента: `return *(qword*)(this + 0x2B0)`.
    pub get_component_2b0: usize,
    /// [54] Геттер компонента: `return *(qword*)(this + 0x2B8)`.
    pub get_component_2b8: usize,

    // ====================================================================
    //  Управление моделями (55–64)
    // ====================================================================
    /// [55] `ChangeModelComplete(C_Frame*, bool)`.
    pub change_model_complete: usize,
    /// [56] `ChangeModel(C_Frame*, int, bool, bool)`.
    pub change_model: usize,
    /// [57] `SetBurnedModel(C_Frame*)`.
    pub set_burned_model: usize,
    /// [58] `RecoverFromBurnedModel()`.
    pub recover_from_burned_model: usize,
    /// [59] `ForceUpdateVisual(bool)`.
    pub force_update_visual: usize,
    /// [60] `GetModelNameByShopID(int) const`.
    pub get_model_name_by_shop_id: usize,
    /// [61] `GetCurrentModelShopID() const`.
    pub get_current_model_shop_id: usize,
    /// [62] `EnableAnimFPSLod(bool)`.
    pub enable_anim_fps_lod: usize,
    /// [63] `AnimFPSLodEnabled() const`.
    pub anim_fps_lod_enabled: usize,
    /// [64] `GetPosRequest(E_PosRequestType)`. locomotion->vfunc[14].
    pub get_pos_request: usize,

    // ====================================================================
    //  Запросы позиции и физики (65–68)
    // ====================================================================
    /// [65] `GetDirRequest(E_DirRequestType)`.
    pub get_dir_request: usize,
    /// [66] `GetTMRequest(E_TMRequestType, C_Matrix&)`.
    pub get_tm_request: usize,
    /// [67] `QueryPhysicsBodyZone`. switch(0-6) -> locomotion vfunc[46].
    pub query_physics_body_zone: usize,
    /// [68] `GetVelocity() const`. locomotion: provider+0x230.
    pub get_velocity: unsafe extern "C" fn(this: *const c_void, out: *mut Vec3) -> *mut Vec3,

    // ====================================================================
    //  Геймплей (69–82)
    // ====================================================================
    /// [69] `IsInCover() const`. `*(u32*)(this+0x430) == 4`.
    pub is_in_cover: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [70] `IsFighting() const`.
    pub is_fighting: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [71] `IsDraggingBodyV()`.
    pub is_dragging_body: usize,
    /// [72] `GetRigidBody() const`.
    pub get_rigid_body: usize,
    /// [73] `Spawn()`. Raycast вниз, привязка к земле.
    pub spawn: usize,
    /// [74] `return *(float*)(this + 0x158)`.
    pub get_damage_param: usize,
    /// [75] `SetTransparency(float)`. Пишет +0x294 и +0x298.
    pub set_transparency: unsafe extern "C" fn(this: *mut c_void, value: f32),
    /// [76] `GetTransparency() const`. `return *(float*)(this + 0x298)`.
    pub get_transparency: unsafe extern "C" fn(this: *const c_void) -> f32,
    /// [77] `SetTransparencyTarget(float)`. Пишет только +0x294.
    pub set_transparency_target: unsafe extern "C" fn(this: *mut c_void, value: f32),
    /// [78] `SetDirtBlend(float)`.
    pub set_dirt_blend: usize,
    /// [79] `CustomRequest(E_CustomRequest)`.
    pub custom_request: usize,
    /// [80] `SetColor(uint)`.
    pub set_color: usize,
    /// [81] `GetColor()`.
    pub get_color: usize,
    /// [82] `DoDamage(C_EntityMessageDamage*)`.
    pub do_damage: unsafe extern "C" fn(this: *mut c_void, msg: *const c_void) -> u8,

    // ====================================================================
    //  Управление игроком (83–109)
    // ====================================================================
    /// [83] `AreControlsLocked() const`. `*(u32*)(this+0x430) == 10`.
    pub are_controls_locked: usize,
    /// [84] `IsProcessedByRender() const`.
    pub is_processed_by_render: usize,
    /// [85] `GetUnderwaterDetectionEnabled() const`.
    pub get_underwater_detection_enabled: usize,
    /// [86] `InitColours()`.
    pub init_colours: usize,
    /// [87] `GetDuplicatedMaterialGUID(C_GUID const&)`.
    pub get_duplicated_material_guid: usize,
    /// [88] `LinkObject(uint64 const&, C_Frame*)`.
    pub link_object: usize,
    /// [89] `UnlinkObject(C_Frame*)`.
    pub unlink_object: usize,
    /// [90] `LockControls(bool)`.
    pub lock_controls: usize,
    /// [91] `IsControlLockFinished() const`.
    pub is_control_lock_finished: usize,
    /// [92] `SetPlayerCtrlStyle(char const*)`. Хеширует строку -> +0x438.
    pub set_player_ctrl_style: usize,
    /// [93] `GetPlayerCtrlStyle() const`. `return *(qword*)(this+0x520)`.
    pub get_player_ctrl_style: usize,
    /// [94] `IsPlayerMovement() const`. `low_byte(+0x3D8) != 3 && != 4`.
    pub is_player_movement: usize,
    /// [95] `CanPlayerTakeCover() const`.
    pub can_player_take_cover: usize,
    /// [96] `SetFightAbility(bool, int)`. Биты [1..3] в +0x490.
    pub set_fight_ability: usize,
    /// [97] `SetFightControlStyle(int)`. Биты [4..6].
    pub set_fight_control_style: usize,
    /// [98] `SetFightHint(bool, int)`. Биты [7..13].
    pub set_fight_hint: usize,
    /// [99] `SetFightGrabTimeScale(bool)`. Бит 14.
    pub set_fight_grab_time_scale: usize,
    /// [100] `SetForcedDropWeapon(bool)`. Бит 15.
    pub set_forced_drop_weapon: usize,
    /// [101] `GetGuiController() const`. `return *(qword*)(this+0x428)`.
    pub get_gui_controller: usize,
    /// [102] `IsCrouch() const`. `sub45c.state == 1`.
    pub is_crouch: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [103] `IsCrouchOrDrag() const`. `sub45c.state ∈ {1, 2, 3}`.
    pub is_crouch_or_drag: unsafe extern "C" fn(this: *const c_void) -> bool,
    /// [104] `ProcessNotification(short, C_GUID const&)`.
    pub process_notification: usize,
    /// [105] `UpdateAnimSetFromWeapon(int)`.
    pub update_anim_set_from_weapon: usize,
    /// [106] `ProcessNotificationCamera(short)`.
    pub process_notification_camera: usize,
    /// [107] `ProcessNotificationFFX(short)`. Формирует `PlayerFx%u`.
    pub process_notification_ffx: usize,
    /// [108] `CoatEnabled()`.
    pub coat_enabled: usize,
    /// [109] `GetRigidBody()` — Player override (non-const).
    pub get_rigid_body_player: usize,
}

// Проверки на этапе компиляции

const _: () = {
    assert!(std::mem::size_of::<CHumanVTable>() == 110 * 8);

    assert!(std::mem::offset_of!(CHumanVTable, set_pos) == 32 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, set_dir) == 33 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, set_rot) == 34 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_scale) == 39 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_camera_point) == 43 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, is_dead) == 47 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, tick_pre_physics) == 50 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_pos_request) == 64 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_dir_request) == 65 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_tm_request) == 66 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, query_physics_body_zone) == 67 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_velocity) == 68 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, do_damage) == 82 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, is_crouch) == 102 * 8);
    assert!(std::mem::offset_of!(CHumanVTable, get_rigid_body_player) == 109 * 8);
};
