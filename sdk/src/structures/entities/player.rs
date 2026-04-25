//! Структуры гуманоидов: `I_Human2`, `C_Human2`, `C_Player2`.
//!
//! ## Иерархия наследования
//!
//! ```text
//! C_Entity (0x78)
//!   └─ C_Actor (0xA8)
//!       └─ I_Human2 — abstract base (alloc 0xA58 компонентный блок)
//!           └─ C_Human2 — NPC (FactoryType 0x0E)
//!               └─ C_Player2 — игрок (FactoryType 0x10)
//! ```
//!
//! ## Карта классов
//!
//! | Класс движка | Rust struct | Конструктор | VTable |
//! |:-------------|:-----------|:------------|:-------|
//! | `I_Human2`   | `CHuman`    | `0x140D730B0` | `0x1418E2BD8` (abstract) |
//! | `C_Human2`   | `CHumanNPC` | `0x140D712E0` | `0x1418E5188` |
//! | `C_Player2`  | `CPlayer`   | `0x1400B9160` | `0x14184C060` |
//!
//! ## Компонентный блок гуманоида (alloc 0xA58 байт)
//!
//! Базовый `I_Human2::I_Human2` аллоцирует один heap-блок и раскладывает
//! в нём все компоненты гуманоида. Указатели в самом гуманоиде ссылаются
//! на смещения внутри этого блока:
//!
//! - `C_HumanInventory` — инвентарь (back-ref на human)
//! - `C_HumanAIController` — ИИ-поведение
//! - `C_HumanWeaponController` — управление оружием
//! - `C_HumanHeadController` — управление головой/прицеливанием взгляда
//! - `mafia::C_FrameColors` — цвета и материалы модели
//! - 5× `C_PlayerEmitter` (с разными type-параметрами 0/1/3/5)
//! - `ai::sight::C_HumanDetector` + `C_ActorEmitter`/`C_ActorEar` — система sight/hear AI
//!
//! Активация/деактивация эмиттеров и детектора управляется через
//! `C_Human2::OnActivate`/`OnDeactivate` и `I_Human2::ResetSenses`.

use super::{
    CFrameColors, CHumanAIController, CHumanHeadController, CHumanInventory,
    CHumanWeaponController, CPlayerEmitter,
};
use super::entity::CActor;
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

// =============================================================================
//  CHuman — базовый класс гуманоидов
// =============================================================================

