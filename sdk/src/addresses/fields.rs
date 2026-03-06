//! Смещения полей от начала структуры (в байтах).

pub mod game_manager {
    /// `+0x180` → `Player*`
    pub const ACTIVE_PLAYER: usize = 0x180;
}

pub mod player {
    /// `+0xE8` → `Inventory*`
    ///
    /// IDA: `a1[2] + 232` где `a1[2]` = entity ptr
    /// 232 decimal = 0xE8 hex
    pub const INVENTORY: usize = 0xE8;
}

pub mod inventory {
    pub const TYPE: usize = 0x24;
    pub const SLOTS_START: usize = 0x50;
    pub const SLOTS_END: usize = 0x58;
    pub const WEAPONS: usize = 0xE8;
    
    /// `+0x170` → parent/manager ref (проверяется для HUD: type==16)
    /// IDA: `M2DE_Inventory_AddMoneyNotify` читает `[inv+0x170]`
    pub const PARENT_REF: usize = 0x170;
}

pub mod money_item {
    /// `+0x18` → MoneyCore* (M2DE_MoneyItem_GetCore возвращает [a1+0x18])
    pub const CORE: usize = 0x18;
}

pub mod money_core {
    /// `+0x00` → vtable (0x1418E5520)
    pub const VTABLE: usize = 0x00;
    /// `+0x08` → i32 limit (default 2000 = $20.00 limit?)
    pub const LIMIT: usize = 0x08;
    /// `+0x10` → MoneyContainer* → +0x10 = i64 cents
    pub const CONTAINER_PTR: usize = 0x10;
}

/// Структура слота инвентаря (например MoneySlot).
///
/// ```text
/// +0x00: vtable
/// +0x08: back-pointer на Inventory
/// +0x10: i32 (-1?)
/// +0x14: i32 (0x80?)
/// +0x18: vec_begin — начало внутреннего std::vector<ptr>
/// +0x20: vec_end   — конец вектора
/// +0x28: vec_capacity
/// ```
///
/// IDA: `M2DE_Inventory_GetMoneyPtrFromArray`:
/// ```c
/// rdx = *(slot + 0x18);  // vec_begin
/// rax = *(slot + 0x20);  // vec_end
/// ```
pub mod slot {
    /// `+0x18` → начало внутреннего вектора (std::vector begin)
    pub const VEC_BEGIN: usize = 0x18;

    /// `+0x20` → конец внутреннего вектора (std::vector end)
    pub const VEC_END: usize = 0x20;
}

/// Объект "wallet" — первый элемент вектора слота.
///
/// IDA: `M2DE_Wallet_GetInnerStruct` возвращает `[a1 + 0x18]`
pub mod wallet {
    /// `+0x18` → inner struct pointer
    ///
    /// IDA comment: "Returns [a1+24]" (24 decimal = 0x18)
    pub const INNER_STRUCT: usize = 0x18;
}

/// Inner struct (из wallet).
///
/// IDA: `M2DE_Wallet_GetMoneyPtr` делает:
/// ```asm
/// mov rax, [rcx+10h]    ; ptr1 = *(inner + 0x10)
/// mov rax, [rax+10h]    ; money = *(ptr1 + 0x10)
/// ret
/// ```
pub mod wallet_inner {
    /// `+0x10` → указатель на контейнер денег
    pub const MONEY_CONTAINER_PTR: usize = 0x10;
}

/// Контейнер денег (последний уровень).
pub mod money_container {
    /// `+0x10` → значение денег (i64, truncated to i32)
    ///
    /// Хранится в центах: $600 = 60000
    ///
    /// AOB: `48 8B 41 10 48 8B 40 10 C3`
    pub const VALUE: usize = 0x10;
}

// ═══════════════════════════════════════════════════════════════════════
//  Vehicle и другие (без изменений)
// ═══════════════════════════════════════════════════════════════════════

pub mod vehicle {
    pub const COLOR_ID: usize = 0xA4;
    pub const SPAWN_DATA: usize = 0xE0;
    pub const SPEED: usize = 0x360;
    pub const ANIM_PARAM1: usize = 0x388;
    pub const ANIM_PARAM2: usize = 0x394;
    pub const SPAWN_TIMESTAMP: usize = 0x1248;
    pub const MIN_SPAWN_TIME: usize = 0x1288;
    pub const MAX_SPAWN_TIME: usize = 0x128C;
    pub const SPAWN_PROGRESS: usize = 0x12AC;
    pub const SPAWN_SPEED_MULT: usize = 0x12CC;
}

pub mod vehicle_wrapper {
    pub const REFCOUNT: usize = 0x08;
    pub const VEHICLE: usize = 0x18;
}

pub mod c_car {
    pub const IMPORTANT_DATA: usize = 0x38;
}

pub mod car_data {
    pub const SIZE_INFO: usize = 0x270;
    pub const INIT_INFO: usize = 0x2F8;
    pub const BBOX_MIN_X: usize = 0x328;
    pub const BBOX_MIN_Y: usize = 0x330;
    pub const BBOX_MAX_X: usize = 0x334;
    pub const BBOX_MAX_Y: usize = 0x33C;
}

pub mod garage {
    pub const CURRENT_CAPACITY: usize = 0x08;
    pub const VEHICLES_BEGIN: usize = 0x10;
    pub const VEHICLES_END: usize = 0x18;
    pub const VEHICLES_CAPACITY: usize = 0x20;
    pub const CURRENT_VEHICLE_INDEX: usize = 0x60;
    pub const MAX_VEHICLES: usize = 0x64;
}

pub mod table_manager {
    pub const POLICE_OFFENCES: usize = 0x38;
    pub const WEAPONS: usize = 0x40;
    pub const ATTACK_PARAMS: usize = 0x50;
    pub const VEHICLES: usize = 0x60;
    pub const PHOBJ_SOUNDS: usize = 0xB8;
    pub const MATERIALS_PHYSICS: usize = 0xC8;
    pub const MATERIALS_SHOTS: usize = 0xD0;
    pub const MUSIC: usize = 0xD8;
    pub const GLASSBREAKING: usize = 0xE0;
    pub const GLASSMATTEMPLATES: usize = 0xE8;
    pub const HUMAN_DMGZONES: usize = 0xF8;
    pub const PINUPS_GALLERIES: usize = 0x100;
    pub const PINUPS: usize = 0x108;
    pub const RAMBO_ACTIONS: usize = 0x150;
}

pub mod callback_entry {
    pub const NAME: usize = 0x00;
    pub const EVENT_TYPE: usize = 0x20;
    pub const EVENT_ID: usize = 0x24;
    pub const CALLBACK: usize = 0x28;
    pub const CONTEXT: usize = 0x30;
    pub const SIZE: usize = 64;
}