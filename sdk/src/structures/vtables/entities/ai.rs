//! Типизированные VTable для AI-подсистемы гуманоидов.
//!
//! ## Адреса в .rdata
//!
//! | Vtable | RVA | Описание |
//! |:-------|:----|:---------|
//! | `C_AIController` base | `0x14185B238` | 11 слотов, базовый AI-контроллер |
//! | `C_HumanAIController` | `0x1418E37A8` | 15 слотов, расширение для humanoid'ов |
//! | `C_HumanAIState` | `0x1418E34E0` | 3 слота, inline-объект внутри controller'а на +0x70 |
//! | `C_HumanAIResources` | `0x1418E3760` | 10 слотов, ресурс-контейнер (alloc 3792 байт) |
//! | `C_MafiaNavAgent` (candidate) | `0x1418F43D8` | 8 слотов, нав-агент (требует доп верификации) |
//! | `C_AITask` | TBD | 5 слотов, задача в global scheduler'е |
//! | secondary MI vtable | `0x1418E3290` | adjustor thunk для C_HumanAIController MI |
//!
//! ## Иерархия наследования
//!
//! ```text
//! game::ai::C_AIController (11 slots)
//!   └─ C_HumanAIController (15 slots) — override [0,1,5,6,7,8,9,10] + new [11..14]
//!
//! C_HumanAIState (3 slots) — inline в C_HumanAIController на +0x70
//! C_HumanAIResources (10 slots) — heap-allocated, в C_HumanAIController на +0x08
//! C_MafiaNavAgent (8 slots) — heap-allocated, в C_HumanAIController на +0x210
//! C_AITask (5 slots) — отдельная задача в global AI scheduler'е
//! ```
//!
//! ## Зоны слотов C_AIController primary vtable (11 слотов)
//!
//! | Слоты | Назначение |
//! |:------|:-----------|
//! | 0–1   | Жизненный цикл (dtor + deleting dtor) |
//! | 2–3   | Request management: `AcceptRequest`, `TerminateRequest` |
//! | 4     | `GetMessagesAccessor()` — accessor очереди сообщений |
//! | 5     | **`Inactivate()`** — деактивация AI (cleanup chain) |
//! | 6     | **`Activate()`** — активация AI |
//! | 7–8   | Сериализация: `GameSave`, `GameLoad` |
//! | 9–10  | Tick: `AIPreUpdate`, `AIPostUpdate` |
//!
//! ## Зоны слотов C_HumanAIController vtable (15 слотов)
//!
//! | Слоты | Назначение |
//! |:------|:-----------|
//! | 0–10  | Override базовых слотов C_AIController (см. выше) |
//! | 11    | `ProcessMessage(C_EntityMessage*)` — диспатч сообщения в AI |
//! | 12    | Reserved (semantic TBD) |
//! | 13–14 | `GameSaveDependencies`, `GameLoadDependencies` |
//!
//! ## Slot order для slot 5/6 (Inactivate/Activate)
//!
//! Подтверждено по живым vtable:
//! - `M2DE_CAIController_Inactivate` (`0x1401DED30`) — slot 5 в
//!   `M2DE_vtbl_CAIController` (`0x14185B238`).
//! - `M2DE_CHumanAIController_Inactivate` (`0x140DAC750`) — slot 5 в
//!   `M2DE_vtbl_HumanBehaviorComponent` (`0x1418E37A8`).

use std::ffi::c_void;

// =============================================================================
//  game::ai::C_AIController (11 slots) — базовый AI-контроллер
// =============================================================================