/// Базовый класс всех гуманоидов (I_Human2 в движке).
///
/// Содержит компоненты ИИ, инвентарь, здоровье, модель, физику.
///
/// ## Компонентный блок (+0xA8..+0x120)
///
/// Все указатели ведут в **один аллоцированный блок** размером 0xA58 (2648 байт).
///
/// | Смещение | Компонент | Описание |
/// |:---------|:----------|:---------|
/// | +0x0A8 | Визуальный компонент | Поздняя инициализация |
/// | +0x0B0 | C_ViewCone | Конус обзора ИИ |
/// | +0x0B8 | C_FrameColors | Цвета/материалы модели |
/// | +0x0C0 | Emitter (тип 0) | Эмиттер ИИ, back-ref |
/// | +0x0C8 | Emitter (тип 3) | Синхронизация трансформов |
/// | +0x0D0 | Контроллер сущности | Управление entity |
/// | +0x0E8 | C_HumanInventory | Инвентарь, back-ref на human |
/// | +0x0F0 | Хранилище свойств | Property store |
/// | +0x0F8 | C_HumanAIController | ИИ-контроллер поведения |
/// | +0x100 | Физическая группа | Минимальная структура (3 dword) |
/// | +0x108 | C_HumanWeaponController | Управление оружием |
/// | +0x110 | C_HumanHeadController | Управление головой |
/// | +0x118 | Emitter (тип 5) | Эмиттер ИИ (доп.) |
/// | +0x120 | Emitter (тип 1) | Эмиттер ИИ (back-ref) |
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHuman {
    pub actor: CActor, // +0x000..+0x0A7

    // ====================================================================
    //  Компонентные указатели (alloc-блок 0xA58 байт)
    // ====================================================================
    //
    // Все указатели ниже ссылаются ВНУТРЬ единого heap-блока, который
    // `I_Human2::I_Human2` аллоцирует как `MemAlloc(0xA58)`.

    /// Поздний визуальный компонент.
    /// В базовом ctor оставляется NULL — устанавливается в `GameInit`.
    pub visual_component: *mut c_void, // +0x0A8
    /// `game::ai::sight::C_ViewCone` — конус обзора AI.
    /// Поле на этом смещении заполняется в `C_Human2::C_Human2` или `GameInit`.
    pub view_cone: *mut c_void, // +0x0B0
    /// `mafia::C_FrameColors` — цвета и материалы модели гуманоида.
    /// Создаётся в `I_Human2::I_Human2`.
    pub frame_colors: *mut CFrameColors, // +0x0B8
    /// `C_PlayerEmitter` (type=0) — emitter ИИ с back-ref на гуманоида.
    pub emitter_ai_0: *mut CPlayerEmitter, // +0x0C0
    /// `C_PlayerEmitter` (type=3) — синхронизация трансформов.
    /// VTable: `M2DE_vtbl_TransformSyncComponent` (`0x1418E33A8`).
    pub emitter_transform: *mut CPlayerEmitter, // +0x0C8
    /// Контроллер-эмиттер (back-ref на гуманоида).
    /// Структура: `[vtable][NULL][zero_dword][this back-ref]`.
    pub entity_controller: *mut c_void, // +0x0D0
    /// Опциональный компонент, активируется в `GameInit`. Часто NULL.
    /// VTable: `M2DE_vtbl_HumanOptionalComponent_D8` (`0x1418E3380`).
    pub optional_component: *mut CPlayerEmitter, // +0x0D8
    /// Резервное компонентное поле. В базовом ctor не инициализируется.
    pub component_e0: *mut c_void, // +0x0E0
    /// `C_HumanInventory` — инвентарь гуманоида (back-ref на гуманоида внутри).
    /// VTable: `M2DE_vtbl_HumanInventory` (`0x1418E3090`). Размер 376 байт.
    pub inventory: *mut CHumanInventory, // +0x0E8
    /// Контейнер AI-controller'а (`property-store`).
    /// `C_HumanAIController` лежит как inline-объект внутри этого контейнера.
    pub property_store: *mut c_void, // +0x0F0
    /// `C_HumanAIController` — ИИ-контроллер поведения NPC.
    /// Создаётся как `M2DE_CHumanAIController_Constructor(this, entity)` (`0x140D71AD0`).
    /// VTable: `M2DE_vtbl_HumanBehaviorComponent` (`0x1418E37A8`).
    pub ai_controller: *mut CHumanAIController, // +0x0F8
    /// Физическая группа (минимальная struct: 3 dword nul'а).
    pub physics_group: *mut c_void, // +0x100
    /// `C_HumanWeaponController` — управление оружием.
    pub weapon_controller: *mut CHumanWeaponController, // +0x108
    /// `C_HumanHeadController` — управление поворотом головы и прицеливанием взгляда.
    pub head_controller: *mut CHumanHeadController, // +0x110
    /// `C_PlayerEmitter` (type=5) — дополнительный AI emitter.
    pub emitter_ai_5: *mut CPlayerEmitter, // +0x118
    /// `C_PlayerEmitter` (type=1) — emitter ИИ (back-ref на гуманоида).
    pub emitter_ai_1: *mut CPlayerEmitter, // +0x120

    pub _unk_128: [u8; 0x20], // +0x128..+0x147

    // Здоровье и урон
    /// Текущее здоровье. Начальное значение: 210.0.
    pub current_health: f32, // +0x148
    /// Максимальное здоровье. Начальное значение: 210.0.
    pub max_health: f32, // +0x14C
    /// Множитель урона (1.0 по умолчанию).
    pub damage_distance_mult: f32, // +0x150
    /// Дистанция полного урона (5.0 по умолчанию).
    pub damage_full_dist: f32, // +0x154
    /// Параметр урона/физики. Vtable[74]: `return *(float*)(this+0x158)`.
    pub damage_param: f32, // +0x158
    /// Множитель входящего урона. Vtable[78] `SetDirtBlend` пишет сюда.
    pub damage_scale_factor: f32, // +0x15C
    /// Неуязвимость (0 = уязвим).
    pub invulnerability: u8, // +0x160
    /// Флаг смерти (0 = жив). Vtable[47]: `return *(u8*)(this+0x161)`.
    pub is_dead: u8, // +0x161
    /// Полубог: получает урон, но не умирает.
    pub demigod: u8, // +0x162
    pub _flag_163: u8,     // +0x163
    pub _pad_164: [u8; 4], // +0x164..+0x167

    // Модель и внешний вид
    /// Дескриптор модели/одежды.
    pub model_descriptor: *mut c_void, // +0x168
    /// Сохранённый frame при переключении моделей. Vtable[57].
    pub saved_frame_ptr: *mut c_void, // +0x170
    /// Флаг стриминга при переключении.
    pub saved_streaming_flag: u8, // +0x178
    pub _pad_179: [u8; 7], // +0x179..+0x17F

    // Зоны повреждений
    /// Таблица множителей урона по зонам тела.
    pub body_damage_multipliers: *mut f32, // +0x180
    /// Количество зон повреждений (обычно 12).
    pub body_zone_count: u32, // +0x188
    pub _unk_18c: u32, // +0x18C

    /// Слабая ссылка на себя (`self_ref == this`).
    /// Регистрируется через `ue::C_WeakPtrObj<C_Human2>::m_StaticTable`.
    /// Инициализируется в `C_Human2::C_Human2` (NPC-ctor), не в базовом `I_Human2`.
    pub self_ref: *mut CHuman, // +0x190
    pub _unk_198: u32,        // +0x198
    pub _unk_19c: u32,        // +0x19C
    pub _pad_1A0: [u8; 0x20], // +0x1A0..+0x1BF

    /// 8 слотов SmartPtr (по 16 байт).
    /// ID слотов: {1,2,3,4,5,6,7,-1(sentinel)}.
    pub smart_ptr_slots: [[u8; 16]; 8], // +0x1C0..+0x23F
    pub _pad_240: [u8; 0x18], // +0x240..+0x257

    /// Контроллер передвижения и анимации (locomotion controller).
    ///
    /// Управляет направлением, вращением, скоростью, анимациями.
    /// 68 vtable-слотов. Внутренний layout:
    /// - +0x08: back-ref на CHuman
    /// - +0x1E8: кватернион ориентации
    /// - +0x230: вектор скорости
    pub locomotion: *mut c_void, // +0x258

    pub _pad_260: [u8; 0x18], // +0x260..+0x277

    /// CS-интерфейс (lazy-init 32 байта). Vtable[24]/[25].
    pub cs_interface: *mut c_void, // +0x278
    pub _pad_280: [u8; 0x14], // +0x280..+0x293

    /// Целевая прозрачность. Vtable[75] пишет сюда и в current.
    pub transparency_target: f32, // +0x294
    /// Текущая прозрачность. Vtable[76]: `return *(float*)(this+0x298)`.
    pub transparency_current: f32, // +0x298
    pub _pad_29C: [u8; 0x0C], // +0x29C..+0x2A7

    /// Компонент (vtable[52]: `return *(qword*)(this+0x2A8)`).
    pub component_2a8: *mut c_void, // +0x2A8
    /// Компонент (vtable[53]: `return *(qword*)(this+0x2B0)`).
    pub component_2b0: *mut c_void, // +0x2B0
    /// Компонент (vtable[54]: `return *(qword*)(this+0x2B8)`).
    pub component_2b8: *mut c_void, // +0x2B8

    pub _pad_2C0: [u8; 0x38], // +0x2C0..+0x2F7

    /// Флаг активности оверлея модели. Vtable[62].
    pub model_overlay_active: u8, // +0x2F8
    pub _pad_2F9: [u8; 7], // +0x2F9..+0x2FF
    /// Состояние оверлея модели.
    pub model_overlay_state: usize, // +0x300
    /// Флаг отсоединения frame. Vtable[56].
    pub frame_detach_flag: u8, // +0x308
    pub _pad_309: [u8; 7], // +0x309..+0x30F

    /// Детектор водных коллизий.
    /// Создаётся `C_WaterCollisionsModule::CreateDetector()`.
    /// Vtable[48]/[49].
    pub water_detector: *mut c_void, // +0x310
}

