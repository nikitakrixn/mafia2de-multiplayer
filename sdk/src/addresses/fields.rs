//! Смещения полей от начала структуры (в байтах).

pub mod game_manager {
    /// `+0x180` → `Player*`
    pub const ACTIVE_PLAYER: usize = 0x180;
}

pub mod player {
    /// `+0xE8` → `Inventory*`
    pub const INVENTORY: usize = 0xE8;

    /// `+0xF0` → `PlayerControl*` / control component
    pub const CONTROL_COMPONENT: usize = 0xF0;

    /// `+0x78` → Frame/transform node pointer
    ///
    /// Contains 4x4 transformation matrix.
    /// Position: frame+0x64 (X), frame+0x74 (Y), frame+0x84 (Z)
    ///
    /// IDA: `0x140DA7630` fallback path reads `[rcx+0x78]`
    pub const FRAME_NODE: usize = 0x78;

    /// `+0x024` → uint8 тип сущности.
    /// 0x10 = игрок, 0x0E = NPC-человек, 0x12 = физическое тело.
    pub const ENTITY_TYPE: usize = 0x24;

    /// `+0x080` → C_Entity* владелец.
    /// Пешком = NULL, в машине = указатель на vehicle entity.
    pub const OWNER: usize = 0x80;

    /// `+0x0C0` → AI/Navigation компонент.
    /// vtbl 0x1418E3290. Хранит текущую AI-позицию.
    pub const AI_NAV_COMPONENT: usize = 0xC0;

    /// `+0x0D0` → TransformSync компонент.
    /// vtbl 0x1418E33A8. Синхронизирует позицию из актора ~258 раз/сек.
    /// comp+0x18 = Vec3 позиция, comp+0x24 = параметр (100.0).
    pub const TRANSFORM_SYNC: usize = 0xD0;

    /// `+0x0F8` → Behavior компонент.
    /// vtbl 0x1418E37A8. Сюда пересылаются сообщения из HandleMessage.
    pub const BEHAVIOR_COMPONENT: usize = 0xF8;

    /// `+0x108` → Компонент состояния оружия.
    /// *(component + 0x2B0) → указатель на ID выбранного оружия (uint32).
    pub const WEAPON_STATE_COMPONENT: usize = 0x108;

    /// `+0x148` → float текущее здоровье.
    /// 720.0 = полное на нормальной сложности.
    pub const CURRENT_HEALTH: usize = 0x148;

    /// `+0x14C` → float.
    /// Для NPC: максимальное здоровье.
    /// Для игрока: множитель урона определённых типов (12/15/22).
    /// Максимум здоровья игрока берётся из g_M2DE_PlayerData+0x00.
    pub const NPC_HEALTHMAX: usize = 0x14C;

    /// `+0x150` → float множитель урона от NPC.
    /// Lua возвращает это * 100.0 как проценты. Default 1.0 (= 100%).
    pub const NONPLAYER_DAMAGE_MULT: usize = 0x150;

    /// `+0x154` → float пороговая дистанция для снижения урона.
    /// Default 5.0.
    pub const NONPLAYER_DAMAGE_DIST: usize = 0x154;

    /// `+0x160` → uint8 флаг неуязвимости.
    /// 0 = обычный режим, 1 = неуязвим (урон пропускается полностью).
    pub const INVULNERABILITY: usize = 0x160;

    /// `+0x161` → uint8 флаг смерти.
    /// IsDeath() = vtable[47] = return *(this + 353).
    pub const IS_DEAD: usize = 0x161;

    /// `+0x162` → uint8 флаг полубога.
    /// Если установлен — здоровье не опускается ниже 1.0.
    pub const DEMIGOD: usize = 0x162;

    /// `+0x180` → float* массив множителей урона по частям тела.
    /// [4]=голова, [5]=торс, [6]=руки, [7]=ноги.
    pub const BODY_DAMAGE_MULTIPLIERS: usize = 0x180;

    /// `+0x190` → C_Human* ссылка на самого себя (== this).
    /// Используется для валидации указателя.
    pub const SELF_REF: usize = 0x190;

    /// `+0x258` → Physics provider (выделен в куче отдельно).
    /// vtbl 0x141993998. Используется GetPos/SetPos для physics-пути.
    pub const PHYSICS_PROVIDER: usize = 0x258;

