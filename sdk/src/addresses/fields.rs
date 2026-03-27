//! Смещения полей от начала структуры (в байтах).
//!
//! Политика файла:
//! - подтверждённые поля именуются явно
//! - спорные или недоревершенные зоны не выдаются за факт

// =============================================================================
//  GameManager
// =============================================================================

pub mod game_manager {
    /// `+0x180` -> `Player*`
    pub const ACTIVE_PLAYER: usize = 0x180;
}

// =============================================================================
//  Base Entity / Actor
// =============================================================================

pub mod entity {
    /// `+0x08` -> Расширенный указатель 1.
    /// Ненулевой у Player и C_CarVehicle. NULL у остальных.
    pub const EXT_PTR_1: usize = 0x08;
    /// `+0x10` -> Расширенный указатель 2.
    pub const EXT_PTR_2: usize = 0x10;
    /// `+0x18` -> Расширенный указатель 3.
    pub const EXT_PTR_3: usize = 0x18;

    /// `+0x20` -> state/alive flags byte.
    pub const STATE_FLAGS: usize = 0x20;

    /// `+0x24` -> packed table_id (`u32`).
    ///
    /// Формат:
    /// - low byte   = factory type
    /// - upper 24b  = instance id
    ///
    /// ВАЖНО:
    /// это НЕ отдельное поле `entity_type`.
    pub const TABLE_ID: usize = 0x24;

    /// Legacy alias: читать как `u8`, если нужен только low byte factory type.
    pub const FACTORY_TYPE_BYTE: usize = TABLE_ID;

    /// `+0x28` -> entity flags (`u32`).
    pub const ENTITY_FLAGS: usize = 0x28;

    /// `+0x30` -> FNV-1 64-bit name hash.
    pub const NAME_HASH: usize = 0x30;

    /// `+0x38` -> Parent/container reference.
    pub const PARENT_REF: usize = 0x38;

    /// `+0x40` -> RB-tree 1 root sentinel.
    pub const TREE_1_ROOT: usize = 0x40;
    /// `+0x48` -> Tree 1 entry count (Player=2, rest=0).
    pub const TREE_1_COUNT: usize = 0x48;

    /// `+0x50` -> RB-tree 2 root sentinel.
    pub const TREE_2_ROOT: usize = 0x50;

    /// `+0x78` -> frame/transform node pointer.
    pub const FRAME_NODE: usize = 0x78;

    /// `+0x80` -> owner entity pointer.
    pub const OWNER: usize = 0x80;

    /// `+0x88` -> Компонент Actor-расширение 1.
    /// NULL у чистого Actor. Ненулевой у C_Car, C_CarVehicle.
    pub const COMPONENT_88: usize = 0x88;
    pub const COMPONENT_90: usize = 0x90;
    pub const COMPONENT_98: usize = 0x98;

    /// `+0xA0` -> Entity subtype (Actor layer). Значения:
    /// - Player = 6
    /// - CarVehicle = 3
    /// - Car = varies (0x36, 0x37, 0x3A)
    /// Устанавливается после конструирования.
    pub const ENTITY_SUBTYPE: usize = 0xA0;
}

// =============================================================================
//  Human / Player
// =============================================================================

pub mod player {
    /// `+0xE8` -> `Inventory*`
    pub const INVENTORY: usize = 0xE8;

    /// `+0xF0` -> control/property-like component pointer.
    pub const CONTROL_COMPONENT: usize = 0xF0;

    /// `+0x78` -> frame/transform node pointer.
    ///
    /// Позиция:
    /// - `frame + 0x64` = X
    /// - `frame + 0x74` = Y
    /// - `frame + 0x84` = Z
    pub const FRAME_NODE: usize = super::entity::FRAME_NODE;

    /// `+0x24` -> packed table_id alias.
    ///
    /// Legacy use:
    /// - читать как `u8`, если нужен только factory type byte
    ///
    /// Примеры:
    /// - `0x10` = Player
    /// - `0x0E` = Human NPC
    /// - `0x12` = C_Car
    /// - `0x70` = C_CarVehicle
    pub const ENTITY_TYPE: usize = super::entity::TABLE_ID;

    /// Явный alias для packed table_id.
    pub const TABLE_ID: usize = super::entity::TABLE_ID;

    /// `+0x28` -> entity flags.
    pub const ENTITY_FLAGS: usize = super::entity::ENTITY_FLAGS;

    /// `+0x80` -> owner entity pointer.
    ///
    /// Пешком = NULL, в машине = pointer на vehicle entity.
    pub const OWNER: usize = super::entity::OWNER;

    /// `+0xA0` -> Entity subtype (=6 для Player).
    pub const ENTITY_SUBTYPE: usize = super::entity::ENTITY_SUBTYPE;

    /// `+0xA8` -> AI params pointer.
    pub const AI_PARAMS: usize = 0xA8;

    /// `+0xB0` -> Отдельный объект (НЕ из компонентного блока 2648 байт).
    /// Устанавливается после конструирования. Heap-указатель в другом регионе.
    pub const EXTERNAL_COMPONENT_B0: usize = 0xB0;

    /// `+0xB8` -> Неизвестный компонент (из компонентного блока).
    pub const COMPONENT_B8: usize = 0xB8;

    /// `+0xC0` -> AI/navigation component.
    pub const AI_NAV_COMPONENT: usize = 0xC0;

    /// `+0xC8` -> Компонент (из блока).
    pub const COMPONENT_C8: usize = 0xC8;

    /// `+0xD0` -> TransformSync component.
    pub const TRANSFORM_SYNC: usize = 0xD0;

    /// `+0xD8` -> Optional component (может быть NULL).
    pub const OPT_COMPONENT: usize = 0xD8;

    /// `+0xE0` -> Компонент (из блока).
    pub const COMPONENT_E0: usize = 0xE0;

    /// `+0xF8` -> Behavior component.
    pub const BEHAVIOR_COMPONENT: usize = 0xF8;

    /// `+0x100` -> Блок компонентов.
    pub const COMPONENT_BLOCK_100: usize = 0x100;

    /// `+0x108` -> weapon state component.
    pub const WEAPON_STATE_COMPONENT: usize = 0x108;

    /// `+0x110` -> Компонент.
    pub const COMPONENT_110: usize = 0x110;

    /// `+0x118` -> Компонент.
    pub const COMPONENT_118: usize = 0x118;

    /// `+0x120` -> Последний компонент из блока.
    pub const COMPONENT_120: usize = 0x120;

    /// `+0x148` -> current health (`f32`).
    pub const CURRENT_HEALTH: usize = 0x148;

    /// `+0x14C` -> NPC healthmax or type-related multiplier field.
    pub const NPC_HEALTHMAX: usize = 0x14C;

    /// `+0x150` -> damage multiplier from NPC/non-player sources.
    pub const NONPLAYER_DAMAGE_MULT: usize = 0x150;

