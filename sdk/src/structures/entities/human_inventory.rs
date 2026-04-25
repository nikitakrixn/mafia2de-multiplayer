//! `C_HumanInventory` — инвентарь гуманоида (наследник `C_Inventory`).
//!
//! Создаётся как inline-объект внутри компонентного блока `I_Human2`
//! и наполняется в `M2DE_CHumanInventory_FullInit` (`0x140D72520`).
//!
//! ## Связь с публичным API инвентаря
//!
//! Верхнеуровневые данные инвентаря (видимые слоты, деньги) описаны в
//! `structures::entities::inventory::Inventory`. `C_HumanInventory` —
//! низкоуровневый компонент движка, который содержит resources и slots
//! и управляет ими через vtable.
//!
//! ## Иерархия
//!
//! ```text
//! C_Inventory (база, ~0x88 байт)
//!   └─ C_HumanInventory (расширение для humanoid'ов, всего 0x178 = 376 байт)
//!        └─ inline C_HumanThrowInventory @ +0xE0 (метательные предметы)
//! ```
//!
//! ## Содержимое инвентаря (создаётся в FullInit)
//!
//! ### Resources (4 штуки)
//!
//! | Index | Тип | Acceptable items |
//! |:-----:|:----|:-----------------|
//! | 0 | Type0, alloc 40 байт | bits 0, 8 |
//! | 1 | Type0, alloc 40 байт | bits 0, 8 |
//! | 2 | Special (тип -1), alloc 48 байт | bit 0 |
//! | 3 | Special (тип -1), alloc 48 байт | bit 0 |
//!
//! ### Slots (7 штук) — связаны с resources
//!
//! Slot[0,1] = слоты оружия (primary/secondary).
//! Slot[2,3] = специальные слоты A/B.
//! Slot[4,5,6] = item-slots для различных предметов (типы 7, 8, 10).
//!
//! ### Стартовое оружие (2 штуки)
//!
//! Конструктор сразу добавляет 2x `C_WeaponItem` (size 0x14 каждый) и
//! вызывает `SelectNextWeapon`. Это создаёт начальный набор `EmptyHands`.
//!
//! ## Layout (по `M2DE_CHumanInventory_FullInit`)
//!
//! | Offset | Поле | Тип |
//! |:-------|:-----|:----|
//! | +0x00 | vtable (`M2DE_VT_CHumanInventory`) | ptr |
//! | +0x08..+0x88 | base fields `C_Inventory` (vectors resource/slot, `master_inv`, `slave_inv`) | mixed |
//! | +0x88 | back-ref на `I_Human2*` (owner) | ptr |
//! | +0x90 | флаги/состояние (init = 0) | qword |
//! | +0x98 | флаги/состояние (init = 0) | qword |
//! | +0xA0 | флаги/состояние (init = 0) | qword |
//! | +0xA8 | init = -1 | qword |
//! | +0xB0 | init = -1 | dword |
//! | +0xB8 | init = 0 | qword |
//! | +0xC0 | `active_resource_id` (init = -1) | i32 |
//! | +0xC8..+0xE0 | три qword'а (init = 0) | qwords |
//! | +0xE0..+0x168 | inline `C_HumanThrowInventory` (~136 байт) | inline |
//! | +0x168 | флаг состояния (init = 0) | u8 |
//!
//! **Размер:** `0x170` (368 байт), подтверждено через caller alloc:
//! `M2DE_GlobalAlloc(368)` -> `M2DE_CHumanInventory_FullInit`.

use std::ffi::c_void;

/// `C_HumanInventory` — компонент инвентаря гуманоида (376 байт).
///
/// Низкоуровневая структура движка. Получить указатель на неё можно
/// через `CHuman::inventory` (поле `+0xE8`).
#[repr(C)]
pub struct CHumanInventory {
    /// Vtable `M2DE_VT_CHumanInventory` (`0x1418E3090`).
    pub vtable: *const c_void,
    /// Базовые поля `C_Inventory`: vectors resource/slot, master_inv, slave_inv.
    /// Точная разметка см. `inventory_components::CInventory`.
    pub _inventory_base: [u8; 0x80],
    /// Back-ref на гуманоида-владельца (`I_Human2*`).
    /// Устанавливается на `+0x88` в `M2DE_CHumanInventory_FullInit`.
    pub owner: *mut c_void,
    /// Зона флагов/состояния `+0x90..+0xC0` (различные init-значения).
    pub _state_block: [u8; 0x30],
    /// `active_resource_id` (init = -1) — индекс активного ресурса.
    pub active_resource_id: i32,
    /// Padding до начала throw-inventory блока.
    pub _padding_c4: [u8; 0x1C],
    /// Inline `C_HumanThrowInventory` (~136 байт).
    /// Хранит метательные предметы (грн, коктейли) в отдельной структуре.
    pub _throw_inventory: [u8; 0x88],
    /// Флаг состояния (например, `throw_grenade_active`).
    pub state_flag: u8,
    /// Padding до конца структуры (`0x170 - 0x169 = 7` байт).
    pub _padding_tail: [u8; 0x7],
}

const _: () = {
    assert!(std::mem::size_of::<CHumanInventory>() == 0x170);
    assert!(std::mem::offset_of!(CHumanInventory, owner) == 0x88);
    assert!(std::mem::offset_of!(CHumanInventory, active_resource_id) == 0xC0);
    assert!(std::mem::offset_of!(CHumanInventory, state_flag) == 0x168);
};