    /// `+0x338` → Vec3 позиция смерти (12 байт).
    /// Записывается когда здоровье достигает 0.
    pub const DEATH_POSITION: usize = 0x338;

    /// `+0x344` → int32 тип смерти (1=обычная, 128=взрыв).
    pub const DEATH_TYPE: usize = 0x344;
}

pub mod inventory {
    /// `+0x08` → Корень RB-дерева поиска оружия по ID.
    /// std::map<int32, WeaponData*>.
    /// node+0x20 = weapon_id (ключ), node+0x28 = weapon data ptr.
    pub const WEAPON_TREE: usize = 0x08;

    /// `+0x24` → uint8 тип инвентаря (0=player).
    pub const TYPE: usize = 0x24;

    /// `+0x50` → начало массива указателей на слоты.
    /// *(slots_begin + N*8) = Slot* для слота N.
    pub const SLOTS_START: usize = 0x50;

    /// `+0x58` → конец массива слотов.
    pub const SLOTS_END: usize = 0x58;

    /// `+0x168` → bool бесконечные патроны.
    pub const UNLIMITED_AMMO: usize = 0x168;

    /// `+0x170` → C_Human* обратная ссылка на владельца.
    pub const OWNER_ENTITY_REF: usize = 0x170;
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

/// Индексы слотов инвентаря.
pub mod slots {
    /// Слот 0 — текущее оружие в руках
    pub const CURRENT_WEAPON: usize = 0;
    /// Слот 1 — неизвестно
    pub const UNKNOWN_1: usize = 1;
    /// Слот 2 — оружейный слот 1 (пистолеты, дробовики)
    pub const WEAPON_1: usize = 2;
    /// Слот 3 — оружейный слот 2 (винтовки, автоматы)
    pub const WEAPON_2: usize = 3;
    /// Слот 4 — запас патронов
    pub const AMMO: usize = 4;
    /// Слот 5 — деньги
    pub const MONEY: usize = 5;
}

/// Структура одного слота инвентаря.
pub mod slot {
    /// `+0x18` → начало std::vector<ptr> элементов
    pub const VEC_BEGIN: usize = 0x18;
    /// `+0x20` → конец std::vector
    pub const VEC_END: usize = 0x20;
    /// `+0x50` → указатель на weapon table entry
    pub const TABLE_ENTRY: usize = 0x50;
}

/// Запись из /tables/weapons.tbl.
pub mod weapon_table_entry {
    /// `+0x24` → int32 flags. Бит 1 (& 2): слот назначения
    pub const FLAGS: usize = 0x24;
    /// `+0x58` → int32 максимальная ёмкость обоймы
    pub const MAX_AMMO: usize = 0x58;
}

/// Данные оружия (из RB-дерева или weapon_state компонента).
pub mod weapon_data {
    /// `+0x00` → int32 ID оружия
    pub const WEAPON_ID: usize = 0x00;
    /// `+0x10` → ptr → container → +0x10 → int32 текущие патроны
    pub const AMMO_CONTAINER: usize = 0x10;
    /// `+0x24` → uint32 флаги типа оружия
    ///   бит 5 (0x20) = огнестрельное
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
    /// `+0x2B0` → ptr на WeaponData текущего оружия в руках
    /// NULL = руки пусты.
    pub const CURRENT_WEAPON_DATA: usize = 0x2B0;
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

pub mod hud_manager {
    /// `+0x98` → Money display component (HudMoneyDisplay*)
    pub const MONEY_DISPLAY: usize = 0x98;
}

pub mod hud_money_display {
    /// `+0x48` → displayed/animated value (i64)
    pub const ANIMATED_VALUE: usize = 0x48;
    /// `+0x50` → actual target value (i64)
    pub const TARGET_VALUE: usize = 0x50;
    /// `+0x5C` → animation timer (f32, popup shows only when <= 0)
    pub const ANIM_TIMER: usize = 0x5C;
    /// `+0x78` → popup enabled flag (bool)
    pub const POPUP_ENABLED: usize = 0x78;
}

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

// ═══════════════════════════════════════════════════════════════════════
//  RenderDevice и swapchain (новые)
// ═══════════════════════════════════════════════════════════════════════

pub mod render_device {
    /// `+0x2008` → начало блока init-параметров рендера.
    ///
    /// Внутри этого блока лежат:
    /// - `+0x18` от блока → текущая ширина
    /// - `+0x1C` от блока → текущая высота
    /// - `+0x20` от блока → указатель на window/config структуру
    pub const INIT_CONFIG: usize = 0x2008;