    /// `+0x154` -> distance threshold for damage falloff.
    pub const NONPLAYER_DAMAGE_DIST: usize = 0x154;

    /// `+0x160` -> invulnerability flag.
    pub const INVULNERABILITY: usize = 0x160;

    /// `+0x161` -> is_dead flag.
    pub const IS_DEAD: usize = 0x161;

    /// `+0x162` -> demigod flag.
    pub const DEMIGOD: usize = 0x162;

    /// `+0x163` -> Неизвестный флаг (=1 у живого Player).
    /// Возможно alive/spawned/has_controller.
    pub const UNKNOWN_FLAG_163: usize = 0x163;

    /// `+0x180` -> pointer to body damage multipliers array.
    pub const BODY_DAMAGE_MULTIPLIERS: usize = 0x180;

    /// `+0x188` -> Количество body damage zones (=12).
    pub const BODY_ZONE_COUNT: usize = 0x188;

    /// `+0x190` -> self-reference (`this`).
    pub const SELF_REF: usize = 0x190;

    /// `+0x198` -> Неизвестное значение (=0x400=1024 у Player).
    pub const UNKNOWN_198: usize = 0x198;

    /// `+0x1C0` -> Smart pointer slots начало (8 слотов по 16 байт).
    /// Формат слота: { ptr(8), id(u32), state(u32) }
    pub const SMART_PTR_SLOTS: usize = 0x1C0;

    /// `+0x258` -> physics provider pointer.
    pub const PHYSICS_PROVIDER: usize = 0x258;

    /// `+0x338` -> death position (`Vec3`) [provisional usage].
    pub const DEATH_POSITION: usize = 0x338;

    /// `+0x344` -> death type / death mode (`i32`) [provisional].
    pub const DEATH_TYPE: usize = 0x344;

    /// `+0x3D8` -> player state / flags dword.
    ///
    /// Low byte участвует в predicate:
    /// - значение != 3
    /// - значение != 4
    ///
    /// Также higher bits читаются как flags, например `0x40000`.
    pub const STATE_FLAGS_3D8: usize = 0x3D8;

    /// `+0x428` -> pointer returned by slot [101].
    pub const FIELD_428: usize = 0x428;

    /// `+0x430` -> state code dword.
    ///
    /// Подтверждённый getter:
    /// `M2DE_CPlayer_IsStateCode430_Equal10` возвращает `*(u32*)(this+0x430) == 10`.
    pub const STATE_CODE_430: usize = 0x430;

    /// `+0x438` -> state mask/profile dword.
    ///
    /// Загружается из строки через `M2DE_CPlayer_LoadStateMask438_ByName`.
    pub const STATE_MASK_438: usize = 0x438;

    /// `+0x43C` -> auxiliary player state field.
    pub const FIELD_43C: usize = 0x43C;

    /// `+0x440` -> auxiliary state/config dword.
    pub const FIELD_440: usize = 0x440;

    /// `+0x448` -> auxiliary player state field.
    pub const FIELD_448: usize = 0x448;
    ///
    /// Подтверждённый getter:
    /// `M2DE_CPlayer_IsField464_Equal1` возвращает `*(u32*)(this+0x464) == 1`.
    pub const FIELD_464: usize = 0x464;

    /// `+0x490` -> player state flags / bitfield.
    ///
    /// Подтверждено:
    /// - bits [1..3]   -> mask set/clear
    /// - bits [4..6]   -> 3-bit field
    /// - bits [7..13]  -> 7-bit mask/field
    /// - bit  [14]     -> bool
    /// - bit  [15]     -> bool
    ///
    /// См.:
    /// - `M2DE_CPlayer_StateFlags490_SetClearMaskedBits1_3`
    /// - `M2DE_CPlayer_StateFlags490_SetFieldBits4_6`
    /// - `M2DE_CPlayer_StateFlags490_SetClearMaskedBits7_13`
    /// - `M2DE_CPlayer_StateFlags490_SetBit14`
    /// - `M2DE_CPlayer_StateFlags490_SetBit15`
    pub const STATE_FLAGS_490: usize = 0x490;

    /// `+0x45C` -> player special-state subobject.
    /// HasSpecialState46F0 delegates into this object.
    pub const SUBOBJECT_45C: usize = 0x45C;

    /// `+0x45C` -> sub45c.code_a (primary stored action/code).
    pub const SUBOBJECT_45C_CODE_A: usize = 0x45C;

    /// `+0x460` -> sub45c.code_b (auxiliary stored action/code).
    pub const SUBOBJECT_45C_CODE_B: usize = 0x460;

    /// `+0x464` -> sub45c.state.
    /// NOT an independent field — part of the subobject at +0x45C.
    ///
    /// This is sub45c + 0x08.
    pub const SUBOBJECT_45C_STATE: usize = 0x464;

    /// `+0x4E0` -> dword reset to -1 in player reset/init path.
    pub const FIELD_4E0: usize = 0x4E0;

    /// `+0x4E4` -> flags dword, low bits cleared in reset/init path.
    pub const FLAGS_4E4: usize = 0x4E4;

    /// `+0x4F8` -> qword reset to 0 in player reset/init path.
    pub const FIELD_4F8: usize = 0x4F8;

    /// `+0x500` -> world/entity handle used by ClearSpecialState45C.
    pub const WORLD_HANDLE_500: usize = 0x500;

    /// `+0x508` -> owned helper/resource ptr. Released in teardown path.
    pub const OWNED_HELPER_508: usize = 0x508;

    /// `+0x510` -> player state flags dword.
    ///
    /// В Character_Update используется bit 4 / bit 3-like логика,
    /// зависящая от результата `sub_1400C46F0`.
    pub const STATE_FLAGS_510: usize = 0x510;

    /// `+0x518` -> string-related pointer.
    pub const STRING_PTR_518: usize = 0x518;

    /// `+0x520` -> string object / buffer reset by M2DE_String_SetCStr.
    pub const STRING_OBJ_520: usize = 0x520;

    /// `+0x520` -> qword field (alias for STRING_OBJ_520).
    pub const FIELD_520_QWORD: usize = 0x520;
}

// =============================================================================
//  Player Sub45C subobject
// =============================================================================

pub mod player_sub45c {
    /// `+0x00` -> primary stored action/code.
    pub const CODE_A: usize = 0x00;

    /// `+0x04` -> auxiliary stored action/code.
    pub const CODE_B: usize = 0x04;

    /// `+0x08` -> sub-state.
    ///
    /// Observed:
    /// - 0 = idle
    /// - 1 = pending
    /// - 2 = active A
    /// - 3 = active B
    /// - 4 = deferred
    pub const STATE: usize = 0x08;

    pub const SIZE: usize = 0x0C;
}

// =============================================================================
//  Inventory / weapon / money
// =============================================================================

