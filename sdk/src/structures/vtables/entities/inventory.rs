//! Типизированные VTable для классов системы инвентаря.
//!
//! ## Адреса в .rdata
//!
//! | Vtable | RVA | Описание |
//! |:-------|:----|:---------|
//! | `C_Inventory` base | `0x1418E3010` | 17 слотов, базовый контейнер |
//! | `C_HumanInventory` | `0x1418E3090` | 17 слотов, override большинства методов |
//! | `C_InventoryResource` type 0 | `0x1418EA390` | 10 слотов, обычные resources (40 байт) |
//! | `C_InventoryResource` type -1 | `0x1418E6E60` | 10 слотов, special resources (48 байт) |
//! | `C_InventorySlot` weapon A | `0x1418E6EA8` | 25 слотов, weapon-slot #1 (88 байт) |
//! | `C_InventorySlot` weapon B | `0x1418E6DA0` | 25 слотов, weapon-slot #2 (88 байт) |
//! | `C_InventorySlot` special A | `0x1418E6C78` | 25 слотов, special-slot A (80 байт) |
//! | `C_InventorySlot` special B | `0x1418E6B50` | 25 слотов, special-slot B (80 байт) |
//! | `C_InventorySlot` misc | `0x1418E6A68` | 25 слотов, misc item slot (80 байт) |
//! | `C_WeaponItem` primary | `0x1418EA180` | 14 слотов |
//! | `C_WeaponItem` MI sub-vtable | `0x14184C780` | adjustor для `C_InventoryItem` base |
//!
//! ## Иерархия наследования
//!
//! ```text
//! C_Inventory (17 slots)
//!   └─ C_HumanInventory (17 slots) — override [0..7, 12..16]
//!   └─ C_HumanThrowInventory (17 slots) — override только [0, 1]
//!
//! C_InventoryItem (база)
//!   └─ C_WeaponItem (14 slots)
//!
//! C_InventorySlot (25 slots, 5 вариантов)
//! C_InventoryResource (10 slots, 2 варианта)
//! ```
//!
//! ## Зоны слотов C_Inventory primary vtable (17 слотов)
//!
//! | Слоты | Назначение |
//! |:------|:-----------|
//! | 0–1   | Жизненный цикл (dtor + deleting dtor) |
//! | 2     | `Reset()` — очистка содержимого |
//! | 3–4   | `GameInit()` / `GameDone()` |
//! | 5–7   | `GetItemAnchor`, `AddItem`, `RemoveItem` — основной item API |
//! | 8–11  | Master/Slave inventory hierarchy |
//! | 12–13 | Сериализация: `WriteToStream`, `ReadFromStream` |
//! | 14    | `Dump()` — debug-вывод |
//! | 15–16 | Callback hooks: `OnAddItem`, `OnRemoveItem` |
//!
//! ## Зоны слотов C_InventorySlot vtable (25 слотов)
//!
//! | Слоты | Назначение |
//! |:------|:-----------|
//! | 0–1   | Жизненный цикл |
//! | 2–4   | Сериализация + `GetType` |
//! | 5–10  | Compatibility checks: `Accepts`, `HasCapacityFor*` |
//! | 11–16 | `CanChangeSize`, `GetMaxSize`, `CanAdd*`, `CanAddInsteadOf*` |
//! | 17–18 | `AddItem`, `RemoveItem` — основной item API |
//! | 19    | `Dump(C_String const&)` |
//! | 20–22 | Anchor management: `Add/RemoveItemAnchor`, `RemoveItemAnchorByIndex` |
//! | 23–24 | Callbacks: `OnAddItem`, `OnRemoveItem` |

use std::ffi::c_void;

// =============================================================================
//  C_Inventory (17 slots)
// =============================================================================

