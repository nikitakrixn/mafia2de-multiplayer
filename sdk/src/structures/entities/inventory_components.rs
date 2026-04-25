//! Низкоуровневые классы системы инвентаря: `C_Inventory`, `C_InventorySlot`,
//! `C_InventoryResource`, `C_WeaponItem`, `C_HumanThrowInventory`.
//!
//! Все layouts проверены через декомпиляцию live-ctorов и методов
//! (`AddSlot`, `AddResource`, `AddAcceptable`, `LinkToResource`).
//!
//! ## Иерархия
//!
//! ```text
//! C_Inventory (base, ~136 байт = 0x88)
//!   └─ C_HumanInventory (humanoid-расширение, 376 байт)
//!   └─ C_HumanThrowInventory (метательные предметы)
//!
//! C_InventoryItem (база предмета)
//!   └─ C_WeaponItem (оружие, 40 байт + 24 байт weapon_data на куче)
//!
//! C_InventorySlot (~80 байт base, 88 байт для weapon variants)
//! C_InventoryResource (40 байт base, 48 байт для special)
//! ```
//!
//! ## Архитектура `acceptable_mask`
//!
//! `C_InventorySlot` и `C_InventoryResource` хранят список приемлемых
//! типов предметов в виде **DWORD bitmask** (один бит = один тип),
//! не как vector. Поддерживается до 32 типов
//!
//! Это компактнее vector'а, но ограничивает количество типов до 32.

use std::ffi::c_void;

// =============================================================================
//  C_Inventory — базовый контейнер инвентаря
// =============================================================================

/// `C_Inventory` — базовый контейнер инвентаря.
///
/// Конструктор `M2DE_CInventory_Init` (`0x140DF21C0`) инициализирует:
/// - vtable `M2DE_VT_CInventory_Base` (`0x1418E3010`)
/// - sentinel-list head (alloc 48 байт)
/// - все поля до `+0x80` обнуляются
/// - `active_slot_id = -1` на `+0x18`
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable | ptr |
/// | +0x08 | items_list_sentinel (alloc 48b, sentinel-узел) | ptr |
/// | +0x10 | reserved | u64 |
/// | +0x18 | active_slot_id (default = -1) | i32 |
/// | +0x1C | reserved padding | u32 |
/// | +0x20 | reserved | u64 |
/// | +0x28 | reserved | u64 |
/// | +0x30 | reserved | u64 |
/// | +0x38 | reserved | u64 |
/// | +0x40 | reserved | u64 |
/// | +0x48 | reserved | u64 |
/// | +0x50 | `slots_begin` — vector<C_InventorySlot*> | ptr |
/// | +0x58 | `slots_end` | ptr |
/// | +0x60 | `slots_cap` | ptr |
/// | +0x68 | `resources_begin` — vector<C_InventoryResource*> | ptr |
/// | +0x70 | `resources_end` | ptr |
/// | +0x78 | `resources_cap` | ptr |
/// | +0x80 | byte flag (active?) | u8 |
///
/// **Размер base: 0x88 = 136 байт.**
#[repr(C)]
pub struct CInventory {
    /// `+0x00` vtable.
    pub vtable: *const c_void,
    /// `+0x08` Sentinel-узел linked-list предметов (heap-alloc 48b).
    pub items_list_sentinel: *mut c_void,
    /// `+0x10` Зарезервировано (init = 0).
    pub _reserved_10: u64,
    /// `+0x18` ID активного слота (-1 = нет).
    pub active_slot_id: i32,
    pub _pad_1c: u32,
    /// `+0x20..+0x50` Зарезервированные поля (init = 0). 6 u64 = 48 байт.
    pub _reserved_20_50: [u64; 6],
    /// `+0x50` `vector<C_InventorySlot*>::begin`.
    pub slots_begin: *mut *mut c_void,
    /// `+0x58` `vector<C_InventorySlot*>::end`.
    pub slots_end: *mut *mut c_void,
    /// `+0x60` `vector<C_InventorySlot*>::cap`.
    pub slots_cap: *mut *mut c_void,
    /// `+0x68` `vector<C_InventoryResource*>::begin`.
    pub resources_begin: *mut *mut c_void,
    /// `+0x70` `vector<C_InventoryResource*>::end`.
    pub resources_end: *mut *mut c_void,
    /// `+0x78` `vector<C_InventoryResource*>::cap`.
    pub resources_cap: *mut *mut c_void,
    /// `+0x80` Активность контейнера (или другой byte флаг).
    pub active_flag: u8,
    pub _pad_81: [u8; 7],
}

const _: () = {
    assert!(std::mem::size_of::<CInventory>() == 0x88);
    assert!(std::mem::offset_of!(CInventory, active_slot_id) == 0x18);
    assert!(std::mem::offset_of!(CInventory, slots_begin) == 0x50);
    assert!(std::mem::offset_of!(CInventory, resources_begin) == 0x68);
};