pub mod inventory {
    /// `+0x08` -> root of RB-tree / map lookup by weapon id.
    pub const WEAPON_TREE: usize = 0x08;

    /// `+0x24` -> inventory type byte.
    pub const TYPE: usize = 0x24;

    /// `+0x50` -> slots begin.
    pub const SLOTS_START: usize = 0x50;

    /// `+0x58` -> slots end.
    pub const SLOTS_END: usize = 0x58;

    /// `+0x168` -> unlimited ammo flag.
    pub const UNLIMITED_AMMO: usize = 0x168;

    /// `+0x170` -> owner entity ref.
    pub const OWNER_ENTITY_REF: usize = 0x170;
}

pub mod money_item {
    /// `+0x18` -> MoneyCore*
    pub const CORE: usize = 0x18;
}

pub mod money_core {
    /// `+0x00` -> vtable (0x1418E5520)
    pub const VTABLE: usize = 0x00;
    /// `+0x08` -> limit (i32).
    pub const LIMIT: usize = 0x08;
    /// `+0x10` -> MoneyContainer* -> +0x10 = i64 cents
    pub const CONTAINER_PTR: usize = 0x10;
}

/// Индексы слотов инвентаря.
pub mod slots {
    /// `+0x00` -> current weapon slot.
    pub const CURRENT_WEAPON: usize = 0;
    /// `+0x01` -> unknown slot.
    pub const UNKNOWN_1: usize = 1;
    /// `+0x02` -> weapon slot 1.
    pub const WEAPON_1: usize = 2;
    /// `+0x03` -> weapon slot 2.
    pub const WEAPON_2: usize = 3;
    /// `+0x04` -> ammo slot.
    pub const AMMO: usize = 4;
    /// `+0x05` -> money slot.
    pub const MONEY: usize = 5;
}

/// Структура одного слота инвентаря.
pub mod slot {
    /// `+0x18` -> begin of vector.
    pub const VEC_BEGIN: usize = 0x18;
    /// `+0x20` -> конец std::vector
    pub const VEC_END: usize = 0x20;
    /// `+0x50` -> указатель на weapon table entry
    pub const TABLE_ENTRY: usize = 0x50;
}

/// Запись из /tables/weapons.tbl.
pub mod weapon_table_entry {
    /// `+0x24` -> flags.
    pub const FLAGS: usize = 0x24;
    /// `+0x58` -> int32 максимальная ёмкость обоймы
    pub const MAX_AMMO: usize = 0x58;
}

/// Данные оружия (из RB-дерева или weapon_state компонента).
pub mod weapon_data {
    /// `+0x00` -> int32 ID оружия
    pub const WEAPON_ID: usize = 0x00;
    /// `+0x10` -> ptr -> container -> +0x10 -> int32 текущие патроны
    pub const AMMO_CONTAINER: usize = 0x10;
    /// `+0x24` -> flags
    pub const WEAPON_FLAGS: usize = 0x24;
}

/// Битовые флаги типа оружия (weapon_data+0x24).
pub mod weapon_type_flags {
    /// Холодное оружие (нож, бита, кулаки)
    pub const COLD_WEAPON: u32 = 0x04;
    /// Огнестрельное оружие (пистолет, автомат, дробовик, винтовка)
    pub const FIRE_WEAPON: u32 = 0x20;
    /// Метательное оружие (граната, молотов)
    pub const THROWING_WEAPON: u32 = 0x200000;
}

/// Weapon State Component (player+0x108).
pub mod weapon_state {
    /// `+0x2B0` -> ptr на WeaponData текущего оружия в руках
    /// NULL = руки пусты.
    pub const CURRENT_WEAPON_DATA: usize = 0x2B0;
}

/// Объект "wallet" — первый элемент вектора слота.
///
/// IDA: `M2DE_Wallet_GetInnerStruct` возвращает `[a1 + 0x18]`
pub mod wallet {
    /// `+0x18` -> inner struct pointer
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
    /// `+0x10` -> указатель на контейнер денег
    pub const MONEY_CONTAINER_PTR: usize = 0x10;
}

/// Контейнер денег (последний уровень).
pub mod money_container {
    /// `+0x10` -> значение денег (i64, truncated to i32)
    ///
    /// Хранится в центах: $600 = 60000
    ///
    /// AOB: `48 8B 41 10 48 8B 40 10 C3`
    pub const VALUE: usize = 0x10;
}

pub mod hud_manager {
    /// `+0x98` -> Money display component (HudMoneyDisplay*)
    pub const MONEY_DISPLAY: usize = 0x98;
}

pub mod hud_money_display {
    /// `+0x48` -> displayed/animated value (i64)
    pub const ANIMATED_VALUE: usize = 0x48;
    /// `+0x50` -> actual target value (i64)
    pub const TARGET_VALUE: usize = 0x50;
    /// `+0x5C` -> animation timer (f32, popup shows only when <= 0)
    pub const ANIM_TIMER: usize = 0x5C;
    /// `+0x78` -> popup enabled flag (bool)
    pub const POPUP_ENABLED: usize = 0x78;
}

// =============================================================================
//  Frame / transform
// =============================================================================

pub mod entity_frame {
    // Позиция
    pub const POS_X: usize = 0x64;
    pub const POS_Y: usize = 0x74;
    pub const POS_Z: usize = 0x84;

    // Right вектор (Col0) — направление вправо от персонажа
    pub const RIGHT_X: usize = 0x58;
    pub const RIGHT_Y: usize = 0x68;
    pub const RIGHT_Z: usize = 0x78;

    // Forward вектор (Col1) — куда смотрит персонаж (в XY плоскости)
    pub const FORWARD_X: usize = 0x5C;
    pub const FORWARD_Y: usize = 0x6C;
    pub const FORWARD_Z: usize = 0x7C;

    // Up вектор (Col2) — мировой "вверх" (обычно 0,0,1)
    pub const UP_X: usize = 0x60;
    pub const UP_Y: usize = 0x70;
    pub const UP_Z: usize = 0x80;
}

// =============================================================================
//  Vehicle / C_Car / C_CarVehicle
// =============================================================================

pub mod vehicle {
    // --- inherited entity/actor part ---

    /// `+0x00` -> primary vtable.
    ///
    /// Runtime:
    /// - `0x141850030` = `C_Car`
    /// - `0x1418EAAC8` = `C_CarVehicle`
    pub const VTABLE: usize = 0x00;

    /// `+0x24` -> packed table_id.
    ///
    /// Low byte:
    /// - `0x12` = parked/static `C_Car`
    /// - `0x70` = drivable/active `C_CarVehicle`
    pub const TABLE_ID: usize = 0x24;

    /// Legacy alias: читать low byte как u8.
    pub const ENTITY_TYPE: usize = TABLE_ID;

    /// `+0x28` -> entity flags.
    pub const ENTITY_FLAGS: usize = 0x28;