/// VTable базового `C_Inventory`. 16 слотов.
///
/// ## Особенности архитектуры
///
/// - Один combined destructor (slot[0]) вместо отдельных `D1` + `D0` пары
/// - slot[1] — это `Reset()` с восстановлением default loadout
///   (создаёт 2 пустых `C_WeaponItem`)
/// - slots[8..10] — двусторонний unlink операций между slot ↔ resource
/// - slot[11] — сериализация двух DWORD ID'ов в `BitStream`
#[repr(C)]
pub struct CInventoryVTable {
    /// `[0]` `~C_Inventory(this, char delete_flag)` — combined deleting destructor.
    pub dtor: usize,
    /// `[1]` `Reset()` — восстанавливает default loadout.
    /// Override CHumanInventory: создаёт 2 пустых `C_WeaponItem(1, 0)` и вставляет.
    pub reset: usize,
    /// `[2]` `OnResetDelegate()` — proxy на subobject @ +224 (HumanThrowInventory).
    /// Тривиальный stub: `nullsub_117(); return (*this[224][2])(this+224);`
    pub on_reset_delegate: usize,
    /// `[3]` `GameInit()` — инициализация при старте игры.
    pub game_init: usize,
    /// `[4]` `GameDone(C_InventoryItem*)` — создаёт ItemAnchor (alloc 64 или 48).
    /// Возможно misnamed; реально это `CreateItemAnchor` или `RegisterItem`.
    pub game_done: usize,
    /// `[5]` `TryInsertToSlot(item, int slot_id, char flag)`.
    /// Использует `vt[13]`/`vt[16]` slot'а для CanAdd/AddItem dispatch.
    pub try_insert_to_slot: usize,
    /// `[6]` `AddItem(C_InventoryItem*)` — 5-байтовый thunk -> `sub_140E2C060`.
    pub add_item: usize,
    /// `[7]` `RemoveItem(C_InventoryItem*)` — vector erase из items @ +32.
    pub remove_item: usize,
    /// `[8]` `UnlinkSlotFromResource(C_InventoryResource*)` — **двусторонний unlink**.
    /// Удаляет res из this[+32..+40] (slots) и this из res[+56..+64] (back-refs).
    pub unlink_slot_from_resource: usize,
    /// `[9]` `RemoveResource(C_InventoryResource*)` — удаляет из vector @ +56..+64.
    /// Также чистит back-ref в res[+32] и вызывает `sub_140E2FD60` для recompute.
    pub remove_resource: usize,
    /// `[10]` `UnlinkResourceFromSlot(C_InventorySlot*)` — обратное к slot[8].
    /// Удаляет slot из this[+56..+64] (resources) и this из slot[+32..+40].
    pub unlink_resource_from_slot: usize,
    /// `[11]` `GameSaveIDs(C_BitStream*)` — сериализация двух DWORD ID'ов.
    /// Записывает `this[+176]` (ID#1) и `this[+172]` (ID#2) по 32 бита.
    pub game_save_ids: usize,
    /// `[12]` `WriteToStream(C_BitStream*)` — полная сериализация inventory.
    pub write_to_stream: usize,
    /// `[13]` `ReadFromStream(C_BitStream*)` — десериализация (читает 7 sub-elements).
    pub read_from_stream: usize,
    /// `[14]` `Dump(C_InventoryItem*)` — гибрид debug-dump + RB-tree поиска.
    /// 804 байт — самая большая helper-функция в vtable.
    pub dump: usize,
    /// `[15]` `OnAddItem(C_InventoryItem*)` — callback hook (960 байт).
    /// Обновляет внутреннее состояние при добавлении item.
    pub on_add_item: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CInventoryVTable>() == 16 * 8);
};

// =============================================================================
//  C_HumanInventory (17 slots) — те же что у C_Inventory, но с overrides
// =============================================================================

/// VTable `C_HumanInventory` (override от `C_Inventory`).
///
/// Layout идентичен `CInventoryVTable`. Большинство слотов переопределены
/// human-specific версиями кроме slot 6 (`AddItem` наследуется), 8-11
/// (master/slave связи).
pub type CHumanInventoryVTable = CInventoryVTable;

// =============================================================================
//  C_HumanThrowInventory (17 slots) — почти полностью наследует C_Inventory
// =============================================================================

/// VTable `C_HumanThrowInventory` (наследник `C_Inventory`).
///
/// Отличается от `C_Inventory` только slot 0 и 1 (свой destructor).
/// Все остальные методы inherited.
pub type CHumanThrowInventoryVTable = CInventoryVTable;

// =============================================================================
//  C_InventorySlot (25 slots)
// =============================================================================

