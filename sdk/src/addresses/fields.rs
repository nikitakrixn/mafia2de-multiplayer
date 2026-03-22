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
    /// +0x08 → указатель на std::vector<ScriptMachine*>
    pub const VECTOR: usize = 0x08;
}

/// Внутренний std::vector (begin/end/capacity).
///
/// Используется для vector'а script machines и других
/// std::vector в структурах движка.
pub mod std_vector {
    /// +0x00 → первый элемент
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
    pub const SUB_VTABLE_1: usize = 0xE0;   // 0x141850298
    pub const SUB_VTABLE_2: usize = 0x1E0;  // 0x141850478
    pub const SUB_VTABLE_3: usize = 0x1E8;  // 0x1418504C0
    pub const SUB_VTABLE_4: usize = 0x1F8;  // 0x1418504E0
    pub const SUB_VTABLE_5: usize = 0x210;  // 0x1418504F0

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
