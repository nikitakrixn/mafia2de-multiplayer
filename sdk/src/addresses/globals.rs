//! Глобальные переменные (RVA).
//!
//! Все указатели — **двойная косвенность**:
//! `module_base + RVA` → `*const *mut T`
//!
//! Чтобы получить объект, нужно дважды разыменовать:
//! ```ignore
//! let ptr_addr = base + rva;
//! let obj: *mut T = *(ptr_addr as *const *mut T);
//! ```

/// `GameManager**` — указатель на менеджер игры.
///
/// IDA: `0x141CAF770` (`qword_141CAF770`)
///
/// Чтение:
/// ```text
/// GameManager* mgr = **(GameManager***)(module_base + GAME_MANAGER);
/// Player* player = *(Player**)(mgr + 0x180);
/// ```
pub const GAME_MANAGER: usize = 0x1CA_F770;

/// `C_GarageManager*` — менеджер гаража.
///
/// IDA: `0x143146A18` (`qword_143146A18`)
///
/// VTable: [`super::vtables::garage::GARAGE_MANAGER`]
pub const GARAGE_MANAGER: usize = 0x314_6A18;

/// `ResourceManager*` — менеджер ресурсов.
///
/// IDA: `0x141CA1FD0` (`qword_141CA1FD0`)
pub const RESOURCE_MANAGER: usize = 0x1CA_1FD0;

/// `GameCallbackManager*` — менеджер игровых событий.
///
/// IDA: `0x141CAF038` (`qword_141CAF038`)
///
/// Управляет 39 зарегистрированными событиями.
/// Вызывается ~494 раза за игровую сессию.
pub const GAME_CALLBACK_MANAGER: usize = 0x1CA_F038;

/// Аллокатор памяти.
///
/// IDA: `0x141CD4A28`
pub const MEMORY_ALLOCATOR: usize = 0x1CD_4A28;

/// Менеджер моделей машин.
///
/// IDA: `0x141CAE1D8`
pub const CAR_MODEL_MANAGER: usize = 0x1CA_E1D8;

/// Трансформация машины по умолчанию.
///
/// IDA: `0x141CBC0D0`
pub const DEFAULT_VEHICLE_TRANSFORM: usize = 0x1CB_C0D0;

/// Система загрузки ресурсов.
///
/// IDA: `0x141CA52B8`
pub const RESOURCE_LOADER: usize = 0x1CA_52B8;

/// HUD Manager (управление отображением денег, иконок).
///
/// IDA: `qword_143138FA8`
/// Доступ: `sub_140D01600()` возвращает этот глобал.
///
/// `+0x98` → Money display component (для popup)
pub const HUD_MANAGER: usize = 0x313_8FA8;

/// Notify Manager (система нотификаций).
///
/// IDA: `qword_141CABBE0`
pub const NOTIFY_MANAGER: usize = 0x1CA_BBE0;

/// Stats Tracker 1 (отслеживание доходов).
///
/// IDA: `qword_1431464A0`
pub const STATS_TRACKER_1: usize = 0x314_64A0;

/// Stats Tracker 2 (отслеживание расходов).
///
/// IDA: `qword_143140BD0`
pub const STATS_TRACKER_2: usize = 0x314_0BD0;

/// `C_ScriptMachineManager*` — singleton менеджер Lua script machines.
///
/// IDA: `qword_141CB1238`
///
/// Цепочка:
/// `manager + 0x08 -> vector*`
/// `vector + 0x00 -> ScriptMachine**`
/// `array[0] -> Main Game Script Machine`
/// `script_machine + 0x70 -> lua_State*`
pub const SCRIPT_MACHINE_MANAGER: usize = 0x1CB_1238;

/// `M2DE_C_RenderDeviceD3D11*` — глобальный singleton рендер-устройства.
///
/// Это главный объект DX11 backend'а игры.
/// Через него можно получить:
/// - `IDXGIFactory*`
/// - `ID3D11Device*`
/// - `ID3D11DeviceContext*`
/// - текущий `M2DE_SwapChainWrapper*`
///
/// Цепочка до raw swapchain:
/// `RENDER_DEVICE -> current_swapchain -> swapchain`
///
/// IDA: `qword_141CD5D18`
pub const RENDER_DEVICE: usize = 0x1CD_5D18;

