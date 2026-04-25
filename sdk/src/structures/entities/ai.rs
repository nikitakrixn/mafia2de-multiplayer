//! AI-подсистема гуманоидов: контроллеры, состояния, навигация, задачи.
//!
//! ## Иерархия классов
//!
//! ```text
//! game::ai::C_AIController         (база, ~56 байт)
//!   └─ C_HumanAIController         (расширение для humans, ~376 байт)
//!        ├─ inline C_HumanAIState  (~340 байт, поведение/восприятие)
//!        ├─ heap: C_HumanAIResources (~2276 байт, animации/движение/оружие AI)
//!        ├─ heap: C_MafiaNavAgent   (~180 байт, нав-агент)
//!        ├─ heap: C_HumanStateVariables (~60 байт, state variables)
//!        ├─ heap: C_ServiceQuery (~44 байт)
//!        └─ heap: AnimSettings, TimerData (мелкие блоки)
//!
//! ue::ai::framework::C_AITask      (~24 байт, задача в global scheduler)
//! ```
//!
//! ## Где живёт
//!
//! `CHumanAIController` хранится по указателю в `CHumanNPC + 0xF8`
//! (см. `M2DE_CHuman_BaseConstructor`). Создаётся внутри компонентного
//! блока `I_Human2` как `C_HumanAIController(this, -1, NULL)`.
//!
//! ## Lifecycle
//!
//! - **Создание:** `I_Human2` ctor аллоцирует и конструирует
//! - **Активация:** `I_Human2::OnActivate` -> `C_HumanAIController::Activate`
//!   -> биндит мозг (brain) к agent через `BindAgentToBrain`
//! - **Тик:** `C_Human2::TickPrePhysics` -> `CommandUpdate` -> AI logic
//! - **Деактивация:** `M2DE_CHumanAIController_Inactivate` (vtable[5]) —
//!   симметрично OnActivate, безопасно снимает task из scheduler

use crate::macros::assert_field_offsets;
use crate::memory;
use crate::structures::vtables::entities::ai::CHumanAIControllerVTable;
use std::ffi::c_void;

// =============================================================================
//  game::ai::C_AIController — базовый AI-контроллер
// =============================================================================

/// `game::ai::C_AIController` — базовый AI-контроллер.
///
/// ## Конструктор
///
/// `C_AIController(C_AIController*, C_Actor* actor, E_BrainType brain_type, S_EntityInitProps* props)`
///
/// IDA: `0x1401BFE10` (`M2DE_CAIController_Constructor`).
/// VTable: `0x14185B238` (`M2DE_vtbl_CAIController`).
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable | ptr |
/// | +0x08 | _ext_08 (heap, alloc 16) | ptr |
/// | +0x10 | actor (back-ref на C_Actor*) | ptr |
/// | +0x18 | ai_task (`C_AITask*`) | ptr |
/// | +0x20 | _ext_20 (heap, alloc 12, sentinel-ptr к самому себе) | ptr |
/// | +0x28 | init_props (`S_EntityInitProps*`) | ptr |
/// | +0x30..+0x40 | vector<sub-task>::begin/end/cap | 3× ptr |
/// | +0x48..+0x58 | vector<C_EntityMessage*>::begin/end/cap | 3× ptr |
/// | +0x60 | cycle_list (sentinel, alloc 0x30, self-ref) | ptr |
#[repr(C)]
#[allow(non_snake_case)]
pub struct CAIController {
    /// `+0x000` VTable.
    pub vtable: *const c_void,

    /// `+0x008` Heap-указатель (alloc 16, init by `sub_1401C3FA0`).
    pub _ext_08: *mut c_void,

    /// `+0x010` Целевая `C_Actor*` (target/owner entity).
    pub entity: *mut c_void,

    /// `+0x018` Указатель на `C_AITask`.
    pub ai_task: *mut CAITask,

    /// `+0x020` Heap-блок (alloc 16) — sentinel-list head.
    pub _ext_20: *mut c_void,