    /// `+0x2020` → текущая ширина рендера.
    ///
    /// Поле обновляется и в recreate/resize path.
    pub const RENDER_WIDTH: usize = 0x2020;

    /// `+0x2024` → текущая высота рендера.
    pub const RENDER_HEIGHT: usize = 0x2024;

    /// `+0x2028` → указатель на window/config структуру из init params.
    pub const WINDOW_CONFIG_PTR: usize = 0x2028;

    /// `+0x2032` → поддержка feature level 10.0.
    pub const SUPPORTS_FL_10_0: usize = 0x2032;

    /// `+0x2033` → дополнительный флаг поддержки FL10.0.
    pub const SUPPORTS_FL_10_0_DUP: usize = 0x2033;

    /// `+0x2034` → поддержка feature level 10.1.
    pub const SUPPORTS_FL_10_1: usize = 0x2034;

    /// `+0x2035` → флаг завершённой DX-инициализации.
    pub const DX_INITIALIZED: usize = 0x2035;

    /// `+0x203C` → количество выходов текущего адаптера.
    pub const ADAPTER_OUTPUT_COUNT: usize = 0x203C;

    /// `+0x2040` → максимальный размер текстур.
    pub const MAX_TEXTURE_SIZE: usize = 0x2040;

    /// `+0x2044` → настройка фильтрации / aniso-related state.
    pub const ANISO_FILTER_SETTING: usize = 0x2044;

    /// `+0x2050` → shader/resource cache из базового конструктора.
    pub const SHADER_CACHE: usize = 0x2050;

    /// `+0x2070` → dynamic vertex buffer resource.
    ///
    /// Строка: `"RenderDeviceBase::DynamicVB"`
    pub const DYNAMIC_VB: usize = 0x2070;

    /// `+0x2078` → dynamic index buffer resource.
    ///
    /// Строка: `"RenderDeviceBase::DynamicIB"`
    pub const DYNAMIC_IB: usize = 0x2078;

    /// `+0x21A8` → текущий режим/профиль рендера.
    ///
    /// Используется в switch внутри init.
    pub const CURRENT_STATE_MODE: usize = 0x21A8;

    /// `+0x21AC` → вторичный state mode.
    pub const CURRENT_STATE_MODE_B: usize = 0x21AC;

    /// `+0x2780` → `IDXGIFactory1*`
    pub const DXGI_FACTORY: usize = 0x2780;

    /// `+0x2788` → `D3D_FEATURE_LEVEL`
    pub const FEATURE_LEVEL: usize = 0x2788;

    /// `+0x2790` → `ID3D11Device*`
    pub const D3D_DEVICE: usize = 0x2790;

    /// `+0x2798` → `ID3D11DeviceContext*`
    pub const D3D_CONTEXT: usize = 0x2798;

    /// `+0x27A0` → `M2DE_SwapChainManager*`
    pub const SWAPCHAIN_MANAGER: usize = 0x27A0;

    /// `+0x27A8` → `M2DE_SwapChainWrapper*`
    ///
    /// Это основной путь к текущему DXGI swapchain.
    pub const CURRENT_SWAPCHAIN: usize = 0x27A8;

    /// `+0x27B0` → дополнительный указатель на активный swapchain wrapper.
    pub const ACTIVE_SWAPCHAIN: usize = 0x27B0;

    /// `+0x510C` → флаги адаптера / init flags.
    ///
    /// Влияют на выбор режима swapchain.
    pub const ADAPTER_FLAGS: usize = 0x510C;

    /// `+0x5110` → `ID3DUserDefinedAnnotation*` или NULL.
    pub const DEBUG_ANNOTATION: usize = 0x5110;
}

pub mod swapchain_manager {
    /// `+0x00` → root/sentinel узел RB-дерева.
    ///
    /// Ключ дерева: `HWND`
    /// Значение: `M2DE_SwapChainWrapper*`
    pub const TREE_ROOT: usize = 0x00;

    /// `+0x08` → количество элементов в дереве.
    pub const TREE_SIZE: usize = 0x08;