    /// `+0x38` -> raw entity-head / class-specific region.
    ///
    /// ⚠️ Раньше здесь фигурировала гипотеза про GUID.
    /// Сейчас GUID по `+0x38` НЕ подтверждён.
    pub const RAW_38: usize = 0x38;

    /// `+0x78` -> frame pointer.
    pub const FRAME: usize = 0x78;

    /// `+0x80` -> owner pointer.
    pub const OWNER: usize = 0x80;

    /// `+0x258` -> physics provider pointer.
    pub const PHYSICS_PROVIDER: usize = 0x258;

    // --- nested / class-specific zones ---
    // ВАЖНО: эти смещения исторически собраны для `C_Car` path и часть из них
    // ещё требует дополнительной проверки на всех runtime сценариях.

    /// `+0x0E0` -> nested/subobject region [provisional].
    pub const SUBOBJ_0E0: usize = 0x0E0;

    /// `+0x1E0` -> nested/subobject region [provisional].
    pub const SUBOBJ_1E0: usize = 0x1E0;

    /// `+0x1E8` -> nested/subobject region [provisional].
    pub const SUBOBJ_1E8: usize = 0x1E8;

    /// `+0x1F8` -> nested/subobject region [provisional].
    pub const SUBOBJ_1F8: usize = 0x1F8;

    /// `+0x210` -> nested/subobject region [provisional].
    pub const SUBOBJ_210: usize = 0x210;

    // --- color-related ---
    pub const COLOR_FLAGS: usize = 0x944;
    pub const COLOR1_RGB: usize = 0x954;
    pub const COLOR1_ALPHA: usize = 0x95C;
    pub const COLOR2_RGB: usize = 0x960;
    pub const COLOR2_ALPHA: usize = 0x968;

    // --- spawn/time-related ---
    pub const SPAWN_TIMESTAMP: usize = 0x1248;

    // --- old/provisional offsets ---
    pub const COLOR_ID: usize = 0xA4; // provisional
    pub const SPAWN_DATA: usize = 0xE0; // provisional
    pub const SPEED: usize = 0x360; // provisional
    pub const ANIM_PARAM1: usize = 0x388; // provisional
    pub const ANIM_PARAM2: usize = 0x394; // provisional
    pub const MIN_SPAWN_TIME: usize = 0x1288; // provisional
    pub const MAX_SPAWN_TIME: usize = 0x128C; // provisional
    pub const SPAWN_PROGRESS: usize = 0x12AC; // provisional
    pub const SPAWN_SPEED_MULT: usize = 0x12CC; // provisional
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

// =============================================================================
//  Garage
// =============================================================================

pub mod garage {
    pub const CURRENT_CAPACITY: usize = 0x08;
    pub const VEHICLES_BEGIN: usize = 0x10;
    pub const VEHICLES_END: usize = 0x18;
    pub const VEHICLES_CAPACITY: usize = 0x20;
    pub const CURRENT_VEHICLE_INDEX: usize = 0x60;
    pub const MAX_VEHICLES: usize = 0x64;
}

// =============================================================================
//  Tables / callbacks
// =============================================================================

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

// =============================================================================
//  RenderDevice / swapchain
// =============================================================================

pub mod render_device {
    pub const INIT_CONFIG: usize = 0x2008;
    pub const RENDER_WIDTH: usize = 0x2020;
    pub const RENDER_HEIGHT: usize = 0x2024;
    pub const WINDOW_CONFIG_PTR: usize = 0x2028;
    pub const SUPPORTS_FL_10_0: usize = 0x2032;
    pub const SUPPORTS_FL_10_0_DUP: usize = 0x2033;
    pub const SUPPORTS_FL_10_1: usize = 0x2034;
    pub const DX_INITIALIZED: usize = 0x2035;
    pub const ADAPTER_OUTPUT_COUNT: usize = 0x203C;
    pub const MAX_TEXTURE_SIZE: usize = 0x2040;
    pub const ANISO_FILTER_SETTING: usize = 0x2044;
    pub const SHADER_CACHE: usize = 0x2050;
    pub const DYNAMIC_VB: usize = 0x2070;

    /// `+0x2078` -> dynamic index buffer resource.
    ///
    /// Строка: `"RenderDeviceBase::DynamicIB"`
    pub const DYNAMIC_IB: usize = 0x2078;

    /// `+0x21A8` -> текущий режим/профиль рендера.
    ///
    /// Используется в switch внутри init.
    pub const CURRENT_STATE_MODE: usize = 0x21A8;

    /// `+0x21AC` -> вторичный state mode.
    pub const CURRENT_STATE_MODE_B: usize = 0x21AC;

    /// `+0x2780` -> `IDXGIFactory1*`
    pub const DXGI_FACTORY: usize = 0x2780;

    /// `+0x2788` -> `D3D_FEATURE_LEVEL`
    pub const FEATURE_LEVEL: usize = 0x2788;

    /// `+0x2790` -> `ID3D11Device*`
    pub const D3D_DEVICE: usize = 0x2790;

    /// `+0x2798` -> `ID3D11DeviceContext*`
    pub const D3D_CONTEXT: usize = 0x2798;

    /// `+0x27A0` -> `M2DE_SwapChainManager*`
    pub const SWAPCHAIN_MANAGER: usize = 0x27A0;

    /// `+0x27A8` -> `M2DE_SwapChainWrapper*`
    ///
    /// Это основной путь к текущему DXGI swapchain.
    pub const CURRENT_SWAPCHAIN: usize = 0x27A8;

    /// `+0x27B0` -> дополнительный указатель на активный swapchain wrapper.
    pub const ACTIVE_SWAPCHAIN: usize = 0x27B0;

    /// `+0x510C` -> флаги адаптера / init flags.
    ///
    /// Влияют на выбор режима swapchain.
    pub const ADAPTER_FLAGS: usize = 0x510C;

    /// `+0x5110` -> `ID3DUserDefinedAnnotation*` или NULL.
    pub const DEBUG_ANNOTATION: usize = 0x5110;
}

pub mod swapchain_manager {
    pub const TREE_ROOT: usize = 0x00;

    /// `+0x08` -> количество элементов в дереве.
    pub const TREE_SIZE: usize = 0x08;