impl CInventory {
    /// Количество слотов в `vector<C_InventorySlot*>`.
    #[inline]
    pub fn slot_count(&self) -> usize {
        if self.slots_begin.is_null() {
            return 0;
        }
        unsafe { self.slots_end.offset_from(self.slots_begin) as usize }
    }

    /// Количество ресурсов в `vector<C_InventoryResource*>`.
    #[inline]
    pub fn resource_count(&self) -> usize {
        if self.resources_begin.is_null() {
            return 0;
        }
        unsafe { self.resources_end.offset_from(self.resources_begin) as usize }
    }
}

// =============================================================================
//  C_InventorySlot — слот для предмета
// =============================================================================

/// `C_InventorySlot` — слот в инвентаре, в который кладётся предмет.
///
/// Конструктор-инициализатор: `M2DE_CInventorySlot_Init` (`0x140DF22F0`).
///
/// ## Layout (по `Init`/`AddAcceptable`/`LinkToResource`)
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable | ptr |
/// | +0x08 | owner_inventory (back-ref на `C_Inventory`) | ptr |
/// | +0x10 | active_resource_id (default = -1) | i32 |
/// | +0x14 | **acceptable_mask** (`mask \|= (1 << type_id)`) | u32 |
/// | +0x18 | current_item (`C_InventoryItem*`) | ptr |
/// | +0x20 | reserved (init = 0) | u64 |
/// | +0x28 | reserved (init = 0) | u64 |
/// | +0x30 | resource_links_begin (vector) | ptr |
/// | +0x38 | resource_links_end | ptr |
/// | +0x40 | resource_links_cap | ptr |
/// | +0x48 | active_flag | u8 |
/// | +0x50 | extra ptr (только в weapon-variants, +0x10 байт сверху) | u64 |
///
/// **Размеры:**
/// - special/misc variants: 80 байт = `0x50`
/// - weapon variants A/B: 88 байт = `0x58` (с extra полем на +0x50)
#[repr(C)]
pub struct CInventorySlot {
    /// `+0x00` vtable.
    pub vtable: *const c_void,
    /// `+0x08` Back-ref на инвентарь-владельца.
    pub owner_inventory: *mut c_void,
    /// `+0x10` ID текущего активного ресурса (-1 = нет).
    pub active_resource_id: i32,
    /// `+0x14` Bitmask приемлемых типов предметов.
    /// Каждый bit = один тип (`mask |= 1 << type_id`). До 32 типов.
    pub acceptable_mask: u32,
    /// `+0x18` Текущий предмет в слоте.
    pub current_item: *mut c_void,
    /// `+0x20..+0x28` Зарезервированные поля.
    pub _reserved_20_28: [u64; 2],
    /// `+0x30` `vector<C_InventoryResource*>::begin` — связанные ресурсы.
    pub resource_links_begin: *mut *mut c_void,
    /// `+0x38` `vector<C_InventoryResource*>::end`.
    pub resource_links_end: *mut *mut c_void,
    /// `+0x40` `vector<C_InventoryResource*>::cap`.
    pub resource_links_cap: *mut *mut c_void,
    /// `+0x48` Флаг активности слота.
    pub active_flag: u8,
    pub _pad_49: [u8; 7],
}

const _: () = {
    assert!(std::mem::size_of::<CInventorySlot>() == 0x50);
    assert!(std::mem::offset_of!(CInventorySlot, acceptable_mask) == 0x14);
    assert!(std::mem::offset_of!(CInventorySlot, resource_links_begin) == 0x30);
};

impl CInventorySlot {
    /// Проверить принимает ли слот данный тип предмета.
    #[inline]
    pub fn accepts_type(&self, type_id: u8) -> bool {
        if type_id >= 32 {
            return false;
        }
        (self.acceptable_mask & (1 << type_id)) != 0
    }
}

// =============================================================================
//  C_InventoryResource — ресурс/категория предметов
// =============================================================================

/// `C_InventoryResource` — ресурс инвентаря (категория предметов).
///
/// Конструктор: `M2DE_CInventoryResource_Type0_Ctor` (`0x140DF2290`).
/// Аргументы: `(this, owner_inventory, capacity_units, max_capacity)`.
///
/// ## Layout
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable | ptr |
/// | +0x08 | owner_inventory | ptr |
/// | +0x10 | acceptable_mask (bitmask, как в slot) | u32 |
/// | +0x14 | max_capacity | i32 |
/// | +0x18 | capacity_units | i32 |
/// | +0x1C | padding | u32 |
/// | +0x20 | first_item_in_chain | ptr (linked-list head) |
///
/// **Размер base (Type0): 40 байт = 0x28.**
#[repr(C)]
pub struct CInventoryResource {
    /// `+0x00` vtable.
    pub vtable: *const c_void,
    /// `+0x08` Back-ref на инвентарь-владельца.
    pub owner_inventory: *mut c_void,
    /// `+0x10` Bitmask приемлемых типов (`mask |= 1 << type_id`).
    pub acceptable_mask: u32,
    /// `+0x14` Максимальная вместимость.
    pub max_capacity: i32,
    /// `+0x18` Единицы измерения вместимости (`E_InventoryCapacityUnits`).
    pub capacity_units: i32,
    pub _pad_1c: u32,
    /// `+0x20` Первый предмет в linked-list.
    pub first_item: *mut c_void,
}