    /// `+0x10` → `IDXGIFactory4*`
    pub const FACTORY: usize = 0x10;

    /// `+0x18` → `ID3D11Device*`
    pub const DEVICE: usize = 0x18;

    /// `+0x20` → `ID3D11DeviceContext*`
    pub const CONTEXT: usize = 0x20;

    /// `+0x28` → поддерживается ли `DXGI_FEATURE_PRESENT_ALLOW_TEARING`.
    pub const TEARING_SUPPORTED: usize = 0x28;

    /// `+0x29` → debug mode флаг.
    pub const DEBUG_MODE: usize = 0x29;
}

pub mod swapchain_wrapper {
    /// `+0x00` → ширина swapchain.
    pub const WIDTH: usize = 0x00;

    /// `+0x04` → высота swapchain.
    pub const HEIGHT: usize = 0x04;

    /// `+0x08` → режим / флаг swapchain.
    pub const SWAPCHAIN_MODE: usize = 0x08;

    /// `+0x10` → HWND окна.
    pub const HWND: usize = 0x10;

    /// `+0x18` → `IDXGISwapChain1*`
    ///
    /// Это raw DXGI swapchain, который нужен для hook'а `Present`.
    pub const SWAPCHAIN: usize = 0x18;

    /// `+0x20` → `ID3D11Texture2D*` back buffer.
    pub const BACK_BUFFER: usize = 0x20;

    /// `+0x28` → `ID3D11Texture2D*` depth texture.
    pub const DEPTH_TEXTURE: usize = 0x28;

    /// `+0x30` → `ID3D11DepthStencilView*`
    pub const DSV: usize = 0x30;

    /// `+0x38` → `ID3D11RenderTargetView*`
    pub const RTV: usize = 0x38;

    /// `+0x40` → `ID3D11ShaderResourceView*`
    pub const SRV: usize = 0x40;
}

// ═══════════════════════════════════════════════════════════════════
//  Script Machine (Lua VM)
// ═══════════════════════════════════════════════════════════════════

/// Менеджер скриптовых машин.
///
/// Цепочка до lua_State*:
/// `g_ScriptMachineManager + 0x08` → vector
/// `vector + 0x00` → begin (ScriptMachine**)
/// `array[0]` → Main Game Script Machine
/// `machine + 0x70` → lua_State*
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
    /// +0x08 → один за последним
    pub const END: usize = 0x08;
    /// +0x10 → конец выделенной памяти
    pub const CAPACITY: usize = 0x10;
}

/// Одна ScriptMachine.
pub mod script_machine {
    /// +0x70 → lua_State*
    pub const LUA_STATE: usize = 0x70;
}

// ═══════════════════════════════════════════════════════════════════════
//  Camera System
// ═══════════════════════════════════════════════════════════════════════

/// `CameraManager` — статический объект камерной системы.
///
/// Доступ:
/// ```ignore
/// let camera_mgr = module_base + addresses::globals::CAMERA_MANAGER;
/// ```
///
/// ⚠️ Это не `CameraManager*`, а сам объект по фиксированному RVA.
///
/// Содержит:
/// - два `PlayerCameraView`:
///   - `Interier` по `+0x0000`
///   - `Exterier` по `+0x0D18`
/// - `TransitSpeed`
/// - параметры автомобильных камер
/// - fpv/death/meelee камеры
///
/// Подтверждено из:
/// - `M2DE_CameraSystem_Init`
/// - `M2DE_CameraManager_LoadConfigById`
/// - `M2DE_CameraManager_LoadPlayerCamera`
pub mod camera_manager {
    // ── Player camera ───────────────────────────────────────────────

    /// `Interier` PlayerCameraView base.
    pub const INTERIER_VIEW: usize = 0x0000;

    /// `Exterier` PlayerCameraView base.
    pub const EXTERIER_VIEW: usize = 0x0D18;

    /// `Interier.DefaultParams[Fov]`
    ///
    /// Формула:
    /// `INTERIER_VIEW + camera_view::DEFAULT_PARAMS + camera_params::FOV * 4`
    pub const INTERIER_DEFAULT_FOV: usize = 0x0C80;

    /// `Exterier.DefaultParams[Fov]`
    ///
    /// Формула:
    /// `EXTERIER_VIEW + camera_view::DEFAULT_PARAMS + camera_params::FOV * 4`
    pub const EXTERIER_DEFAULT_FOV: usize = 0x1998;