    /// `+0x10` -> `IDXGIFactory4*`
    pub const FACTORY: usize = 0x10;
    pub const DEVICE: usize = 0x18;
    pub const CONTEXT: usize = 0x20;
    pub const TEARING_SUPPORTED: usize = 0x28;
    pub const DEBUG_MODE: usize = 0x29;
}

pub mod swapchain_wrapper {
    /// `+0x00` -> ширина swapchain.
    pub const WIDTH: usize = 0x00;
    pub const HEIGHT: usize = 0x04;
    pub const SWAPCHAIN_MODE: usize = 0x08;
    pub const HWND: usize = 0x10;
    pub const SWAPCHAIN: usize = 0x18;
    pub const BACK_BUFFER: usize = 0x20;
    pub const DEPTH_TEXTURE: usize = 0x28;
    pub const DSV: usize = 0x30;
    pub const RTV: usize = 0x38;
    pub const SRV: usize = 0x40;
}

// =============================================================================
//  Script machine / Lua
// =============================================================================

pub mod script_machine_manager {
    /// +0x08 -> указатель на std::vector<ScriptMachine*>
    pub const VECTOR: usize = 0x08;
}

/// Внутренний std::vector (begin/end/capacity).
///
/// Используется для vector'а script machines и других
/// std::vector в структурах движка.
pub mod std_vector {
    /// +0x00 -> первый элемент
    pub const BEGIN: usize = 0x00;
    pub const END: usize = 0x08;
    pub const CAPACITY: usize = 0x10;
}

pub mod script_machine {
    pub const LUA_STATE: usize = 0x70;
}

// =============================================================================
//  Camera system
// =============================================================================

pub mod camera_manager {
    pub const INTERIER_VIEW: usize = 0x0000;
    pub const EXTERIER_VIEW: usize = 0x0D18;

    pub const INTERIER_DEFAULT_FOV: usize = 0x0C80;
    pub const EXTERIER_DEFAULT_FOV: usize = 0x1998;

    pub const TRANSIT_SPEED_EXTERIER: usize = 0x1A30;
    pub const TRANSIT_SPEED_INTERIER: usize = 0x1A34;
    pub const PLAYER_CAMERA_LOADED: usize = 0x1FB4;

    pub const CAR_BUMPER_FOV: usize = 0x21D8;
    pub const CAR_WHEEL_FOV: usize = 0x21E8;
    pub const CAR_HOOD_FOV: usize = 0x21F8;
    pub const CAR_LOOKBACK_FOV: usize = 0x230C;

    pub const CAR_DYNAMIC_PARAMS: usize = 0x2244;
    pub const CAR_DYNAMIC_PARAM_COUNT: usize = 25;
    pub const CAR_DYNAMIC_FOV: usize = 0x2244 + 11 * 4;
    pub const CAR_DYNAMIC_FOV_MAX: usize = 0x2244 + 16 * 4;

    pub const CAR_DYNAMIC_LONG_PARAMS: usize = 0x22A8;
    pub const CAR_DYNAMIC_LONG_PARAM_COUNT: usize = 25;
    pub const CAR_DYNAMIC_LONG_FOV: usize = 0x22A8 + 11 * 4;
    pub const CAR_DYNAMIC_LONG_FOV_MAX: usize = 0x22A8 + 16 * 4;

    pub const CAR_SHOOT_PARAMS: usize = 0x2208;
    pub const CAR_SHOOT_PARAM_COUNT: usize = 15;
    pub const CAR_SHOOT_FOV: usize = 0x2208 + 3 * 4;

    pub const CAR_GAMEPAD_PARAMS: usize = 0x231C;
    pub const CAR_GAMEPAD_PARAM_COUNT: usize = 24;
    pub const CAR_GAMEPAD_FOV: usize = 0x231C + 10 * 4;
    pub const CAR_GAMEPAD_FOV_MAX: usize = 0x231C + 14 * 4;

    pub const FPV_FOV: usize = 0x275C;
    pub const DEATH_FOV: usize = 0x277C;
    pub const MEELEE_FOV: usize = 0x2784 + 5 * 4;
}

pub mod camera_view {
    pub const SIZE: usize = 0xD18;
    pub const NUM_STATES: usize = 15;
    pub const NUM_PARAMS: usize = 27;
    pub const NUM_SPEEDS: usize = 15;

    pub const STATES_BASE: usize = 0x04;
    pub const STATE_STRIDE: usize = 0xD4;
    pub const STATE_PARAMS_OFFSET: usize = 0x00;
    pub const STATE_SPEEDS_OFFSET: usize = 0x6C;
    pub const STATE_PARAM_FLAGS_OFFSET: usize = 0xA8;
    pub const STATE_SPEED_FLAGS_OFFSET: usize = 0xC3;

    pub const DEFAULT_PARAMS: usize = 0xC70;
    pub const DEFAULT_SPEEDS: usize = 0xCDC;
}

// =============================================================================
//  AI / DB / wrappers / caches
// =============================================================================

pub mod ai_params {
    pub const AGGRESSIVITY: usize = 0x04;
}

pub mod db_record {
    pub const TABLE_ID: usize = 0x24;
    pub const FLAGS: usize = 0x28;
    pub const NAME_HASH: usize = 0x30;
}

/// Script Wrapper layout (общий для всех типов).
pub mod script_wrapper {
    pub const VTABLE: usize = 0x00;
    pub const REFCOUNT: usize = 0x08;
    pub const NATIVE: usize = 0x10;
    pub const OBSERVER: usize = 0x18;
}

pub mod sds_manager {
    pub const VTABLE: usize = 0x00;
    pub const VTABLE2: usize = 0x08;
    pub const CURRENT_LOAD_INDEX: usize = 0x10;
    pub const LOADING_FLAG: usize = 0x30;
    pub const FIELD_44: usize = 0x44;
    pub const FIELD_68: usize = 0x68;
    pub const LOADED_SLOTS_BEGIN: usize = 0x70;
    pub const LOADED_SLOTS_END: usize = 0x78;
    pub const VTABLE3: usize = 0x88;
    pub const MODULE_OBJECT: usize = 0x90;
}

pub mod sds_line_cache {
    pub const CACHE_BEGIN: usize = 0x18;
    pub const CACHE_END: usize = 0x20;
    pub const ENTRY_SIZE: usize = 24;
    pub const ENTRY_HASH: usize = 0x00;
    pub const ENTRY_INDEX: usize = 0x14;
}

pub mod entity_cache {
    pub const HASH_CACHE_BEGIN: usize = 0x08;
    pub const HASH_CACHE_END: usize = 0x10;
    pub const TABLE_ID_CACHE_BEGIN: usize = 0x28;
    pub const TABLE_ID_CACHE_END: usize = 0x30;
    pub const ENTRY_SIZE: usize = 16;
    pub const ENTRY_WRAPPER: usize = 0x08;
}

pub mod value_container {
    pub const STORE_PTR: usize = 0x10;
}

pub mod value_store {
    pub const VALUE: usize = 0x10;
}

pub mod rb_tree_node {
    pub const LEFT: usize = 0x00;
    pub const RIGHT: usize = 0x10;
    pub const IS_SENTINEL: usize = 0x19;
    /// +0x20 -> key (u32 type_id).
    pub const KEY: usize = 0x20;
    /// +0x28 -> value (ptr).
    pub const VALUE: usize = 0x28;
}

// =============================================================================
//  ScriptEntity family
// =============================================================================

pub mod script_entity {
    /// `+0x78` -> script entry id / slot index.
    ///
    /// В direct police child path используется как индекс в `scripts[this+0x78]`.
    pub const SCRIPT_ENTRY_ID: usize = 0x78;