    /// `+0x028` `S_EntityInitProps*` (или mode-арг, передаётся как `a4`).
    pub init_props: *mut c_void,

    /// `+0x030..+0x040` Vector<sub-task>::begin/end/cap.
    pub _subtasks_begin: *mut c_void,
    pub _subtasks_end: *mut c_void,
    pub _subtasks_cap: *mut c_void,

    /// `+0x048..+0x058` Vector<C_EntityMessage*>::begin/end/cap (pending messages).
    pub _messages_begin: *mut c_void,
    pub _messages_end: *mut c_void,
    pub _messages_cap: *mut c_void,

    /// `+0x060` Cycle-list head (sentinel, alloc 0x30, points to self).
    pub _cycle_list: *mut c_void,
}

assert_field_offsets!(CAIController {
    vtable     == 0x000,
    entity     == 0x010,
    ai_task    == 0x018,
    init_props == 0x028,
});

impl CAIController {
    /// Смещение в `CHumanNPC` где хранится указатель на контроллер.
    pub const HUMAN_OFFSET: usize = 0xF8;

    /// Получить контроллер из `CHumanNPC` entity pointer.
    ///
    /// # Safety
    ///
    /// `entity` должен указывать на валидный `CHumanNPC`.
    #[inline]
    pub unsafe fn from_human(entity: usize) -> Option<&'static mut Self> {
        if !memory::is_valid_ptr(entity) {
            return None;
        }
        let slot = entity + Self::HUMAN_OFFSET;
        let ptr = unsafe { memory::read_value::<usize>(slot)? };
        if ptr == 0 || !memory::is_valid_ptr(ptr) {
            return None;
        }
        Some(unsafe { &mut *(ptr as *mut Self) })
    }

    /// Указатель на C_AITask. `None` если NULL/invalid.
    #[inline]
    pub fn task(&self) -> Option<&mut CAITask> {
        let p = self.ai_task as usize;
        if p == 0 || !memory::is_valid_ptr(p) {
            return None;
        }
        Some(unsafe { &mut *self.ai_task })
    }

    /// Получить функцию из vtable по slot index.
    #[inline]
    pub fn vtable_fn(&self, slot: usize) -> Option<usize> {
        let vt = self.vtable as usize;
        if vt == 0 || !memory::is_valid_ptr(vt) {
            return None;
        }
        let entry = unsafe { memory::read_value::<usize>(vt + slot * 8)? };
        if entry == 0 { None } else { Some(entry) }
    }
}

// =============================================================================
//  C_HumanAIController — расширение для гуманоидов
// =============================================================================