    /// `TransitSpeed["Exterier"]` (float)
    pub const TRANSIT_SPEED_EXTERIER: usize = 0x1A30;

    /// `TransitSpeed["Interier"]` (float)
    pub const TRANSIT_SPEED_INTERIER: usize = 0x1A34;

    /// Флаг, который пишет `M2DE_CameraManager_LoadPlayerCamera(a1, a2)`
    /// в `a1 + 0x1FB4`.
    pub const PLAYER_CAMERA_LOADED: usize = 0x1FB4;

    // ── Простые автомобильные камеры ────────────────────────────────
    // Здесь `Fov` лежит в `params[0]`

    /// `carCameraBumper.xml` → `Params[0] = Fov`
    ///
    /// Params:
    /// - Fov
    /// - Slope
    /// - WheelLeft
    /// - WheelFront
    pub const CAR_BUMPER_FOV: usize = 0x21D8;

    /// `carCameraWheel.xml` → `Params[0] = Fov`
    ///
    /// Params:
    /// - Fov
    /// - Slope
    /// - WheelLeft
    /// - WheelFront
    pub const CAR_WHEEL_FOV: usize = 0x21E8;

    /// `carCameraHood.xml` → `Params[0] = Fov`
    ///
    /// Params:
    /// - Fov
    /// - Slope
    /// - HoodTop
    /// - HoodFront
    pub const CAR_HOOD_FOV: usize = 0x21F8;

    /// `carCameraLookback.xml` → `Params[0] = Fov`
    ///
    /// Params:
    /// - Fov
    /// - Slope
    /// - HOffset
    /// - VOffset
    pub const CAR_LOOKBACK_FOV: usize = 0x230C;

    // ── carCameraDynamic ────────────────────────────────────────────
    //
    // Base params array: +0x2244
    // Count: 25
    // String table base: 0x1418EE3A0
    // Stride: 0x20
    //
    // Reverse result:
    //   Fov    = index 11
    //   FovMax = index 16

    /// `carCameraDynamic.Params[11] = Fov`
    ///
    /// Runtime observed value: ~72.12
    pub const CAR_DYNAMIC_FOV: usize = 0x2244 + 11 * 4; // 0x2270

    /// `carCameraDynamic.Params[16] = FovMax`
    ///
    /// Это speed-based delta FOV.
    /// Runtime observed value: ~9.96
    pub const CAR_DYNAMIC_FOV_MAX: usize = 0x2244 + 16 * 4; // 0x2284

    // ── carCameraDynamicLong ────────────────────────────────────────
    //
    // Base params array: +0x22A8
    // Count: 25
    // Та же string table, что и у carCameraDynamic.
    //
    // Reverse result:
    //   Fov    = index 11
    //   FovMax = index 16

    /// `carCameraDynamicLong.Params[11] = Fov`
    ///
    /// Runtime observed value: ~70.08
    pub const CAR_DYNAMIC_LONG_FOV: usize = 0x22A8 + 11 * 4; // 0x22D4

    /// `carCameraDynamicLong.Params[16] = FovMax`
    ///
    /// Runtime observed value: ~15.0
    pub const CAR_DYNAMIC_LONG_FOV_MAX: usize = 0x22A8 + 16 * 4; // 0x22E8

    // ── carCameraShoot ──────────────────────────────────────────────
    //
    // Base params array: +0x2208
    // Count: 15
    // String table base: 0x1418EE190
    // Stride: 0x20
    //
    // Reverse result:
    //   Fov = index 3

    /// `carCameraShoot.Params[3] = Fov`
    ///
    /// Runtime observed value: ~61.08
    pub const CAR_SHOOT_FOV: usize = 0x2208 + 3 * 4; // 0x2214

    // ── carCameraGamepad ────────────────────────────────────────────
    //
    // Base params array: +0x231C
    // Count: 24
    // String table base: 0x1418EE720
    // Stride: 0x20
    //
    // Reverse result:
    //   Fov    = index 10
    //   FovMax = index 14

    /// `carCameraGamepad.Params[10] = Fov`
    ///
    /// Runtime observed value: ~65.16
    pub const CAR_GAMEPAD_FOV: usize = 0x231C + 10 * 4; // 0x2344