    /// `+0x7C` -> script context index / selector.
    ///
    /// Низкий байт используется для получения Lua context/manager:
    /// `movzx ecx, byte ptr [this+7Ch]`.
    pub const SCRIPT_CONTEXT_INDEX: usize = 0x7C;

    /// `+0x80` -> auxiliary code/state field.
    ///
    /// В add/init path участвует как дополнительный аргумент/state.
    pub const AUX_CODE_OR_STATE: usize = 0x80;

    /// `+0x88` -> provider/list-like pointer.
    ///
    /// В direct police child path используется как источник списка script entries.
    pub const SCRIPT_PROVIDER_OR_LIST: usize = 0x88;
}

// =============================================================================
//  Police-script owner singleton
// =============================================================================

pub mod police_script_owner {
    /// `+0x00` -> root/sentinel pointer.
    pub const ROOT_OR_SENTINEL: usize = 0x00;

    /// `+0x08` -> count/state-like field.
    pub const COUNT_OR_STATE: usize = 0x08;

    /// `+0x10` -> active child pointer.
    pub const ACTIVE_CHILD: usize = 0x10;
}

pub mod police_script_owner_node {
    pub const LINK_00: usize = 0x00;
    pub const LINK_08: usize = 0x08;
    pub const CHILD_OR_LINK_10: usize = 0x10;
    pub const FLAGS_18: usize = 0x18;
    pub const SENTINEL_BYTE_19: usize = 0x19;
    pub const SIZE: usize = 0x30;
}

pub mod vehicle_common {
    /// `+0x00` -> primary vtable.
    pub const VTABLE: usize = 0x00;

    /// `+0x24` -> packed table_id.
    pub const TABLE_ID: usize = 0x24;

    /// Legacy alias: читать low byte как u8.
    pub const ENTITY_TYPE: usize = TABLE_ID;

    /// `+0x28` -> entity flags.
    pub const ENTITY_FLAGS: usize = 0x28;

    /// `+0x78` -> frame pointer.
    pub const FRAME: usize = 0x78;

    /// `+0x80` -> owner pointer.
    pub const OWNER: usize = 0x80;

    /// `+0x258` -> physics provider pointer.
    pub const PHYSICS_PROVIDER: usize = 0x258;
}

pub mod car {
    /// Размер аллокации C_Car.
    pub const SIZE: usize = 0x1258;

    /// Embedded sub-vtables (множественное наследование).
    pub const SUB_VTABLE_1: usize = 0xE0; // 0x141850298
    pub const SUB_VTABLE_2: usize = 0x1E0; // 0x141850478
    pub const SUB_VTABLE_3: usize = 0x1E8; // 0x1418504C0
    pub const SUB_VTABLE_4: usize = 0x1F8; // 0x1418504E0
    pub const SUB_VTABLE_5: usize = 0x210; // 0x1418504F0

    /// Vector-like component storage.
    pub const VEC_BEGIN: usize = 0xB0;
    pub const VEC_END: usize = 0xB8;
    pub const VEC_CAPACITY: usize = 0xC0;

    /// Self reference (= this). Подтверждено 3 образцами runtime.
    pub const SELF_REF: usize = 0x2F0;

    /// Color data.
    pub const COLOR_CONFIG: usize = 0x940;
    pub const COLOR_FLAGS: usize = 0x944;
    pub const COLOR1_RGB: usize = 0x954;
    pub const COLOR1_ALPHA: usize = 0x95C;
    pub const COLOR1_FLOATS: usize = 0x948;
    pub const COLOR2_RGB: usize = 0x960;
    pub const COLOR2_ALPHA: usize = 0x968;
    pub const COLOR2_FLOATS: usize = 0x950;

    /// Car data block.
    pub const DATA_BLOCK_START: usize = 0xEE0;
    pub const FLOAT_1_0_F54: usize = 0xF54;
    pub const HELPER_OBJ_PTR: usize = 0x11A8;

    /// Spawn data.
    pub const SPAWN_TIMESTAMP: usize = 0x1248;
    pub const SPAWN_TICK: usize = 0x1258;

    // Provisional legacy fields
    pub const SPEED: usize = 0x360;
    pub const ANIM_PARAM1: usize = 0x388;
    pub const ANIM_PARAM2: usize = 0x394;
}

pub mod car_vehicle {
    /// Размер аллокации C_CarVehicle.
    pub const SIZE: usize = 0x2F0;

    /// Multiple inheritance sub-vtables.
    pub const SUB_VTABLE_1: usize = 0xA8;
    pub const SUB_VTABLE_2: usize = 0xB0;
    pub const SUB_VTABLE_3: usize = 0xB8;
    pub const SUB_VTABLE_4: usize = 0xC0;

    /// Указатель на physics/dynamics.
    pub const PHYSICS_PTR: usize = 0xC8;

    /// 6 transform слотов по 12 байт (Vec3).
    pub const TRANSFORM_SLOTS_BASE: usize = 0xD0;
    pub const TRANSFORM_SLOT_SIZE: usize = 12;
    pub const TRANSFORM_SLOT_COUNT: usize = 6;

    /// 3 inline string слота по 0x20 байт: { u8 flag, char[31] name }.
    pub const CLOTH_SLOT: usize = 0x118;
    pub const BODY_SLOT: usize = 0x138;
    pub const LOOK_SLOT: usize = 0x158;
    pub const NAME_SLOT_SIZE: usize = 0x20;

    /// Дополнительные transform данные.
    pub const TRANSFORM_7: usize = 0x184;
    pub const TRANSFORM_8: usize = 0x190;

    /// Ссылочные указатели.
    pub const REF_PTR_1A0: usize = 0x1A0;
    pub const REF_PTR_1A8: usize = 0x1A8;

    /// Sentinel / control.
    pub const SENTINEL_1C4: usize = 0x1C4;

    /// Container58 (RB-tree + vectors).
    pub const CONTAINER: usize = 0x1D0;
    pub const CONTAINER_1D0: usize = 0x1D0;
    pub const CONTAINER_1D0_ROOT: usize = 0x1D0 + 0x40;
    pub const CONTAINER_1D0_STATE_A: usize = 0x1D0 + 0x48;
    pub const CONTAINER_1D0_STATE_B: usize = 0x1D0 + 0x50;

    /// Масштаб / конфигурация.
    pub const SCALE_FACTOR: usize = 0x234;

    /// Steering scale factors (= 1.0).
    pub const STEER_SCALE_1: usize = 0x25C;
    pub const STEER_SCALE_2: usize = 0x26C;

    /// Smart pointer (refcounted).
    pub const SMART_PTR: usize = 0x2A8;
    pub const SMART_PTR_2A8: usize = 0x2A8;