const _: () = {
    assert!(std::mem::size_of::<CInventoryResource>() == 0x28);
    assert!(std::mem::offset_of!(CInventoryResource, acceptable_mask) == 0x10);
    assert!(std::mem::offset_of!(CInventoryResource, max_capacity) == 0x14);
    assert!(std::mem::offset_of!(CInventoryResource, capacity_units) == 0x18);
};

/// `C_InventoryResource` (special variant). Расширение `Type0` + byte флаг.
///
/// Конструктор: `M2DE_CInventoryResource_Special_Ctor` (`0x140D72E40`).
/// Вызывает `Type0_Ctor`, потом ставит флаг на `+0x28` и переписывает vtable.
///
/// **Размер: 48 байт = 0x30.**
#[repr(C)]
pub struct CInventoryResourceSpecial {
    pub base: CInventoryResource,
    /// `+0x28` Variant flag (`0` или `1`).
    pub variant_flag: u8,
    pub _pad_29: [u8; 7],
}

const _: () = {
    assert!(std::mem::size_of::<CInventoryResourceSpecial>() == 0x30);
};

// =============================================================================
//  C_WeaponItem — оружие в инвентаре
// =============================================================================

/// `C_WeaponItem` — предмет-оружие в инвентаре.
///
/// Конструктор: `M2DE_WeaponItem_Ctor` (`0x140DF3680`).
/// Аргументы: `(this, weapon_id, initial_ammo)`.
///
/// ## Layout (размер 40 байт = 0x28)
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable (`M2DE_VT_CWeaponItem`) | ptr |
/// | +0x08 | item-base поля | u64 |
/// | +0x10 | weapon_data | ptr (heap, alloc 24b) |
/// | +0x18 | reserved | u64 |
/// | +0x20 | item state field (init = 0) | u64 |
///
/// ## weapon_data layout (heap, 24 байт = 0x18)
///
/// | Offset | Поле | Тип |
/// |:-------|:-----|:----|
/// | +0x00 | vtable (`M2DE_VT_CWeaponItem_SecondaryMI`) | ptr |
/// | +0x08 | weapon_id | i32 |
/// | +0x0C | reserved | i32 |
/// | +0x10 | ammo (clamped to >= 0) | i32 |
/// | +0x14 | extra flag | i32 |
#[repr(C)]
pub struct CWeaponItem {
    /// `+0x00` Primary vtable (`C_WeaponItem`).
    pub vtable: *const c_void,
    /// `+0x08` `C_InventoryItem` base поля.
    pub _item_base: u64,
    /// `+0x10` Heap-allocated weapon data (содержит ID, ammo).
    pub weapon_data: *mut CWeaponData,
    /// `+0x18` Зарезервировано.
    pub _reserved_18: u64,
    /// `+0x20` Item state field (init = 0).
    pub _state_20: u64,
}

const _: () = {
    assert!(std::mem::size_of::<CWeaponItem>() == 0x28);
};

/// Heap-allocated `weapon_data` блок внутри `C_WeaponItem` (`+0x10`).
///
/// Содержит `weapon_id`, `ammo` и связанные поля. Имеет свою vtable
/// (`M2DE_VT_CWeaponItem_SecondaryMI`) для multi-inheritance взаимодействия
/// с `C_InventoryItem` базой.
///
/// **Размер: 24 байт = 0x18.**
#[repr(C)]
pub struct CWeaponData {
    /// `+0x00` Secondary MI vtable.
    pub vtable: *const c_void,
    /// `+0x08` ID оружия.
    pub weapon_id: i32,
    pub _pad_0c: i32,
    /// `+0x10` Текущее количество патронов (clamped to >= 0).
    pub ammo: i32,
    /// `+0x14` Дополнительный флаг.
    pub extra_flag: i32,
}

const _: () = {
    assert!(std::mem::size_of::<CWeaponData>() == 0x18);
};

// =============================================================================
//  C_HumanThrowInventory — инвентарь метательных предметов
// =============================================================================

/// `C_HumanThrowInventory` — наследник `C_Inventory` для метательных предметов.
///
/// Создаётся inline внутри `C_HumanInventory` блока. Содержит ровно 1 слот.
#[repr(C)]
pub struct CHumanThrowInventory {
    /// База `C_Inventory` (vtable отличается).
    pub base: CInventory,
}

const _: () = {
    assert!(std::mem::size_of::<CHumanThrowInventory>() == std::mem::size_of::<CInventory>());
};