// Размер CHuman = 0x318 байт.
//
// Подтверждение: фабрика `0x140D90440` (создаёт C_HumanNPC) аллоцирует
// `M2DE_GlobalAlloc(792)` = 0x318. Так как C_HumanNPC = I_Human2 + 0
// дополнительных полей (NPC-ctor пишет ВСЕ свои данные внутрь зоны
// CHuman, до +0x310 включительно), то CHuman имеет точно такой же
// размер, как и реальный C_HumanNPC.
const _: () = {
    assert!(std::mem::size_of::<CHuman>() == 0x318);
};

assert_field_offsets!(CHuman {
    actor                == 0x000,
    visual_component     == 0x0A8,
    view_cone            == 0x0B0,
    frame_colors         == 0x0B8,
    emitter_ai_0         == 0x0C0,
    emitter_transform    == 0x0C8,
    entity_controller    == 0x0D0,
    optional_component   == 0x0D8,
    component_e0         == 0x0E0,
    inventory            == 0x0E8,
    property_store       == 0x0F0,
    ai_controller        == 0x0F8,
    physics_group        == 0x100,
    weapon_controller    == 0x108,
    head_controller      == 0x110,
    emitter_ai_5         == 0x118,
    emitter_ai_1         == 0x120,
    current_health       == 0x148,
    max_health           == 0x14C,
    damage_distance_mult == 0x150,
    damage_full_dist     == 0x154,
    damage_param         == 0x158,
    damage_scale_factor  == 0x15C,
    invulnerability      == 0x160,
    is_dead              == 0x161,
    demigod              == 0x162,
    model_descriptor     == 0x168,
    saved_frame_ptr      == 0x170,
    body_damage_multipliers == 0x180,
    body_zone_count      == 0x188,
    self_ref             == 0x190,
    locomotion           == 0x258,
    cs_interface         == 0x278,
    transparency_target  == 0x294,
    transparency_current == 0x298,
    component_2a8        == 0x2A8,
    component_2b0        == 0x2B0,
    component_2b8        == 0x2B8,
    model_overlay_active == 0x2F8,
    model_overlay_state  == 0x300,
    frame_detach_flag    == 0x308,
    water_detector       == 0x310,
});