    /// Активный ref ptr.
    pub const REF_PTR_2B0: usize = 0x2B0;

    /// Sentinel.
    pub const SENTINEL_2BC: usize = 0x2BC;

    /// Коэффициент демпфирования (0.3).
    pub const DAMPING_FACTOR: usize = 0x2E0;

    /// tail/config block (legacy names)
    pub const FIELD_2B0: usize = 0x2B0;
    pub const FIELD_2B8: usize = 0x2B8;
    pub const FIELD_2BC: usize = 0x2BC;
    pub const FIELD_2C0: usize = 0x2C0;
    pub const FIELD_2C8: usize = 0x2C8;
    pub const FIELD_2D0: usize = 0x2D0;
    pub const FIELD_2D8: usize = 0x2D8;
    pub const FIELD_2E0: usize = 0x2E0;
    pub const FIELD_2E8: usize = 0x2E8;
}

/// Generic smart-pointer slot helpers.
pub mod smart_ptr_slot {
    /// `+0x00` -> stored refcounted object pointer.
    pub const PTR: usize = 0x00;
}

/// Generic refcounted object conventions observed in engine.
pub mod refcounted_object {
    /// `+0x08` -> refcount (`i32`) in many engine-owned refcounted objects.
    pub const REFCOUNT: usize = 0x08;
}

// Generic 0x58-byte container with 0x30-byte sentinel/root.
/// Paired helpers:
/// - `M2DE_InitContainer58_WithSentinel30`
/// - `M2DE_TeardownContainer58_WithSentinel30`
pub mod container58 {
    /// `+0x08..+0x18` -> first vector-like storage (begin/end/capacity)
    pub const VEC1_BEGIN: usize = 0x08;
    pub const VEC1_END: usize = 0x10;
    pub const VEC1_CAPACITY: usize = 0x18;

    /// `+0x28..+0x38` -> second vector-like storage (begin/end/capacity)
    pub const VEC2_BEGIN: usize = 0x28;
    pub const VEC2_END: usize = 0x30;
    pub const VEC2_CAPACITY: usize = 0x38;

    /// `+0x40` -> sentinel/root pointer
    pub const ROOT: usize = 0x40;

    /// `+0x48` -> count/state-like field
    pub const COUNT_OR_STATE_A: usize = 0x48;

    /// `+0x50` -> count/state-like field
    pub const COUNT_OR_STATE_B: usize = 0x50;

    /// Full container size
    pub const SIZE: usize = 0x58;
}

// =============================================================================
//  CHuman расширенные поля (+0x168..+0x310)
// =============================================================================

pub mod human {
    /// `+0x158` -> неизвестный параметр урона/физики (f32).
    pub const DAMAGE_PARAM_158: usize = 0x158;

    /// `+0x15C` -> масштабный коэффициент урона (f32).
    pub const DAMAGE_SCALE_FACTOR: usize = 0x15C;

    /// `+0x168` -> указатель на дескриптор внешнего вида.
    ///
    /// Структура дескриптора:
    /// - +0x00: u32 — идентификатор текущего облика
    /// - +0x0C: u32 — маска флагов совместимости
    /// - +0x18: данные модели
    /// - +0xA8: строка с именем модели (null-terminated)
    pub const MODEL_DESCRIPTOR: usize = 0x168;

    /// `+0x170` -> сохранённый frame node при смене модели.
    pub const SAVED_FRAME_PTR: usize = 0x170;

    /// `+0x178` -> сохранённый флаг стриминга при смене модели.
    pub const SAVED_STREAMING_FLAG: usize = 0x178;

    /// `+0x278` -> вспомогательный объект (32 байта). Создаётся лениво.
    pub const HELPER_OBJECT: usize = 0x278;

    /// `+0x294` -> целевая скорость движения (f32).
    pub const MOVEMENT_SPEED_TARGET: usize = 0x294;

    /// `+0x298` -> текущая скорость движения (f32).
    pub const MOVEMENT_SPEED_CURRENT: usize = 0x298;

    /// `+0x2F8` -> флаг активности наложения модели (u8).
    pub const MODEL_OVERLAY_ACTIVE: usize = 0x2F8;

    /// `+0x300` -> состояние наложения модели.
    pub const MODEL_OVERLAY_STATE: usize = 0x300;

    /// `+0x308` -> флаг отсоединения frame node при смене модели (u8).
    pub const FRAME_DETACH_FLAG: usize = 0x308;

    /// `+0x310` -> указатель на физическое тело коллизии.
    /// NULL = коллизия отключена. Bit 2 по смещению +2 = активна.
    pub const COLLISION_BODY: usize = 0x310;
}

/// Смещения внутри дескриптора внешнего вида (human+0x168).
pub mod model_descriptor {
    /// `+0x00` -> идентификатор текущего облика (u32).
    pub const APPEARANCE_ID: usize = 0x00;
    /// `+0x0C` -> маска флагов совместимости (u32).
    pub const FLAGS: usize = 0x0C;
    /// `+0x18` -> данные модели.
    pub const DATA_PAYLOAD: usize = 0x18;
    /// `+0xA8` -> строка с именем модели (null-terminated).
    pub const MODEL_NAME: usize = 0xA8;
}

// =============================================================================
//  Контроллер передвижения персонажа (human+0x258)
// =============================================================================

/// Смещения внутри объекта контроллера передвижения.
///
/// Контроллер управляет анимационной стейт-машиной персонажа:
/// направлением, скоростью, поворотом, укрытиями, боевыми стойками,
/// стилями смерти/ранений, управлением транспортом.
///
/// Всего 68 слотов в таблице методов.
pub mod locomotion_controller {
    /// `+0x08` -> обратная ссылка на владельца (CHuman*).
    pub const OWNER: usize = 0x08;

    /// `+0x10` -> сентинел (-1).
    pub const SENTINEL: usize = 0x10;

    /// `+0x18` -> счётчик/флаги.
    pub const FLAGS: usize = 0x18;

    /// `+0x20` -> указатель на контекст.
    pub const CONTEXT: usize = 0x20;

    /// Общее количество слотов в таблице методов.
    pub const SLOT_COUNT: usize = 68;

    // Байтовые смещения именованных слотов от начала таблицы методов.
    /// Слот получения направления (Vec3).
    pub const VFUNC_GET_DIR: usize = 22 * 8;
    /// Слот получения поворота (кватернион).
    pub const VFUNC_GET_ROTATION: usize = 24 * 8;
    /// Слот получения скорости.
    pub const VFUNC_GET_VELOCITY: usize = 25 * 8;
    /// Слот получения состояния передвижения.
    pub const VFUNC_GET_STATE: usize = 36 * 8;
    /// Слот получения трансформации.
    pub const VFUNC_GET_TRANSFORM: usize = 40 * 8;
    /// Слот запроса зоны с параметром.
    pub const VFUNC_QUERY_ZONE_PARAM: usize = 45 * 8;
    /// Слот проверки зоны.
    pub const VFUNC_QUERY_ZONE_CHECK: usize = 46 * 8;