    /// `carCameraGamepad.Params[14] = FovMax`
    ///
    /// Runtime observed value: ~20.04
    pub const CAR_GAMEPAD_FOV_MAX: usize = 0x231C + 14 * 4; // 0x2354

    // ── Остальные камеры ────────────────────────────────────────────

    /// `fpvCamera.Params[0] = Fov`
    pub const FPV_FOV: usize = 0x275C;

    /// `deathCamera.Params[0] = Fov`
    pub const DEATH_FOV: usize = 0x277C;

    /// `meeleeCamera.Params[5] = Fov`
    ///
    /// Params:
    /// - MinDistance
    /// - MaxDistance
    /// - DistanceExponent
    /// - MinZ
    /// - MaxZ
    /// - Fov
    /// - ...
    pub const MEELEE_FOV: usize = 0x2784 + 5 * 4; // 0x2798

    // ── Базы массивов параметров для диагностики ───────────────────

    /// База `carCameraDynamic.Params`
    pub const CAR_DYNAMIC_PARAMS: usize = 0x2244;
    /// Количество параметров `carCameraDynamic`
    pub const CAR_DYNAMIC_PARAM_COUNT: usize = 25;

    /// База `carCameraDynamicLong.Params`
    pub const CAR_DYNAMIC_LONG_PARAMS: usize = 0x22A8;
    /// Количество параметров `carCameraDynamicLong`
    pub const CAR_DYNAMIC_LONG_PARAM_COUNT: usize = 25;

    /// База `carCameraShoot.Params`
    pub const CAR_SHOOT_PARAMS: usize = 0x2208;
    /// Количество параметров `carCameraShoot`
    pub const CAR_SHOOT_PARAM_COUNT: usize = 15;

    /// База `carCameraGamepad.Params`
    pub const CAR_GAMEPAD_PARAMS: usize = 0x231C;
    /// Количество параметров `carCameraGamepad`
    pub const CAR_GAMEPAD_PARAM_COUNT: usize = 24;
}

/// `PlayerCameraView` — один набор player camera параметров
/// (`Interier` или `Exterier`).
///
/// Размер структуры: `0xD18` (3352 байта).
///
/// Внутри:
/// - 15 state-блоков (`Stay`, `Walk`, `Run`, `Sprint`, ...)
/// - `DefaultParams[27]`
/// - `DefaultSpeeds[15]`
pub mod camera_view {
    /// Полный размер одного `CameraView`.
    pub const SIZE: usize = 0xD18;

    /// Количество state-блоков.
    pub const NUM_STATES: usize = 15;

    /// Количество camera params в одном state/default block.
    pub const NUM_PARAMS: usize = 27;

    /// Количество speed params в одном state/default block.
    pub const NUM_SPEEDS: usize = 15;

    // ── State layout ────────────────────────────────────────────────

    /// Смещение первого state-блока от начала `CameraView`.
    ///
    /// То есть `State[0]` начинается с `+0x04`.
    pub const STATES_BASE: usize = 0x04;

    /// Размер одного state-блока.
    ///
    /// `0xD4 = 212` байт.
    pub const STATE_STRIDE: usize = 0xD4;

    /// Внутри state-блока: `Params[27]` (`float[27]`)
    pub const STATE_PARAMS_OFFSET: usize = 0x00;

    /// Внутри state-блока: `Speeds[15]` (`float[15]`)
    pub const STATE_SPEEDS_OFFSET: usize = 0x6C;

    /// Внутри state-блока: `ParamFlags[27]` (`byte[27]`)
    ///
    /// Семантика:
    /// - `0` = для параметра есть state-specific override
    /// - `1` = брать значение из `DefaultParams`
    pub const STATE_PARAM_FLAGS_OFFSET: usize = 0xA8;

    /// Внутри state-блока: `SpeedFlags[15]` (`byte[15]`)
    pub const STATE_SPEED_FLAGS_OFFSET: usize = 0xC3;

    // ── Defaults ────────────────────────────────────────────────────

    /// `DefaultParams[27]` (`float[27]`)
    pub const DEFAULT_PARAMS: usize = 0xC70;

    /// `DefaultSpeeds[15]` (`float[15]`)
    pub const DEFAULT_SPEEDS: usize = 0xCDC;
}