/// `C_HumanAIController` — специализированный AI-контроллер для humanoid'ов.
///
/// ## Конструктор
///
/// `C_HumanAIController(C_HumanAIController*, I_Human2* human)`
///
/// IDA: `0x140D71AD0` (`M2DE_CHumanAIController_Constructor`).
/// VTable: `0x1418E37A8` (`M2DE_vtbl_HumanBehaviorComponent`).
///
/// ## Layout
///
/// | Offset | Поле | Тип / Назначение |
/// |:-------|:-----|:------------------|
/// | +0x000 | vtable | ptr |
/// | +0x008 | `ai_resources` | `C_HumanAIResources*` (heap, alloc **0xEF0** = 3792) |
/// | +0x010..+0x070 | базовые поля `C_AIController` (entity, ai_task, vectors) | inline |
/// | +0x070 | inline `C_HumanAIState` (init via `C_HumanAIState::Ctor`) | inline ~440 байт |
/// | +0x1F8..+0x208 | flags/state | qwords |
/// | +0x208 | `state_variables` | `C_HumanStateVariables*` (heap, alloc 0x70) |
/// | +0x210 | `nav_agent` | `C_MafiaNavAgent*` (heap, alloc **0x108** = 264) |
/// | +0x218 | sub-controller (alloc 0x40, vt `off_1418E3448`) | ptr |
/// | +0x220 | sub-controller (alloc 0x38) | ptr |
/// | +0x228 | sub-controller (alloc 0x18) | ptr |
/// | +0x230 | `id_ptr` (heap `int*`, init = -1) | `*mut i32` |
/// | +0x240..+0x260 | прочие флаги/счётчики | mixed |
///
/// **Размер:** `~0x260` (608 байт).
///
/// ## Слот `[5]` vtable = `Inactivate`
///
/// `M2DE_CHumanAIController_Inactivate` (`0x140DAC750`) — безопасная
/// engine-symmetric AI deactivation. Делает:
/// - dereg на minimap
/// - `C_AIController::Inactivate` (база)
/// - SmartPtr release (nav agent, service query)
/// - `C_HumanAIState::DeActivate`
///
/// Не трогает render/lifecycle. Используй вместо прямого clear на C_AITask.
#[repr(C)]
pub struct CHumanAIController {
    /// Базовый контроллер (vtable + 0x68 байт base layout).
    pub base: CAIController,
    /// Inline `C_HumanAIState` начинается на `+0x70`. Зарезервированная зона
    /// для inline state'а и всех heap-указателей на subsystem'ы.
    pub _human_specific: [u8; 0x1F8],
}

impl CHumanAIController {
    /// Vtable slot номер для `Inactivate` метода.
    pub const VT_INACTIVATE_SLOT: usize = 5;

    /// Получить контроллер из `CHumanNPC` entity pointer.
    ///
    /// # Safety
    ///
    /// `entity` должен указывать на валидный `CHumanNPC`.
    #[inline]
    pub unsafe fn from_human(entity: usize) -> Option<&'static mut Self> {
        let base = unsafe { CAIController::from_human(entity)? };
        Some(unsafe { &mut *(base as *mut CAIController as *mut Self) })
    }

    /// Типизированный VTable.
    #[inline]
    pub fn vtable(&self) -> Option<&CHumanAIControllerVTable> {
        let vt = self.base.vtable as *const CHumanAIControllerVTable;
        if vt.is_null() || !memory::is_valid_ptr(vt as usize) {
            return None;
        }
        Some(unsafe { &*vt })
    }

    /// Engine-symmetric AI deactivation через `vtable[5]` dispatch.
    ///
    /// Вызывает `M2DE_CHumanAIController_Inactivate` со всем cleanup chain'ом
    /// (minimap dereg, base AIController::Inactivate, SmartPtr release,
    /// AIState::DeActivate, nav agent release).
    ///
    /// Не трогает render/lifecycle. Симметрично OnActivate.
    ///
    /// # Safety
    ///
    /// Вызывать только из game thread пока entity жив.
    #[inline]
    pub unsafe fn inactivate(&mut self) -> Result<(), &'static str> {
        let vt = self.vtable().ok_or("vtable NULL")?;
        unsafe { (vt.inactivate)(self as *mut Self as *mut c_void) };
        Ok(())
    }
}

// =============================================================================
//  C_HumanAIState — состояние AI-восприятия
// =============================================================================

/// `C_HumanAIState` — состояние AI восприятия и поведения гуманоида.
///
/// Большая структура с sentinel-узлами для linked-list'ов percepts,
/// look-at/aim-at позиции, weak/cnt-ptr на target entities,
/// status flags, state machine.
///
/// ## Конструктор
///
/// `C_HumanAIState(C_HumanAIState*, I_Human2* human)` — создаётся
/// inline внутри `C_HumanAIController` на `+0x70`.
///
/// IDA: `0x140D72030` (`M2DE_CHumanAIState_Ctor`).
/// VTable: `0x1418E34E0` (2 слота).
///
/// ## Lifecycle
///
/// - `Activate(human)` — вызывается из `C_HumanAIController::Activate`
/// - `DeActivate()` — вызывается из `Inactivate` (vtable[5])
/// - `UpdateHumanFreq(dt)` — тик (вызывается из `TickPostPhysics_Base`)
///
/// ## Размер
///
/// Inline-объект внутри `CHumanAIController +0x70..+0x1F8` = `0x188` байт
/// (392 байт). Max field offset из ctor = `0x182`.
#[repr(C)]
pub struct CHumanAIState {
    /// VTable (`M2DE_VT_CHumanAIState` @ `0x1418E34E0`).
    pub vtable: *const c_void,
    /// Зарезервированная зона ~384 байт. Подробный layout не описан,
    /// доступ через vtable-методы или прямые offsets.
    pub _data: [u8; 0x180],
}