/// VTable базового `game::ai::C_AIController`. **11 слотов в DE.**
#[repr(C)]
pub struct CAIControllerVTable {
    /// `[0]` `~C_AIController()` — combined destructor.
    pub dtor: usize,
    /// `[1]` `AcceptRequest(C_Request*)` — принять request.
    pub accept_request: usize,
    /// `[2]` `TerminateRequest(C_Request*)` — завершить request.
    pub terminate_request: usize,
    /// `[3]` `GetMessagesAccessor()` — accessor очереди сообщений.
    pub get_messages_accessor: usize,
    /// `[4]` `Activate()` — активация AI.
    pub activate: usize,
    /// `[5]` `Inactivate()` — деактивация AI (cleanup chain).
    ///
    /// `M2DE_CAIController_Inactivate` (`0x1401DED30`).
    pub inactivate: usize,
    /// `[6]` `GameSave(C_BitStream*)`.
    pub game_save: usize,
    /// `[7]` `GameLoad(C_BitStream*)`.
    pub game_load: usize,
    /// `[8]` `AIPreUpdate()` — pre-tick.
    pub ai_pre_update: usize,
    /// `[9]` `AIPostUpdate()` — post-tick.
    pub ai_post_update: usize,
    /// `[10]` Reserved.
    pub _slot_10: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CAIControllerVTable>() == 11 * 8);
    assert!(std::mem::offset_of!(CAIControllerVTable, activate) == 4 * 8);
    assert!(std::mem::offset_of!(CAIControllerVTable, inactivate) == 5 * 8);
};

// =============================================================================
//  C_HumanAIController (15 slots) — расширение для гуманоидов
// =============================================================================

type FnInactivate = unsafe extern "system" fn(this: *mut c_void);

/// VTable `C_HumanAIController` (extends `C_AIController`). 14 слотов.
///
/// Слоты 0-5 переопределены human-specific; 6/7 — bool getter'ы на
/// inline `C_HumanAIState` (`+0x70`); 8/9 — `AIPreUpdate`/`AIPostUpdate`;
/// 10 — `ProcessMessage` (большой switch на ID сообщения); 11 — CRT
/// adjustor thunk; 12/13 — thunks к базовым `CAIController::GameSave/Load`.
#[repr(C)]
pub struct CHumanAIControllerVTable {
    /// `[0]` `~C_HumanAIController()` — combined destructor.
    pub dtor: usize,
    /// `[1]` `AcceptRequest(C_Request*)` — inherited from `C_AIController`.
    pub accept_request: usize,
    /// `[2]` `TerminateRequest(C_Request*)` — inherited (AVL tree remove).
    pub terminate_request: usize,
    /// `[3]` `GetMessagesAccessor()` — inherited (returns C_String accessor).
    pub get_messages_accessor: usize,
    /// `[4]` `Activate()` — override (human-specific: bind nav, register on minimap,
    /// bind to brain types 1..13).
    pub activate: usize,
    /// `[5]` `Inactivate()` — override (cleanup: nav agent, brain, AI state).
    ///
    /// `M2DE_CHumanAIController_Inactivate` (`0x140DAC750`).
    pub inactivate: FnInactivate,
    /// `[6]` `bool IsAIStateActive_A() const` — getter на флаг в `C_HumanAIState`
    /// (вызывает helper на `this+0x70`).
    pub state_query_a: usize,
    /// `[7]` `bool IsAIStateActive_B() const` — аналогичный getter
    /// на смежный флаг в `C_HumanAIState`.
    pub state_query_b: usize,
    /// `[8]` `AIPreUpdate()` — override (human-specific pre-tick, 1864 байт).
    pub ai_pre_update: usize,
    /// `[9]` `AIPostUpdate()` — override (post-tick, 547 байт).
    pub ai_post_update: usize,
    /// `[10]` `ProcessMessage(C_EntityMessage* msg)` — обработка AI-сообщений
    /// (urgent/info/etc.) через большой switch на `msg->id`. 1906 байт.
    pub process_message: usize,
    /// `[11]` CRT thunk artifact (5 байт). Не используется как vtable-метод.
    pub _slot_11_artifact: usize,
    /// `[12]` Thunk -> `C_AIController::GameSave(C_BitStream*)` (base impl).
    pub game_save: usize,
    /// `[13]` Thunk -> `C_AIController::GameLoad(C_BitStream*)` (base impl).
    pub game_load: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CHumanAIControllerVTable>() == 14 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, activate) == 4 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, inactivate) == 5 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, ai_pre_update) == 8 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, ai_post_update) == 9 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, process_message) == 10 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, game_save) == 12 * 8);
    assert!(std::mem::offset_of!(CHumanAIControllerVTable, game_load) == 13 * 8);
};

// =============================================================================
//  C_HumanAIState (3 slots)
// =============================================================================

