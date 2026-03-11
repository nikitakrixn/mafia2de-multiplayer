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

    /// `+0x258` → Physics body handler (optional, may be NULL)
    ///
    /// When present, position is read via vtable[0xA8] instead of frame node.
    pub const PHYSICS_HANDLER: usize = 0x258;
}

pub mod inventory {
    pub const TYPE: usize = 0x24;
    pub const SLOTS_START: usize = 0x50;
    pub const SLOTS_END: usize = 0x58;
    pub const WEAPONS: usize = 0xE8;
    
    /// `+0x170` → back-pointer на entity-владельца (C_Human* для игрока)
    ///
    /// Проверка `*(parent + 0x24) == 16` в игре определяет
    /// показывать ли HUD popup. Для player это значение = 0,
    /// поэтому HUD вызывается через g_HUDManager напрямую.
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
    /// `+0x64` → Position X (f32)
    ///
    /// Part of 4x4 transform matrix starting at +0x58.
    /// Position is in the last column of each row (stride 0x10).
    ///
    /// IDA: `0x140DA7630` reads `[frame+0x64]`, `[frame+0x74]`, `[frame+0x84]`
    pub const POS_X: usize = 0x64;
    /// `+0x74` → Position Y (f32)
    pub const POS_Y: usize = 0x74;
    /// `+0x84` → Position Z (f32)
    pub const POS_Z: usize = 0x84;
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

// ═══════════════════════════════════════════════════════════════════
//  Camera System
// ═══════════════════════════════════════════════════════════════════

/// CameraManager — статический объект, НЕ указатель.
///
/// Доступ: `module_base + globals::CAMERA_MANAGER + offset`
///
/// Содержит два PlayerCameraView (Interier/Exterier),
/// TransitSpeed, и все конфиги автомобильных камер.
pub mod camera_manager {
    /// Interier PlayerCameraView base (0xD18 bytes).
    pub const INTERIER_VIEW: usize = 0x0000;
    /// Exterier PlayerCameraView base (0xD18 bytes).
    pub const EXTERIER_VIEW: usize = 0x0D18;

    /// Interier default FOV (float).
    /// = INTERIER_VIEW + camera_view::DEFAULT_PARAMS + FOV_INDEX * 4
    pub const INTERIER_DEFAULT_FOV: usize = 0x0C80;
    /// Exterier default FOV (float).
    /// = EXTERIER_VIEW + camera_view::DEFAULT_PARAMS + FOV_INDEX * 4
    pub const EXTERIER_DEFAULT_FOV: usize = 0x1998;

    /// TransitSpeed Exterier (float).
    pub const TRANSIT_SPEED_EXTERIER: usize = 0x1A30;
    /// TransitSpeed Interier (float).
    pub const TRANSIT_SPEED_INTERIER: usize = 0x1A34;
    /// PlayerCamera loaded flag (byte).
    pub const PLAYER_CAMERA_LOADED: usize = 0x1FB4;

    // ── Car/other camera param destinations ──

    /// carCameraBumper params (Fov[4]).
    pub const CAR_CAMERA_BUMPER_PARAMS: usize = 0x21D8;
    /// carCameraWheel params (Fov[4]).
    pub const CAR_CAMERA_WHEEL_PARAMS: usize = 0x21E8;
    /// carCameraHood params (Fov[4]).
    pub const CAR_CAMERA_HOOD_PARAMS: usize = 0x21F8;
    /// carCameraShoot params (RotRatioX[15]).
    pub const CAR_CAMERA_SHOOT_PARAMS: usize = 0x2208;
    /// carCameraDynamic params (RotRatioX[25]).
    pub const CAR_CAMERA_DYNAMIC_PARAMS: usize = 0x2244;
    /// carCameraDynamicLong params (RotRatioX[25]).
    pub const CAR_CAMERA_DYNAMIC_LONG_PARAMS: usize = 0x22A8;
    /// carCameraLookback params (Fov[4]).
    pub const CAR_CAMERA_LOOKBACK_PARAMS: usize = 0x230C;
    /// carCameraGamepad params (RotRatioX[24]).
    pub const CAR_CAMERA_GAMEPAD_PARAMS: usize = 0x231C;
    /// fpvCamera params (Fov[8]).
    pub const FPV_CAMERA_PARAMS: usize = 0x275C;
    /// deathCamera params (Fov[2]).
    pub const DEATH_CAMERA_PARAMS: usize = 0x277C;
    /// meeleeCamera params (MinDistance[28]).
    pub const MEELEE_CAMERA_PARAMS: usize = 0x2784;
}

/// PlayerCameraView — одна камерная "виды" (Interier или Exterier).
///
/// Размер: 0xD18 (3352) байт.
///
/// Содержит 15 state-блоков + defaults.
/// State names: Stay, Walk, Run, Sprint, ...
///
/// Каждый state содержит:
/// - Params\[27\] (float) — параметры камеры
/// - Speeds\[15\] (float) — скорости переходов
/// - ParamFlags\[27\] (byte) — 0=override, 1=use default
/// - SpeedFlags\[15\] (byte)
pub mod camera_view {
    /// Full size of one CameraView.
    pub const SIZE: usize = 0xD18;
    /// Number of camera states per view.
    pub const NUM_STATES: usize = 15;
    /// Number of parameters per state.
    pub const NUM_PARAMS: usize = 27;
    /// Number of speed values per state.
    pub const NUM_SPEEDS: usize = 15;

    // ── State layout ──

    /// Base offset of State\[0\] from CameraView start.
    pub const STATES_BASE: usize = 0x04;
    /// Stride between consecutive states (212 bytes).
    pub const STATE_STRIDE: usize = 0xD4;

    /// Within a state block: offset to Params\[27\] array (float\[27\]).
    pub const STATE_PARAMS_OFFSET: usize = 0x00;
    /// Within a state block: offset to Speeds\[15\] array (float\[15\]).
    pub const STATE_SPEEDS_OFFSET: usize = 0x6C;
    /// Within a state block: offset to ParamFlags\[27\] (byte\[27\]).
    /// 0 = has override value, 1 = use default.
    pub const STATE_PARAM_FLAGS_OFFSET: usize = 0xA8;
    /// Within a state block: offset to SpeedFlags\[15\] (byte\[15\]).
    pub const STATE_SPEED_FLAGS_OFFSET: usize = 0xC3;

    // ── Defaults ──

    /// Offset of DefaultParams\[27\] from CameraView start (float\[27\]).
    pub const DEFAULT_PARAMS: usize = 0xC70;
    /// Offset of DefaultSpeeds\[15\] from CameraView start (float\[15\]).
    pub const DEFAULT_SPEEDS: usize = 0xCDC;
}