const _: () = {
    assert!(std::mem::size_of::<CHumanAIState>() == 0x188);
};

// =============================================================================
//  C_MafiaNavAgent — нав-агент
// =============================================================================

/// `game::ai::C_MafiaNavAgent` — навигационный агент гуманоида.
///
/// Наследник `ue::ai::nav::C_Agent`. Выполняет path-finding по
/// nav-mesh, регистрируется в global Navigation singleton.
///
/// ## Конструктор
///
/// IDA: `0x140EAB290` (`M2DE_CMafiaNavAgent_Ctor`).
/// Alloc: **`M2DE_GlobalAlloc(264)` = 0x108 байт**.
/// VTable: `0x1418F43D8` (7 слотов).
///
/// ## Methods
///
/// - `IsRegistered()` — проверка регистрации в Navigation
/// - destructor — авто-deregister + release smartptr
#[repr(C)]
pub struct CMafiaNavAgent {
    pub vtable: *const c_void,
    /// Зарезервированная зона 0x100 байт = 256 байт.
    pub _data: [u8; 0x100],
}

const _: () = {
    assert!(std::mem::size_of::<CMafiaNavAgent>() == 0x108);
};

// =============================================================================
//  C_HumanStateVariables — переменные состояния
// =============================================================================

/// `C_HumanStateVariables` — набор state-variables для HumanAI.
///
/// Содержит 6 inline `C_StateVariableBase` объектов + `C_CoverSlotPtr`.
///
/// ## Конструктор
///
/// `C_HumanStateVariables(C_HumanStateVariables*, C_HumanAIController*)`
///
/// IDA: `0x140D72CC0` (`M2DE_CHumanStateVariables_Ctor`).
/// Alloc: **`M2DE_GlobalAlloc(112)` = 0x70 байт**.
#[repr(C)]
pub struct CHumanStateVariables {
    /// Back-ref на AI controller-владельца.
    pub owner_ai_controller: *mut c_void,
    /// Зарезервированная зона 104 байт.
    pub _data: [u8; 0x68],
}

const _: () = {
    assert!(std::mem::size_of::<CHumanStateVariables>() == 0x70);
};

// =============================================================================
//  C_HumanAIResources — ресурсы AI (анимации, движение, оружие)
// =============================================================================

/// `game::ai::C_HumanAIResources` — большой контейнер AI-ресурсов.
///
/// **Размер:** `0xEF0` (3792 байт).
///
/// Содержит огромное количество inline `AIResource_*` объектов:
/// `C_AIResource_HumanAnim`, `C_AIResource_Movement`, `C_AIValue_Base`'ы,
/// `C_AIWeapon_Mafia`, и десятки других подсистем.
///
/// ## Конструктор
///
/// `C_HumanAIResources(C_HumanAIResources*, C_Actor* actor)` — создаётся
/// в `C_HumanAIController` ctor через `M2DE_GlobalAlloc(3792)`.
///
/// IDA: `0x140D71CD0` (`M2DE_CHumanAIResources_Ctor`).
/// VTable: `0x1418E3760` (9 слотов).
///
/// ## Доступ из CHumanAIController
///
/// Heap-allocated, хранится в `CHumanAIController + 0x008`.
#[repr(C)]
pub struct CHumanAIResources {
    pub vtable: *const c_void,
    /// Закрытая зона 3784 байт. Реальная аллокация = `0xEF0` (3792).
    pub _data: [u8; 0xEE8],
}

