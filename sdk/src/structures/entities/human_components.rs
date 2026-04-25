//! Не-AI компоненты гуманоида: оружие, голова, цвета фрейма, эмиттеры.
//!
//! Все эти компоненты создаются как inline-объекты внутри компонентного
//! блока `I_Human2` (alloc 0xA58 байт). Доступ к ним через указатели в
//! `CHuman` (`+0xB8 frame_colors`, `+0x108 weapon_controller`, и т.д.).
//!
//! AI-компоненты (`CHumanAIController`, `CHumanAIState`, `CMafiaNavAgent`,
//! `CHumanStateVariables`, `CHumanAIResources`) живут в отдельном модуле
//! `entities::ai`.
//!
//! ## Иерархия компонентов гуманоида (полный список)
//!
//! ```text
//! I_Human2 alloc-block (0xA58 bytes)
//!   ├─ mafia::C_FrameColors      -> CHuman.frame_colors (+0xB8)
//!   ├─ C_HumanInventory          -> CHuman.inventory (+0xE8)        [-> entities::human_inventory]
//!   ├─ C_HumanAIController       -> CHuman.ai_controller (+0xF8)    [-> entities::ai]
//!   ├─ C_HumanWeaponController   -> CHuman.weapon_controller (+0x108)
//!   ├─ C_HumanHeadController     -> CHuman.head_controller (+0x110)
//!   └─ C_PlayerEmitter × 5       -> CHuman.emitter_* поля
//! ```
//!
//! ## TODO
//!
//! Точные структур требуют верификации а так же поля... Всё пока не однозначно.
//! Сейчас размеры приблизительные.

use std::ffi::c_void;

// =============================================================================
//  C_HumanWeaponController
// =============================================================================

/// `C_HumanWeaponController` — управление оружием гуманоида.
///
/// Содержит 3 inline `C_FrameColors` (для weapon material variants),
/// набор queue/list полей, `C_Human2ItemManager` (heap).
///
/// ## Конструктор
///
/// `C_HumanWeaponController()` — без параметров. Human передаётся отдельно
/// в `GameInit(human)`.
///
/// ## Layout (приблизительный)
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00..+0x88 | счётчики и временные поля | DWORD'ы |
/// | +0x90 | inline `C_FrameColors` #1 (weapon material A) | 48 байт |
/// | +0xC0 | inline `C_FrameColors` #2 (weapon material B) | 48 байт |
/// | +0xF0 | inline `C_FrameColors` #3 (weapon material C) | 48 байт |
/// | +0x120 | временные XMM-поля (animation/aim) | 32 байт |
/// | +0x148 | угол прицела | float |
/// | +0x14C | timer | float |
/// | +0x190..+0x290 | поля очереди оружия | DWORD pairs |
/// | +0x294 | item_manager | `C_Human2ItemManager*` (heap, alloc 0x48) |
/// | +0x204 | флаг (-1 default) | byte |
/// | +0x268 | флаг (-1 default) | byte |
///
/// **Размер:** ~870-920.
#[repr(C)]
pub struct CHumanWeaponController {
    pub vtable: *const c_void,
    pub _data: [u8; 0x390],
}

// =============================================================================
//  C_HumanHeadController
// =============================================================================

/// `C_HumanHeadController` — управление поворотом головы и направлением взгляда.
///
/// ## Конструктор
///
/// `C_HumanHeadController()` — простой обнуляющий ctor. Vtable
/// устанавливается отдельно (не в ctor).
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00..+0x1F | 8 DWORD'ов (look-at target, aim direction, blend factor) | — |
///
/// **Размер:** ~32-48
#[repr(C)]
pub struct CHumanHeadController {
    pub _data: [u8; 0x30],
}

// =============================================================================
//  mafia::C_FrameColors
// =============================================================================

/// `mafia::C_FrameColors` — цвета и материалы модели гуманоида.
///
/// Содержит данные о цветовых слотах материалов (грязь, кровь, пыль,
/// burn-effect, и т.д.). Имеет self-ref на `+32` который служит sentinel'ом
/// для пустого списка цветовых слотов.
///
/// ## Конструктор
///
/// `mafia::C_FrameColors()` — обнуляет 12 DWORD-ов, устанавливает self-ref.
///
/// ## Ключевые методы
///
/// - `Init(C_Frame* frame, uint config_id)` — инициализация из таблицы
/// - `InitHR(...)` — high-resolution variant
/// - `RestoreMaterials(C_Frame*, bool, uint)` — восстановление материалов
/// - `ChangeColor()` — смена цветовой схемы
/// - `SetColor(uint color, bool flag)` — установка цвета
/// - `Done()` — деактивация (вызывается в `I_Human2::GameDone`)
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00..+0x1F | 8 DWORD-полей (material slots) | — |
/// | +0x20 | self-ref на `this+0x24` (sentinel) | ptr |
/// | +0x24 | tables data ptr | ptr |
/// | +0x28..+0x2C | дополнительные DWORD-поля | — |
///
/// **Размер:** ~64-72
#[repr(C)]
pub struct CFrameColors {
    pub _data: [u8; 0x48],
}

// =============================================================================
//  C_PlayerEmitter
// =============================================================================

/// `C_PlayerEmitter` — AI emitter гуманоида (sight/hear система).
///
/// Каждый гуманоид имеет несколько emitter'ов с разными `type`-параметрами
/// (0, 1, 3, 5). Тип определяет назначение emitter'а:
///
/// | Type | Назначение |
/// |:----:|:-----------|
/// | 0 | Основной sight emitter (видим для других AI) |
/// | 1 | Back-ref emitter (для tracking-связей) |
/// | 3 | Синхронизация трансформов (привязка к bone'ам) |
/// | 5 | Дополнительный AI emitter (специальные ситуации) |
///
/// ## Конструктор
///
/// `C_PlayerEmitter::ConstructBase(this, &type)` — создаётся в
/// `I_Human2` ctor через base-ctor + установка vtable.
///
/// ## Метод Update
///
/// `Update()` синхронизирует позицию emitter'а с трансформом гуманоида
/// (вызывает `vtable[65/8] = vtable[8]` локомоушна).
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable | ptr |
/// | +0x08 | emitter_type (0/1/3/5) | u32 |
/// | +0x0C | tick state | qword |
/// | +0x14 | last update tick | u32 |
/// | +0x18 | back-ref на гуманоида (`I_Human2*`) | ptr |
/// | +0x20 | inner sight cone params (XMM) | 16 байт |
/// | +0x30 | activation flag | u8 |
/// | +0x34 | extra params | u32 |
///
/// **Размер:** ~72-80
#[repr(C)]
pub struct CPlayerEmitter {
    pub vtable: *const c_void,
    /// Тип эмиттера (0/1/3/5).
    pub emitter_type: i32,
    pub _pad_0c: u32,
    pub _data: [u8; 0x40],
}