    // Подтверждённые поля данных контроллера (прямое чтение памяти).

    /// `+0x1E8` -> кватернион ориентации (f32 x4: x, y, z, w).
    /// Используется в get_direction() для конвертации в forward vector.
    pub const ORIENTATION_QUAT: usize = 0x1E8;

    /// `+0x230` -> вектор скорости (Vec3, f32 x3).
    /// Используется в get_velocity() для прямого чтения.
    pub const VELOCITY: usize = 0x230;

    /// `+0x298` -> объект трансформации.
    pub const TRANSFORM_OBJ: usize = 0x298;
}

// =============================================================================
//  C_Car — подтверждённые поля (из IDA decompile vtable)
// =============================================================================

/// Подтверждённые поля C_Car (из decompile vtable C_Car).
pub mod car_confirmed {
    /// `+0xB0` -> pending dispatch begin.
    pub const PENDING_DISPATCH_BEGIN: usize = 0xB0;
    /// `+0xB8` -> pending dispatch end.
    pub const PENDING_DISPATCH_END: usize = 0xB8;

    /// `+0xC8` -> records begin.
    pub const RECORDS_BEGIN: usize = 0xC8;
    /// `+0xD0` -> records end.
    pub const RECORDS_END: usize = 0xD0;
    /// `+0xD8` -> records capacity.
    pub const RECORDS_CAP: usize = 0xD8;

    /// `+0xE0` -> physics sub-vtable pointer.
    pub const PHYSICS_SUBOBJECT: usize = 0xE0;

    /// `+0x270` -> world matrix 4x4 (f32[16], row-major).
    pub const WORLD_MATRIX: usize = 0x270;

    /// `+0x2C0` -> позиция X (из world_matrix[12]).
    pub const POS_X: usize = 0x2C0;
    /// `+0x2D0` -> позиция Y (из world_matrix[13]).
    pub const POS_Y: usize = 0x2D0;
    /// `+0x2E0` -> позиция Z (из world_matrix[14]).
    pub const POS_Z: usize = 0x2E0;

    /// `+0xED8` -> physics body pointer.
    pub const PHYSICS_BODY: usize = 0xED8;

    /// `+0xF10` -> behavior component pointer.
    pub const BEHAVIOR: usize = 0xF10;

    /// `+0xF30` -> car flags (u64).
    pub const CAR_FLAGS: usize = 0xF30;

    /// `+0xF48` -> template resource pointer.
    pub const TEMPLATE_RESOURCE: usize = 0xF48;

    /// `+0xF88` -> variant index (u32).
    pub const VARIANT_INDEX: usize = 0xF88;

    /// `+0x11EC` -> pos committed flag (u8).
    pub const POS_COMMITTED: usize = 0x11EC;

    /// `+0x1210` -> collision body pointer.
    pub const COLLISION_BODY: usize = 0x1210;

    /// `+0x1218` -> collision body refcount (i32).
    pub const COLLISION_BODY_REFCOUNT: usize = 0x1218;
}

// =============================================================================
//  C_CarVehicle — подтверждённые поля
// =============================================================================

/// Подтверждённые поля C_CarVehicle.
pub mod car_vehicle_confirmed {
    /// `+0xD0` -> physics params (0x44 байта, inline).
    pub const PHYSICS_PARAMS: usize = 0xD0;

    /// `+0x118` -> SDS name 1 / cloth slot (32 байта: u8 flag + char[31]).
    pub const SDS_NAME_1: usize = 0x118;
    /// `+0x138` -> SDS name 2 / body slot (32 байта).
    pub const SDS_NAME_2: usize = 0x138;
    /// `+0x158` -> SDS name 3 / look slot (32 байта).
    pub const SDS_NAME_3: usize = 0x158;

    /// `+0x178` -> extended params (0x30 байт).
    pub const EXTENDED_PARAMS: usize = 0x178;

    /// `+0x1A8` -> global subsystem pointer.
    pub const GLOBAL_SUBSYSTEM: usize = 0x1A8;
}

/// Crash part object (базовый layout, общий для всех типов).
/// Создаётся CCar::CreateCrashPart (vtable[89]).
pub mod crash_part {
    /// +0x00: vtable crash part.
    pub const VTABLE: usize = 0x00;
    /// +0x08: тип детали (u32, коды 0-16).
    pub const PART_TYPE: usize = 0x08;
    /// +0x10: флаги (u32, 0x80000000=detachable, 0x08=physical, etc).
    pub const FLAGS: usize = 0x10;
    /// +0x18: указатель на parent entity context.
    pub const PARENT_CONTEXT: usize = 0x18;
    /// +0x34: float, параметр (обычно прочность?).
    pub const STRENGTH_PARAM: usize = 0x34;
    /// +0x38: float, параметр.
    pub const PARAM_38: usize = 0x38;
    /// +0x148: указатель на sub-context (для сложных деталей).
    pub const SUB_CONTEXT: usize = 0x148;
}

pub mod car_damage {
    /// Индексы основной damage group A (u32 array).
    pub const GROUP_A_BEGIN: usize = 0x6B0;
    pub const GROUP_A_END: usize = 0x6B8;

    /// Связки parent/child между деталями.
    pub const LINK_GROUP_BEGIN: usize = 0x6C8;
    pub const LINK_GROUP_END: usize = 0x6D0;

    /// Damage group B.
    pub const GROUP_B_BEGIN: usize = 0x6E0;
    pub const GROUP_B_END: usize = 0x6E8;

    /// Damage group C.
    pub const GROUP_C_BEGIN: usize = 0x710;
    pub const GROUP_C_END: usize = 0x718;

    /// Damage group D.
    pub const GROUP_D_BEGIN: usize = 0x740;
    pub const GROUP_D_END: usize = 0x748;

    /// Active FX/detachable-part index list.
    pub const FX_GROUP_BEGIN: usize = 0x758;
    pub const FX_GROUP_END: usize = 0x760;

    /// Crash-event bucket groups, stride 0x260.
    pub const EVENT_BUCKETS_BEGIN: usize = 0x8A0;
    pub const EVENT_BUCKETS_END: usize = 0x8A8;
    pub const EVENT_BUCKET_STRIDE: usize = 0x260;

    /// Damage/crash flags dword.
    pub const FLAGS_AA8: usize = 0xAA8;

    /// Runtime crash flags qword.
    pub const FLAGS_AB0: usize = 0xAB0;

    /// Secondary crash-state flags qword.
    pub const FLAGS_AB8: usize = 0xAB8;

    /// Crash FX / manager ptr.
    pub const FX_MANAGER_AC8: usize = 0xAC8;
}