const _: () = {
    assert!(std::mem::size_of::<CHumanAIResources>() == 0xEF0);
};

// =============================================================================
//  C_AITask — единица работы в AI scheduler'е
// =============================================================================

/// `ue::ai::framework::C_AITask` — задача в global AI scheduler'е.
///
/// Регистрируется в global `C_AIFrameWork::instance` vector. Жизненный
/// цикл управляется через `Activate` / `Inactivate` (`M2DE_CAITask_Inactivate`
/// @ `0x14018C7E0`).
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable | ptr |
/// | +0x08 | flags | u8 |
/// | +0x0E | slot_id (default = -1) | i16 |
/// | +0x10 | _vector_a | ptr |
/// | +0x18 | _vector_b | ptr |
/// | +0x20 | _vector_c | ptr |
/// | +0x28 | unique_counter (s_UniqueCounter++) | u32 |
///
/// ## Биты `flags`
///
/// - bit 0 (`ACTIVE`)             — задача в global scheduler vector
/// - bit 1 (`PENDING_INACTIVATE`) — Inactivate уже вызвана (early-exit guard)
/// - bit 2 (`MODE`)               — выбор vector index в scheduler (8 или 20)
///
/// В дизассемблере читается как WORD (`a1[4]` где `a1` имеет тип `_WORD*`),
/// но clear выполняется через младший байт (`*(BYTE*)(this+8) &= ~1u`).
#[repr(C)]
pub struct CAITask {
    /// `+0x000` VTable.
    pub vtable: *const c_void,

    /// `+0x008` Битовое поле флагов (см. [`flags`]).
    pub flags: u8,
    pub _pad_09: [u8; 5],
    /// `+0x00E` Slot ID (default = -1 = unassigned).
    pub slot_id: i16,
    /// `+0x010..+0x028` Внутренние vector'ы.
    pub _vec_a: *mut c_void,
    pub _vec_b: *mut c_void,
    pub _vec_c: *mut c_void,
    /// `+0x028` Уникальный счётчик (выдаётся из `s_UniqueCounter`).
    pub unique_id: u32,
    pub _pad_2c: u32,
}

assert_field_offsets!(CAITask {
    vtable    == 0x000,
    flags     == 0x008,
    slot_id   == 0x00E,
    _vec_a    == 0x010,
    unique_id == 0x028,
});

/// Битовые флаги [`CAITask::flags`].
pub mod flags {
    /// Задача активна и зарегистрирована в global scheduler.
    pub const ACTIVE: u8 = 0x01;
    /// `Inactivate` уже была вызвана (early-exit guard).
    pub const PENDING_INACTIVATE: u8 = 0x02;
    /// Mode-bit (выбирает vector в scheduler).
    pub const MODE: u8 = 0x04;
}

impl CAITask {
    /// Активна ли задача (bit 0).
    #[inline]
    pub fn is_active(&self) -> bool {
        (self.flags & flags::ACTIVE) != 0
    }

    /// `Inactivate` уже вызвана (bit 1).
    #[inline]
    pub fn is_pending_inactivate(&self) -> bool {
        (self.flags & flags::PENDING_INACTIVATE) != 0
    }

    /// ⚠️ **Не симметрично с engine** — клирит только bit, не снимает task
    /// из global scheduler. На следующем тике AI dispatch обходит scheduler
    /// и крашится на inactive task с активным sub-state.
    ///
    /// Используй [`CHumanAIController::inactivate`] вместо этого.
    ///
    /// # Safety
    ///
    /// Только для diagnostic-целей, не для production.
    #[inline]
    pub unsafe fn force_clear_active_unsafe(&mut self) -> u8 {
        let before = self.flags;
        self.flags &= !flags::ACTIVE;
        before
    }
}