/// VTable `C_InventorySlot`. 24 слота.
///
/// ## Архитектура
///
/// Существуют 5 variants: `WeaponA`, `WeaponB`, `SpecialA`, `SpecialB`, `Misc`.
/// Большинство методов **shared** (один адрес на все variants), отличаются
/// только slot[0] (variant-specific destructor) и небольшое количество
/// override'ов в SpecialA/B и WeaponB.
///
/// ## Замечания
///
/// - slot[1] — полный cleanup (~293 байт), не WriteToStream
/// - slot[2] обходит RB-tree и вызывает callback (255 байт)
/// - slot[3] — простой `return this[+16]` (4 байт), не ReadFromStream
/// - slot[4] — `return this[+20] & 1` (проверка bit 0 acceptable mask)
#[repr(C)]
pub struct CInventorySlotVTable {
    /// `[0]` `~C_InventorySlot()` — variant-specific destructor wrapper (~52 байт).
    pub dtor: usize,
    /// `[1]` `FullCleanup()` / `Reset()` — полная очистка sub-objects, vectors
    /// (~293 байт). Shared между всеми 5 variants. НЕ destructor.
    pub full_cleanup: usize,
    /// `[2]` `DispatchOverTree(callback)` — обходит RB-tree, вызывает callback
    /// (255 байт). Точная семантика TBD (возможно WriteToStream или EnumItems).
    pub dispatch_over_tree: usize,
    /// `[3]` `GetActiveResourceID() const` — простой getter `return this[+16]`.
    pub get_active_resource_id: usize,
    /// `[4]` `IsAcceptableType0() const` — getter на bit 0 acceptable_mask
    /// (`return this[+20] & 1`).
    pub is_acceptable_type0: usize,
    /// `[5]` `Accepts(C_InventoryItem const*)` — bitmask check
    /// (`(1 << typeID(item)) & this[+20]`).
    pub accepts_item: usize,
    /// `[6]` `Accepts(int type_id)`.
    pub accepts_type: usize,
    /// `[7]` `HasCapacityFor(C_InventoryItem const*)`.
    pub has_capacity_for_item: usize,
    /// `[8]` `HasCapacityFor(int type_id)`.
    pub has_capacity_for_type: usize,
    /// `[9]` `HasCapacityForInsteadOf(item, replaced_item)`.
    pub has_capacity_for_instead_of_item: usize,
    /// `[10]` `HasCapacityForInsteadOf(type_id, replaced_item)`.
    pub has_capacity_for_instead_of_type: usize,
    /// `[11]` `CanChangeSize(C_InventoryItem const*, int delta)`.
    pub can_change_size: usize,
    /// `[12]` `GetMaxSize(C_InventoryItem const*, int)`.
    pub get_max_size: usize,
    /// `[13]` `CanAdd(C_InventoryItem const*)`.
    pub can_add_item: usize,
    /// `[14]` `CanAdd(int type_id)`.
    pub can_add_type: usize,
    /// `[15]` `CanAddInsteadOf(item, replaced)`.
    pub can_add_instead_of_item: usize,
    /// `[16]` `AddItemAnchor(C_InventoryItemAnchor*)` — delegate на sub_140E31540.
    pub add_item_anchor: usize,
    /// `[17]` `RemoveItem(C_InventoryItem*)`.
    pub remove_item: usize,
    /// `[18]` Reserved (semantic TBD).
    pub _slot_18: usize,
    /// `[19]` `OnAddItem(C_InventoryItem*)` callback — обходит RB-tree (insert).
    pub on_add_item: usize,
    /// `[20]` `RemoveItemAnchor(C_InventoryItemAnchor*)`.
    pub remove_item_anchor: usize,
    /// `[21]` `RemoveItemAnchorByIndex(int)`.
    pub remove_item_anchor_by_index: usize,
    /// `[22]` Reserved (semantic TBD).
    pub _slot_22: usize,
    /// `[23]` Reserved — variant-specific override (semantic TBD).
    pub _slot_23: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CInventorySlotVTable>() == 24 * 8);
};

// =============================================================================
//  C_InventoryResource (10 slots)
// =============================================================================