const _: () = {
    assert!(std::mem::offset_of!(CHuman, actor.base.vtable) == 0x00);
    assert!(std::mem::offset_of!(CHuman, actor.base.table_id) == 0x24);
    assert!(std::mem::offset_of!(CHuman, actor.frame_node) == 0x78);
    assert!(std::mem::offset_of!(CHuman, actor.owner) == 0x80);
    assert!(std::mem::offset_of!(CHuman, actor.entity_subtype) == 0xA0);
};

impl CHuman {
    pub fn factory_type(&self) -> u8 {
        self.actor.base.factory_type()
    }
    pub fn table_id(&self) -> u32 {
        self.actor.base.table_id
    }
    pub fn entity_flags(&self) -> u32 {
        self.actor.base.entity_flags
    }

    pub fn health(&self) -> f32 {
        self.current_health
    }
    pub fn is_alive(&self) -> bool {
        self.is_dead == 0
    }
    pub fn is_invulnerable(&self) -> bool {
        self.invulnerability != 0
    }
    pub fn is_demigod(&self) -> bool {
        self.demigod != 0
    }
    pub fn is_in_vehicle(&self) -> bool {
        !self.actor.owner.is_null()
    }
    pub fn is_player(&self) -> bool {
        self.factory_type() == 0x10
    }

    /// Проверяет инвариант: `self_ref` указывает на `self`.
    pub fn has_valid_self_ref(&self) -> bool {
        self.self_ref == self as *const Self as *mut CHuman
    }
}