/// Флаг особого режима окружения / remote session.
///
/// Используется рендером при выборе ограничения на размер текстур.
///
/// IDA: `byte_141CD5CF2`
pub const RENDER_IS_REMOTE_SESSION: usize = 0x1CD_5CF2;

/// Базовый лимит размеров текстур.
///
/// IDA: `dword_141C34DD0`
pub const RENDER_DEFAULT_MAX_TEXTURE_SIZE: usize = 0x1C3_4DD0;

/// Лимит размеров текстур для обычного локального запуска.
///
/// IDA: `dword_141C3589C`
pub const RENDER_MAX_TEXTURE_SIZE_LOCAL: usize = 0x1C3_589C;

/// Лимит размеров текстур для special/remote режима.
///
/// IDA: `dword_141C358A0`
pub const RENDER_MAX_TEXTURE_SIZE_REMOTE: usize = 0x1C3_58A0;

/// `M2DE_g_CameraManager` — статический объект камерной системы.
///
/// ⚠️ Важно: это **НЕ указатель** на объект, а сам объект,
/// размещённый прямо в `.bss` секции игры.
///
/// То есть доступ такой:
/// ```ignore
/// let camera_mgr = module_base + CAMERA_MANAGER;
/// ```
///
/// А НЕ такой:
/// ```ignore
/// let ptr = *(module_base + CAMERA_MANAGER as *const usize); // НЕПРАВИЛЬНО
/// ```
///
/// Внутри объекта лежат:
/// - `Interier` PlayerCameraView по `+0x0000`
/// - `Exterier` PlayerCameraView по `+0x0D18`
/// - `TransitSpeed` по `+0x1A30` / `+0x1A34`
/// - конфиги автомобильных камер по диапазону `+0x21D8 .. +0x2784`
///
/// Полный layout см. в `addresses::fields::camera_manager`.
///
/// Reverse source:
/// - `M2DE_CameraSystem_Init` (`0x141008230`)
/// - `M2DE_CameraManager_LoadConfigById` (`0x140E767E0`)
/// - все вызовы идут с `rcx = &unk_1431430F0`
///
/// IDA: `0x1431430F0` (`M2DE_g_CameraManager`)
pub const CAMERA_MANAGER: usize = 0x314_30F0;

/// `g_M2DE_PlayerData` — глобальная структура настроек здоровья игрока.
///
/// Важно: это НЕ указатель — прямой объект в `.data` секции.
/// Доступ: `module_base + PLAYER_DATA`, без разыменования.
///
/// Содержит:
/// - `+0x00`: float healthmax (default 520.0, runtime 720.0 на нормальной сложности)
/// - `+0x04`: float healthmaxvar (200.0)
/// - `+0x14`: float healthrestoredown_raw (~0.0175, Lua возвращает *100)
/// - `+0x18`: float boostfall (2.0)
/// - `+0x1C`: float healthmax_threshold (520.0, для статистики)
///
/// Максимум здоровья игрока читается из +0x00, а НЕ из entity+0x14C.
/// NPC используют entity+0x14C для своего healthmax.
///
/// IDA: `0x141CA1B38`
pub const PLAYER_DATA: usize = 0x1CA_1B38;

/// `g_M2DE_PhysicsWorldManager` — менеджер физического мира.
///
/// Важно: Двойная косвенность: *(module_base + RVA) → PhysicsWorldManager*
///
/// Содержит список физических объектов.
/// Используется PlayerActor_GetPosition в режиме 3.
///
/// IDA: `0x141CABDC8`
pub const PHYSICS_WORLD_MANAGER: usize = 0x1CA_BDC8;

/// `M2DE_g_EntityDatabase` — глобальная БД всех entity.
///
/// Двойная косвенность: *(module_base + RVA) → EntityDB*
/// Содержит все entity загруженные через SDS.
/// Используется GetEntityByName и GetEntityByGUID.
///
/// IDA: `0x141CAF788`
pub const ENTITY_DATABASE: usize = 0x1CA_F788;