/// VTable `C_InventoryResource`. 9 слотов.
///
/// ## Архитектура
///
/// - slot[1] — `IsAcceptableType0()`, простой getter на bit 0 в `acceptable_mask`
/// - slots[2..8] — реальные API проверки совместимости и capacity
#[repr(C)]
pub struct CInventoryResourceVTable {
    /// `[0]` `~C_InventoryResource()` — combined destructor.
    pub dtor: usize,
    /// `[1]` `IsAcceptableType0() const` — простой getter на bit 0 в acceptable_mask
    /// (`return this[+16] & 1`). НЕ Accepts метод.
    pub is_acceptable_type0: usize,
    /// `[2]` `Accepts(C_InventoryItem const*)` — bitmask check
    /// (`(1 << typeID(item)) & this[+16]`).
    pub accepts_item: usize,
    /// `[3]` `Accepts(int type_id)` — capacity-based проверка
    /// (`return capacity < 0 || capacity >= 1`).
    pub accepts_type: usize,
    /// `[4]` `HasCapacityFor(C_InventoryItem const*)`.
    pub has_capacity_for_item: usize,
    /// `[5]` `HasCapacityFor(int type_id)`.
    pub has_capacity_for_type: usize,
    /// `[6]` `HasCapacityForInsteadOf(item, replaced)`.
    pub has_capacity_for_instead_of_item: usize,
    /// `[7]` `HasCapacityForInsteadOf(type_id, replaced)`.
    pub has_capacity_for_instead_of_type: usize,
    /// `[8]` `CanChangeSize(C_InventoryItem const*, int delta)`.
    pub can_change_size: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CInventoryResourceVTable>() == 9 * 8);
};

// =============================================================================
//  C_WeaponItem (14 slots, наследник C_InventoryItem)
// =============================================================================

/// VTable `C_WeaponItem` (наследник `C_InventoryItem`). 13 слотов.
///
/// ## Архитектура
///
/// - slot[1] — MSVC adjustor thunk artifact (`return 1LL`), не реальный метод
/// - Большинство методов унаследовано от `C_InventoryItem`
/// - Weapon-specific override'ы: только slot[10] (`WriteToStream` delegate)
///   и slot[11] (`ReadFromStream` с десериализацией weapon_id/ammo)
#[repr(C)]
pub struct CWeaponItemVTable {
    /// `[0]` `~C_WeaponItem(this, char delete_flag)` — combined destructor.
    pub dtor: usize,
    /// `[1]` MSVC adjustor thunk artifact (НЕ реальный метод). `return 1`.
    pub _adjustor_thunk: usize,
    /// `[2]` `SetChanged()` / MarkDirty inherited from C_InventoryItem.
    /// Sets dirty flag at `+25` and notifies subsystem.
    pub set_changed: usize,
    /// `[3]` `ClearChanged()` inherited. Clears flag at `+25`.
    pub clear_changed: usize,
    /// `[4]` `IsChanged() const` inherited. Returns flag at `+25`.
    pub is_changed: usize,
    /// `[5]` `IsOfType(int)` inherited — default stub returns 0.
    pub is_of_type: usize,
    /// `[6]` `OnAddToInventory(C_Inventory*)` — default no-op stub.
    pub on_add_to_inventory: usize,
    /// `[7]` `OnRemoveFromInventory(C_Inventory*)` — default no-op stub.
    pub on_remove_from_inventory: usize,
    /// `[8]` `OnAddToSlot(C_InventorySlot*)` — default no-op stub.
    pub on_add_to_slot: usize,
    /// `[9]` `OnRemoveFromSlot(C_InventorySlot*)` — default no-op stub.
    pub on_remove_from_slot: usize,
    /// `[10]` `WriteToStream(C_BitStream*)` — delegate на attached object @ `+16`
    /// (вероятно WeaponData). 11 байт = jmp + ret.
    pub write_to_stream: usize,
    /// `[11]` `ReadFromStream(C_BitStream*)` override — реальная десериализация:
    /// `sub_140C4CDA0()` (setup) + `sub_140E1C1B0(this)` (read weapon_id, ammo).
    pub read_from_stream: usize,
    /// `[12]` `Dump(C_String const&)` — default empty stub.
    pub dump: usize,
}

const _: () = {
    assert!(std::mem::size_of::<CWeaponItemVTable>() == 13 * 8);
};

// =============================================================================
//  Helper: получить vtable из объекта
// =============================================================================

/// Безопасно получить ссылку на vtable из ptr на объект инвентаря.
///
/// # Safety
///
/// `obj` должен указывать на валидный объект с первым 8-байтным полем
/// vtable указателем.
#[inline]
pub unsafe fn vtable_of<'a, T>(obj: *const c_void) -> Option<&'a T> {
    if obj.is_null() {
        return None;
    }
    let vt = unsafe { *(obj as *const *const T) };
    if vt.is_null() {
        return None;
    }
    Some(unsafe { &*vt })
}