// =============================================================================
//  CHumanNPC — NPC (FactoryType 0x0E)
// =============================================================================

/// NPC-гуманоид (`C_Human2`). FactoryType = `0x0E`.
///
/// **Размер: 0x318 байт** (фабрика `0x140D90440` аллоцирует 792 байта).
///
/// На уровне C++ ABI `C_Human2` **не добавляет собственных полей** сверх
/// `I_Human2` — NPC-конструктор записывает все свои данные внутрь зоны
/// базового `CHuman` (от +0x190 self_ref до +0x310 water_detector).
/// Поэтому в Rust `CHumanNPC` структурно эквивалентна `CHuman`.
///
/// Конструктор `C_Human2::C_Human2`:
/// 1. Вызывает `I_Human2::I_Human2` (заполняет от +0xA8 до +0x180,
///    аллоцирует компонентный блок 0xA58 байт)
/// 2. Устанавливает vtable `C_Human2`, `SetTypeID(0x0E)`
/// 3. `entity_flags |= 0x40` (NPC marker bit 6)
/// 4. Инициализирует 8× SmartPtr на +0x1C0..+0x23F (ID 1..7, sentinel -1)
/// 5. 9-й SmartPtr на +0x270
/// 6. Множество одиночных полей в зоне +0x2A8..+0x310
/// 7. Регистрирует в глобальной EntityTable
/// 8. Устанавливает `self_ref = this` (на +0x190) через
///    `ue::C_WeakPtrObj<C_Human2>::m_StaticTable`
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHumanNPC {
    pub base: CHuman,
    /// Pre-CPlayer alignment zone (32 байта).
    ///
    /// **Не инициализируется** ни NPC-, ни Player-конструктором.
    /// Существует исключительно для корректного выравнивания CPlayer:
    /// `CPlayer.car_wrapper` лежит на +0x338, что = `CHumanNPC` (0x318) + 0x20.
    /// Если `C_HumanNPC` создан сам по себе (без наследника CPlayer), эти
    /// 32 байта — мусор за пределами аллокации (size 0x318).
    pub _player_alignment_pad: [u8; 0x20],
}

const _: () = {
    // Виртуальный размер struct в Rust = 0x338 (для совместимости с CPlayer).
    // Реальный размер C_HumanNPC объекта в памяти = 0x318 (см. фабрику).
    assert!(std::mem::size_of::<CHumanNPC>() == 0x338);
    assert!(std::mem::offset_of!(CHumanNPC, base) == 0x000);
    assert!(std::mem::offset_of!(CHumanNPC, _player_alignment_pad) == 0x318);
};

impl CHumanNPC {
    pub fn as_human(&self) -> &CHuman {
        &self.base
    }
    pub fn as_human_mut(&mut self) -> &mut CHuman {
        &mut self.base
    }
}

// =============================================================================
//  CPlayerSub45C — стейт-машина приседания
// =============================================================================

/// Подсистема состояний приседания и перетаскивания тел.
///
/// | state | Значение |
/// |------:|:---------|
/// | 0 | Покой |
/// | 1 | Ожидание (Vtable[102] `IsCrouch` = true) |
/// | 2 | Активно A (Vtable[103] `IsCrouchOrDrag` = true) |
/// | 3 | Активно B (Vtable[103] = true) |
/// | 4 | Отложенное (deferred) |
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPlayerSub45C {
    /// Текущий код действия.
    pub current_code: u32, // +0x00
    /// Вспомогательный код.
    pub aux_code: u32, // +0x04
    /// Текущее состояние (0..4).
    pub state: u32, // +0x08
}

// =============================================================================
//  CarWrapper — обёртка транспорта (0xA8 байт inline)
// =============================================================================

/// Обёртка взаимодействия гуманоида с транспортом.
///
/// Inline-объект размером 0xA8 байт внутри CPlayer.
/// `player+0x3D8` = `data[0xA0]` — начальное состояние (3).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CarWrapper {
    pub data: [u8; 0xA8],
}

// =============================================================================
//  CPlayer — игрок (FactoryType 0x10, 0x528 байт)
// =============================================================================