/// VTable `C_HumanAIState`. **2 слота в DE.**
///
/// Очень компактная vtable — большинство методов state'а вызываются
/// напрямую (не через vtable).
#[repr(C)]
pub struct CHumanAIStateVTable {
    /// `[0]` `OnEntityDeactivate(C_Entity*)` — callback при деактивации связанной entity.
    pub on_entity_deactivate: usize,
    /// `[1]` `~C_HumanAIState()` — combined destructor.
    pub dtor: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CHumanAIStateVTable>() == 2 * 8);
};

// =============================================================================
//  game::ai::C_MafiaNavAgent (8 slots)
// =============================================================================

/// VTable `game::ai::C_MafiaNavAgent` (наследник `ue::ai::nav::C_Agent`).
#[repr(C)]
pub struct CMafiaNavAgentVTable {
    /// `[0]` `~C_MafiaNavAgent()` — combined destructor.
    pub dtor: usize,
    /// `[1]` `Register()` — регистрация в Navigation singleton.
    pub register: usize,
    /// `[2]` `Unregister()` — дерегистрация.
    pub unregister: usize,
    /// `[3]` Inherited slot (semantic TBD).
    pub _slot_3: usize,
    /// `[4]` `Update()` — тик (path-finding).
    pub update: usize,
    /// `[5]` `SetPosition(C_Vector const&)` — установить позицию агента.
    pub set_position: usize,
    /// `[6]` `GetPosition(C_Vector&)` — получить позицию.
    pub get_position: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CMafiaNavAgentVTable>() == 7 * 8);
};

// =============================================================================
//  ue::ai::framework::C_AITask (5 slots)
// =============================================================================

/// VTable `ue::ai::framework::C_AITask`. **4 слота в DE.**
#[repr(C)]
pub struct CAITaskVTable {
    /// `[0]` `~C_AITask()` — combined destructor.
    pub dtor: usize,
    /// `[1]` Reserved (возможно `Activate`).
    pub _slot_1_activate: usize,
    /// `[2]` Reserved (возможно `Inactivate`).
    pub _slot_2_inactivate: usize,
    /// `[3]` `GetDefaultSleepTime()` — частота тикования задачи.
    pub get_default_sleep_time: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CAITaskVTable>() == 4 * 8);
};

// =============================================================================
//  game::ai::C_HumanAIResources (10 slots)
// =============================================================================

/// VTable `game::ai::C_HumanAIResources`. 9 слотов.
///
/// Большой ресурс-контейнер (alloc 3792 байт). Слот[0] — combined dtor;
/// slot[1] — массивная cleanup-цепочка всех subsystem'ов; slot[2] — `Reset(level)`
/// с условной очисткой полей по их severity; slots[3..8] — compact accessors
/// (8 байт каждый, простые `return field`).
#[repr(C)]
pub struct CHumanAIResourcesVTable {
    /// `[0]` `~C_HumanAIResources()` — combined destructor.
    pub dtor: usize,
    /// `[1]` `DeactivateSubsystems()` — массивная очистка всех sub-resource'ов
    /// (~50 SmartPtr release / inline destructor calls для каждой подсистемы).
    /// Вызывается перед destruct'ом или при reset на максимальном уровне.
    pub deactivate_subsystems: usize,
    /// `[2]` `Reset(uint level)` — обнуление состояния с условным cleanup
    /// в зависимости от уровня (поля reset'ятся только если их severity < level).
    pub reset: usize,
    /// `[3]` `UnLockTimeLock(uint)` — снятие time-lock'а.
    pub unlock_time_lock: usize,
    /// `[4]` `ActHeadPos()` — текущая позиция головы (compact accessor).
    pub act_head_pos: usize,
    /// `[5]` `ActHandsPos()` — текущая позиция рук.
    pub act_hands_pos: usize,
    /// `[6]` `ActBodyPos()` — текущая позиция тела.
    pub act_body_pos: usize,
    /// `[7]` `ActHeadDir()` — текущее направление головы.
    pub act_head_dir: usize,
    /// `[8]` `ActHandsDir()` — текущее направление рук.
    pub act_hands_dir: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CHumanAIResourcesVTable>() == 9 * 8);
};