/// `M2DE_g_EntityWrapperFactoryRegistry` — фабрики script wrappers.
///
/// RB-tree, ключ = entity type (uint8).
/// Создаёт C_WrapperHuman, C_WrapperCar и т.д.
/// Lazy-init при первом обращении.
///
/// IDA: `0x14313C8B8`
pub const ENTITY_WRAPPER_FACTORY_REGISTRY: usize = 0x313_C8B8;

/// `M2DE_g_ScriptWrapperManager` — менеджер Lua script wrappers для entity.
///
/// Двойная косвенность: *(module_base + RVA) → ScriptWrapperManager*
///
/// Lazy-init singleton. Используется для:
/// - `GetEntityByName` (FNV-1a hash → wrapper)
/// - `GetEntityByGUID` (tableID → wrapper)
/// - `CreateCleanEntity` (создание wrapper)
/// - Кеширование wrappers
///
/// Layout:
/// +0x08..+0x10: hash cache (sorted array, 16b/entry: hash + wrapper_ptr)
/// +0x28..+0x30: tableID cache (sorted array, 16b/entry: tableID + wrapper_ptr)
///
/// IDA: `0x1431360F8` (`M2DE_g_ScriptWrapperManager`)
pub const SCRIPT_WRAPPER_MANAGER: usize = 0x313_60F8;

/// `M2DE_g_SDSManager` — глобальный менеджер SDS системы.
///
/// ⚠️ Двойная косвенность: *(module_base + RVA) → SDSManager*
///
/// Используется для:
/// - ActivateStreamMapLine(name)
/// - GetSyncObjectForLoadSDS(name)
/// - LoadCityShop / ReleaseCityShop
/// - LoadCityPart / ReleaseCityPart
///
/// Layout (SDSManager+0x08 = loader context):
/// loader+0x08: int32 current_load_index
/// loader+0x10..0x18: loaded slots array
/// loader+0x18..0x20: map line name cache (sorted, 24b/entry)
///
/// IDA: `0x141CAF758`
pub const SDS_MANAGER: usize = 0x1CA_F758;

/// `M2DE_g_GfxEnvEffSystem` — графика/погода/эффекты окружения.
///
/// ⚠️ Двойная косвенность.
///
/// Layout:
/// +0x18: WeatherDataStore*
/// +0x20: WeatherSystem*
/// +0x28: DateTimeBuffers*
/// +0x60: WeatherRenderParams*
///
/// Используется SDS loader для проверки ночного режима (z-suffix).
///
/// IDA: `0x141CB2340`
pub const GFX_ENV_EFF_SYSTEM: usize = 0x1CB_2340;

/// `qword_141CAF7B0` — SDS Line Manager (загруженные stream map lines).
///
/// ⚠️ Двойная косвенность.
///
/// Layout:
/// +0x00..+0x08: loaded configs array (ptrs to config structs)
/// +0x18: cache_begin (sorted by hash, 24b/entry)
/// +0x20: cache_end
///
/// IDA: `0x141CAF7B0`
pub const SDS_LINE_MANAGER: usize = 0x1CA_F7B0;

/// `qword_141CAF760` — SDS Observer Manager.
///
/// ⚠️ Двойная косвенность.
///
/// +0x78..+0x88: observer list (vector of module ptrs)
///
/// IDA: `0x141CAF760`
pub const SDS_OBSERVER_MANAGER: usize = 0x1CA_F760;

/// `M2DE_g_CarManager` — основной менеджер машин/SDS.
/// ⚠️ Двойная косвенность.
/// Первый аргумент LoadFile_Core. Содержит loaded SDS slots.
/// Layout: +0x20..+0x28 = loaded slots, +0x38 = dirty flag.
/// 8 callbacks registered in constructor.
/// IDA: `0x141CAF7D8`
pub const CAR_MANAGER: usize = 0x1CA_F7D8;

/// `M2DE_g_TypeRegistry` — linked list дескрипторов entity types.
/// node+0x00: next ptr, node+0x08: typeId (int32), node+0x18: createFn ptr.
/// Используется M2DE_TypeRegistry_CreateByTypeId для аллокации entity.
/// IDA: `0x141CAE228`
pub const TYPE_REGISTRY: usize = 0x1CA_E228;