/// Структура игрока (C_Player2 в движке). FactoryType = 0x10.
///
/// **Полный размер: 0x528 байт**.
///
/// ## Цепочка конструкторов
///
/// ```text
/// CPlayer_Constructor (0x1400B9160)
///   -> CHumanNPC_Constructor
///     -> CHuman_BaseConstructor
///       -> ActorEntity_Construct -> BaseEntity_Construct
///   -> VTable CPlayer
///   -> CarWrapper_Init(+0x338)
///   -> SmartPtr × 4 (+0x3E0..+0x3FF)
///   -> S_HumanCommandMoveDir (alloc 0x70) -> +0x408
///   -> S_HumanCommandStand (alloc 0x30) -> +0x410
///   -> Sub45C_Init(+0x45C)
///   -> Component468_Init(+0x468)
///   -> GuiController (alloc 0x30) -> +0x428
///   -> PlayerEmitter (alloc 0x38) -> +0x518
///   -> SetType(0x10)
///   -> WaterCollisions::CreateDetector -> +0x310
/// ```
///
/// ## Встроенные объекты
///
/// | Смещение | Размер | Описание |
/// |:---------|:-------|:---------|
/// | +0x338 | 0xA8 | CarWrapper (inline) |
/// | +0x45C | 0x0C | CPlayerSub45C (inline) |
/// | +0x468 | 0x48 | Компонент действий (inline) |
/// | +0x4B8 | 0x30 | Компонент состояний (inline) |
#[repr(C)]
#[allow(non_snake_case)]
pub struct CPlayer {
    pub base: CHuman, // +0x000..+0x317

    /// NPC-часть (заполняется NPC-конструктором).
    pub _npc_gap: [u8; 0x20], // +0x318..+0x337

    // Транспорт
    /// Обёртка взаимодействия с транспортом (inline).
    pub car_wrapper: CarWrapper, // +0x338..+0x3DF

    // Команды управления
    /// 4 слота SmartPtr общих команд.
    pub cmd_slots_a: [usize; 4], // +0x3E0..+0x3FF
    /// SmartPtr команды (тип B).
    pub cmd_slot_b: usize, // +0x400
    /// -> S_HumanCommandMoveDir (alloc 0x70): направление движения.
    /// в **`M2DE_CPlayer2_ApplyApproachMoveCommand`** использует **`this+0x408`** (`1032`).
    pub cmd_move_dir: usize, // +0x408
    /// -> S_HumanCommandStand (alloc 0x30): стойка.
    pub cmd_stand: usize, // +0x410
    /// SmartPtr (тип E).
    pub cmd_slot_e: usize, // +0x418
    /// SmartPtr (тип F).
    pub cmd_slot_f: usize, // +0x420

    // Основные состояния
    /// GUI-контроллер (alloc 0x30, back-ref на player).
    /// Vtable[101]: `return *(qword*)(this+0x428)`.
    pub gui_controller: *mut c_void, // +0x428

    /// Основной код состояния игрока.
    /// - Vtable[69] `IsInCover`: `== 4`
    /// - Vtable[83] `AreControlsLocked`: `== 10`
    pub state_code: u32, // +0x430

    pub _internal_434: [u8; 4], // +0x434..+0x437

    /// Маска стиля управления.
    /// Vtable[92] `SetPlayerCtrlStyle` хеширует строку -> записывает сюда.
    pub ctrl_style_mask: u32, // +0x438

    pub _pad_43C: [u8; 8], // +0x43C..+0x443

    pub field_444: u32,    // +0x444
    pub _pad_448: [u8; 4], // +0x448..+0x44B
    pub flag_44C: u8,      // +0x44C
    pub _pad_44D: [u8; 3], // +0x44D..+0x44F
    pub field_450: u64,    // +0x450
    pub _pad_458: [u8; 4], // +0x458..+0x45B

    // Стейт-машина приседания
    /// Подсистема приседания и перетаскивания тел (inline).
    pub sub45c: CPlayerSub45C, // +0x45C

    /// Компонент действий (inline, 0x48 байт).
    pub _component_468: [u8; 0x28], // +0x468..+0x48F

    // Боевая система
    /// Битовое поле боевых способностей.
    ///
    /// | Биты | Vtable | Описание |
    /// |:----:|:------:|:---------|
    /// | 1..3 | [96] | SetFightAbility |
    /// | 4..6 | [97] | SetFightControlStyle |
    /// | 7..13 | [98] | SetFightHint |
    /// | 14 | [99] | SetFightGrabTimeScale |
    /// | 15 | [100] | SetForcedDropWeapon |
    pub fight_flags: u32, // +0x490

    pub _pad_494: [u8; 0x1C], // +0x494..+0x4AF

    pub smartptr_4b0: usize, // +0x4B0

    /// Компонент состояний (inline, 0x30 байт).
    pub _component_4b8: [u8; 0x30], // +0x4B8..+0x4E7

    pub smartptr_4e8: usize,  // +0x4E8
    pub _pad_4F0: u8,         // +0x4F0
    pub flag_4f1: u8,         // +0x4F1
    pub _pad_4F2: [u8; 0x12], // +0x4F2..+0x503

    pub field_504: u32, // +0x504
    /// Указатель на текущий обработчик действия.
    pub action_handler: *mut c_void, // +0x508

    /// Флаги состояния. Биты 6-7 очищаются в конструкторе.
    pub state_flags_510: u32, // +0x510
    pub _pad_514: [u8; 4], // +0x514..+0x517

    /// Эмиттер видимости (PlayerEmitter, alloc 0x38).
    /// Vtable[107] форматирует `PlayerFx%u` через этот компонент.
    pub player_emitter: *mut c_void, // +0x518

    /// Строка стиля управления (`ue::sys::utils::C_String`).
    /// Vtable[92] пишет, [93] читает.
    pub ctrl_style_string: usize, // +0x520
}

const _: () = {
    assert!(std::mem::size_of::<CPlayer>() == 0x528);
};

assert_field_offsets!(CPlayer {
    base              == 0x000,
    _npc_gap          == 0x318,
    car_wrapper       == 0x338,
    cmd_slots_a       == 0x3E0,
    cmd_move_dir      == 0x408,
    cmd_stand         == 0x410,
    cmd_slot_e        == 0x418,
    cmd_slot_f        == 0x420,
    gui_controller    == 0x428,
    state_code        == 0x430,
    ctrl_style_mask   == 0x438,
    field_444         == 0x444,
    flag_44C          == 0x44C,
    field_450         == 0x450,
    sub45c            == 0x45C,
    fight_flags       == 0x490,
    smartptr_4b0      == 0x4B0,
    smartptr_4e8      == 0x4E8,
    field_504         == 0x504,
    action_handler    == 0x508,
    state_flags_510   == 0x510,
    player_emitter    == 0x518,
    ctrl_style_string == 0x520,
});

const _: () = {
    assert!(std::mem::offset_of!(CPlayer, base.actor.base.table_id) == 0x24);
    assert!(std::mem::offset_of!(CPlayer, base.actor.frame_node) == 0x78);
    assert!(std::mem::offset_of!(CPlayer, base.actor.owner) == 0x80);
    assert!(std::mem::offset_of!(CPlayer, base.current_health) == 0x148);
    assert!(std::mem::offset_of!(CPlayer, base.is_dead) == 0x161);
    assert!(std::mem::offset_of!(CPlayer, base.locomotion) == 0x258);
    assert!(std::mem::offset_of!(CPlayer, base.water_detector) == 0x310);
};

impl CPlayer {
    pub fn as_human(&self) -> &CHuman {
        &self.base
    }
    pub fn as_human_mut(&mut self) -> &mut CHuman {
        &mut self.base
    }
    pub fn is_player(&self) -> bool {
        self.base.factory_type() == 0x10
    }

    /// Состояние CarWrapper (`player+0x3D8`).
    /// Vtable[94] `IsPlayerMovement`: `!= 3 && != 4`.
    pub fn car_wrapper_state(&self) -> u8 {
        self.car_wrapper.data[0xA0]
    }
}
