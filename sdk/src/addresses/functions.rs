//! Адреса функций (RVA).
//!
//! Calling convention: Microsoft x64 (`extern "C"` в Rust).
//! - RCX = 1-й аргумент (ptr/int) или XMM0 (float)
//! - RDX = 2-й аргумент или XMM1
//! - R8  = 3-й аргумент или XMM2
//! - R9  = 4-й аргумент или XMM3

//=============================================================================
//  Engine System
//=============================================================================

pub mod engine {
    /// Точка входа игры.
    /// IDA: `0x1412CCDC0`
    pub const MAIN_ENTRY_POINT: usize = 0x12C_CDC0;

    /// Инициализация NVAPI.
    /// IDA: `0x140001090`
    pub const NVAPI_INITIALIZE: usize = 0x1090;

    /// Создание PhysX Core SDK.
    /// IDA: `0x14134DBD0`
    pub const PHYSX_CREATE_CORE_SDK: usize = 0x134_DBD0;

    /// Создание APEX SDK.
    /// IDA: `0x1411604F0`
    pub const APEX_CREATE_SDK: usize = 0x116_04F0;

    /// Инициализация DirectX графики.
    /// IDA: `0x140A8B2A0`
    pub const DIRECTX_GRAPHICS_INIT: usize = 0xA8_B2A0;

    /// Рендеринг сцены.
    /// IDA: `0x1402DC7D0`
    pub const SCENE_TRAVERSE_RENDER: usize = 0x2D_C7D0;

    /// Инициализация аудио.
    /// IDA: `0x140B8B5E0`
    pub const AUDIO_SYSTEM_INIT: usize = 0xB8_B5E0;

    /// Обновление игровых объектов.
    /// IDA: `0x140D8B7A0`
    pub const GAME_OBJECTS_UPDATE: usize = 0xD8_B7A0;

    /// Обработчик AI событий.
    /// IDA: `0x140DD31B0`
    pub const AI_EVENT_HANDLER: usize = 0xDD_31B0;

    /// Обработчик AI навигации.
    /// IDA: `0x140DD5690`
    pub const AI_NAVIGATION_HANDLER: usize = 0xDD_5690;
}

//=============================================================================
//  Player TODO: нужно полностью пересмотреть и отрефакторить этот модуль, он сейчас в очень сыром виде
//=============================================================================

pub mod player {
    // WrapperPlayer-level

    /// ⚠️ Принимает C_WrapperPlayer*, не C_Human*!
    /// `void(WrapperPlayer*, float cents)`
    /// IDA: `0x1410C9520`
    pub const WRAPPER_ADD_MONEY: usize = 0x10C_9520;

    /// ⚠️ Принимает C_WrapperPlayer*, не C_Human*!
    /// `float(WrapperPlayer*)`
    /// IDA: `0x1410C96B0`
    pub const WRAPPER_GET_MONEY: usize = 0x10C_96B0;

    /// ⚠️ Принимает C_WrapperPlayer*, не C_Human*!
    /// `void(WrapperPlayer*, u32 weapon_id, u32 ammo)`
    /// IDA: `0x1410C9560`
    pub const WRAPPER_ADD_WEAPON: usize = 0x10C_9560;

    // Inventory-level (принимает Inventory*)
    // ⚠️ Вызывать ТОЛЬКО из игрового потока!

    /// `i64(Inventory*)` — получить центы
    /// IDA: `0x140DD4AC0`
    pub const INVENTORY_GET_MONEY_CENTS: usize = 0xDD_4AC0;

    /// `char(Inventory*, i64 cents, u8 do_apply)` — тихое добавление
    /// IDA: `0x140D7E7D0`
    pub const INVENTORY_MODIFY_MONEY: usize = 0xD7_E7D0;

    /// `char(Inventory*, i64 cents)` — добавление + HUD
    /// IDA: `0x140D7E8D0`
    pub const INVENTORY_ADD_MONEY_NOTIFY: usize = 0xD7_E8D0;

    // Slot/Core level

    /// `void(MoneySlot*, i64 cents)` — добавить в слот
    /// IDA: `0x140D7E800`
    pub const MONEY_SLOT_ADD: usize = 0xD7_E800;

    /// `void(MoneyCore*, i64 new_cents)` — установить + notify
    /// IDA: `0x140DCE920`
    pub const MONEY_CORE_SET: usize = 0xDC_E920;

    // Weapon — inventory-level

    /// `char(Inventory*, u32 weapon_id, i32 ammo)` — добавить оружие.
    ///
    /// Логика:
    /// - Если оружие уже есть -> добавляет только патроны
    /// - Если нового типа -> создаёт WeaponItem, пробует slot[2] и slot[3]
    /// - Патроны капаются на max из tables/weapons.tbl
    ///
    /// Строка: `"C_HumanInventory::AddWeapon(int, int)"`
    ///
    /// IDA: `0x140D7EF30`
    pub const INVENTORY_ADD_WEAPON_CORE: usize = 0xD7_EF30;

    /// `void(Inventory*, u32 weapon_id, u32 ammo)` — добавить патроны.
    ///
    /// Оперирует на slot[4] (ammo slot).
    /// Строка: `"C_HumanInventory::AddAmmo()"`
    ///
    /// IDA: `0x140D7D590`
    pub const INVENTORY_ADD_AMMO: usize = 0xD7_D590;

    /// Проверяет, ограничено ли оружие для игрока.
    ///
    /// `bool(u32 weapon_id)` — true = нельзя добавить в главе.
    ///
    /// IDA: `0x1410C9FD0`
    pub const IS_WEAPON_RESTRICTED: usize = 0x10C_9FD0;
}

pub mod player_control {
    /// `bool(PlayerControlRef*)`
    /// IDA: `0x140D809C0`
    pub const IS_LOCKED: usize = 0xD8_09C0;

    /// `bool(PlayerControlRef*, u8 locked, u8 play_anim_flag)`
    /// IDA: `0x140DB1B40`
    ///
    /// ВАЖНО: Эта функция проверяет текущее состояние через IS_LOCKED
    /// и ничего не делает, если состояние уже соответствует запрошенному.
    /// Для принудительной блокировки используйте SET_LOCKED_INTERNAL.
    pub const SET_LOCKED: usize = 0xDB_1B40;

    /// `i64(control_component+112, u8 locked, u8 flags)`
    /// IDA: `0x140DB1BE0`
    ///
    /// Внутренняя функция блокировки управления, вызываемая из SET_LOCKED.
    /// Принимает указатель на (control_component + 112), а не на сам компонент!
    /// Всегда выполняет блокировку/разблокировку, не проверяя текущее состояние.
    pub const SET_LOCKED_INTERNAL: usize = 0xDB_1BE0;

    /// `const char*(PlayerControlRef*)`
    /// IDA: `0x140DCCEB0`
    pub const GET_STYLE_STR: usize = 0xDC_CEB0;

    /// `bool(PlayerControlRef*, const char* style)`
    /// IDA: `0x140DCD0B0`
    pub const SET_STYLE_STR: usize = 0xDC_D0B0;

    /// `bool(PlayerControlRef*, u32 style_id)`
    /// IDA: `0x140DCD0D0`
    pub const SET_FIGHT_CONTROL_STYLE: usize = 0xDC_D0D0;

    /// `bool(PlayerControlRef*, u8 enabled, u32 hint_id)`
    /// IDA: `0x140DCD0F0`
    pub const SET_FIGHT_HINT: usize = 0xDC_D0F0;
}

// =============================================================================
//  Garage
// =============================================================================

pub mod garage {
    /// Получить ID машины по имени (0–33).
    /// Регистрирует все 34 машины.
    ///
    /// `i32(const char* name)` -> vehicle ID или -1
    ///
    /// IDA: `0x1410_1D0A0` (`M2DE_GetVehicleIDByName`)
    pub const GET_VEHICLE_ID_BY_NAME: usize = 0x101_D0A0;

    /// Добавить машину в гараж. Выделяет 184 байт для Vehicle.
    ///
    /// IDA: `0x1410_1CE40` (`M2DE_AddVehicleToGarage`)
    pub const ADD_VEHICLE_TO_GARAGE: usize = 0x101_CE40;

    /// Загрузить модель машины из ресурсов. Устанавливает state = 1.
    ///
    /// IDA: `0x1410_21ED0` (`M2DE_LoadVehicleModel`)
    pub const LOAD_VEHICLE_MODEL: usize = 0x102_1ED0;

    /// Установить цвет машины. Пишет colorID в vehicle+0xA4.
    ///
    /// IDA: `0x1410_283E0` (`M2DE_SetVehicleColor`)
    pub const SET_VEHICLE_COLOR: usize = 0x102_83E0;

    /// Регистрация гаража в Lua API.
    ///
    /// IDA: `0x1410_25CA0` (`M2DE_RegisterGarageLuaAPI`)
    pub const REGISTER_LUA_API: usize = 0x102_5CA0;

    /// Регистрация всех Lua функций гаража через vtable[7].
    ///
    /// IDA: `0x1410_28B60` (`M2DE_GarageManager_RegisterLuaAPI`)
    pub const MANAGER_REGISTER_LUA_API: usize = 0x102_8B60;

    /// Поиск ресурса по индексу (binary tree).
    ///
    /// IDA: `0x1401_85490` (`M2DE_FindResourceByIndex`)
    pub const FIND_RESOURCE_BY_INDEX: usize = 0x18_5490;

    /// Увеличить std::vector и вставить элемент.
    ///
    /// IDA: `0x1410_17850` (`M2DE_Vector_GrowAndInsert`)
    pub const VECTOR_GROW_AND_INSERT: usize = 0x101_7850;

    /// Копировать VehicleWrapper + ref count++.
    ///
    /// IDA: `0x1410_17D70` (`M2DE_VehicleWrapper_CopyAndAddRef`)
    pub const WRAPPER_COPY_ADD_REF: usize = 0x101_7D70;

    /// Копировать диапазон элементов в vector.
    ///
    /// IDA: `0x1410_2BEC0` (`M2DE_Vector_CopyRange`)
    pub const VECTOR_COPY_RANGE: usize = 0x102_BEC0;
}

// =============================================================================
//  Managers
// =============================================================================

pub mod managers {
    /// Получить ResourceManager*.
    ///
    /// IDA: `0x1401_1FD10` (`M2DE_GetResourceManager`)
    pub const GET_RESOURCE_MANAGER: usize = 0x11_FD10;

    /// Получить GameCallbackManager*.
    ///
    /// IDA: `0x1403_AEEC0` (`M2DE_GetGameCallbackManager`)
    pub const GET_GAME_CALLBACK_MANAGER: usize = 0x3A_EEC0;
}

// =============================================================================
//  HUD
// =============================================================================

pub mod hud {
    /// Показать popup "± $ X.XX" на экране.
    ///
    /// `void(HudMoneyComponent*, i64 cents, char flag)`
    /// IDA: `0x140D45B50`
    pub const SHOW_MONEY_POPUP: usize = 0xD4_5B50;

    /// Обновить счётчик денег в HUD + вызвать popup.
    ///
    /// `i64(HudMoneyDisplay*, i64 cents_delta, i64 unused)`
    /// Popup показывается только если `*(component + 0x5C) <= 0.0`
    ///
    /// IDA: `0x140D23600`
    pub const UPDATE_MONEY_COUNTER: usize = 0xD2_3600;

    /// Загрузить иконку HUD (также используется как FNV-1a хеш).
    ///
    /// IDA: `0x140A76940`
    pub const LOAD_ICON: usize = 0xA7_6940;

    /// Обработчик ввода для интерактивной карты.
    ///
    /// `u64(MapContext*, InputEvent*, float dt)`
    ///
    /// Функционал:
    /// - Зум колёсиком/клавишами (D=68, K=75)
    /// - Панорамирование WASD-подобными клавишами
    /// - Перетаскивание мышью (drag: состояния 0=idle, 1=click, 2=dragging)
    /// - Выбор маркеров на карте
    /// - Ограничения зума (0.5x - 2.5x)
    ///
    /// IDA: `0x1400FE640` (`M2DE_MapInputHandler`)
    pub const MAP_INPUT_HANDLER: usize = 0xFE_640;
}

pub mod entity {
    /// Получить текущую мировую позицию entity.
    ///
    /// Сигнатура:
    /// `Vec3* (Entity* entity, Vec3* out)`
    ///
    /// Что делает функция:
    /// - если у entity есть physics/provider по `entity + 0x258`,
    ///   вызывает его virtual method (`vtable + 0xA8`) и пишет позицию в `out`
    /// - иначе использует fallback через frame/transform node:
    ///   `entity + 0x78` -> frame
    ///   - `frame + 0x64` = x
    ///   - `frame + 0x74` = y
    ///   - `frame + 0x84` = z
    ///
    /// Важно:
    /// это именно getter текущего состояния.
    /// Функция не "двигает" entity и не участвует в tick update.
    ///
    /// IDA: `0x140DA7630`
    pub const GET_POS: usize = 0xDA_7630;

    /// `void(Entity*, const Vec3*)` — установить мировую позицию entity.
    ///
    /// Это high-level setter движка:
    /// - пишет позицию в frame node
    /// - синкает physics (`entity + 0x258`) если есть
    /// - обновляет вспомогательные transform/cache структуры
    /// - выставляет dirty/invalid flags
    ///
    /// Цепочка подтверждения:
    /// `Lua SetPos -> wrapper thunk -> wrapper impl (0x1410ADC60)
    ///  -> entity vtable +0x100 -> thunk 0x1400C9950 -> this function`
    ///
    /// IDA: `0x140DD1000`
    pub const SET_POS: usize = 0xDD_1000;

    /// Низкоуровневая запись позиции только в frame/transform node.
    ///
    /// Обычно напрямую вызывать не нужно.
    /// Для SDK предпочтителен `SET_POS`, так как он обновляет и другие подсистемы.
    ///
    /// IDA: `0x1403B9660`
    pub const SET_POS_RAW: usize = 0x3B_9660;
}

// =============================================================================
//  Tables
// =============================================================================

pub mod tables {
    /// Конструктор TableManager — загружает все .tbl файлы.
    ///
    /// IDA: `0x1400_F1480` (`M2DE_TableManager_Constructor`)
    pub const CONSTRUCTOR: usize = 0xF_1480;

    /// Загрузка /tables/vehicles.tbl -> TableManager+0x60.
    ///
    /// IDA: `0x1400_DEAD0`
    pub const LOAD_VEHICLES: usize = 0xD_EAD0;

    /// Загрузка /tables/weapons.tbl -> TableManager+0x40.
    ///
    /// IDA: `0x1400_DEBE0`
    pub const LOAD_WEAPONS: usize = 0xD_EBE0;

    /// Загрузка /tables/police_offences.tbl -> TableManager+0x38.
    ///
    /// IDA: `0x1400_DE140`
    pub const LOAD_POLICE_OFFENCES: usize = 0xD_E140;

    /// Загрузка /tables/attack_params.tbl -> TableManager+0x50.
    ///
    /// IDA: `0x1400_DC490`
    pub const LOAD_ATTACK_PARAMS: usize = 0xD_C490;

    /// Получить загрузчик ресурсов.
    ///
    /// IDA: `0x1401_85480`
    pub const GET_RESOURCE_LOADER: usize = 0x18_5480;
}

// =============================================================================
//  Profiling
// =============================================================================

pub mod profiling {
    /// IDA: `0x1404_0E7F0`
    pub const BEGIN_PROFILE: usize = 0x40_E7F0;
    /// IDA: `0x1404_130D0`
    pub const END_PROFILE: usize = 0x41_30D0;
    /// IDA: `0x1404_11BE0`
    pub const CREATE_PROFILE: usize = 0x41_1BE0;
}

// =============================================================================
//  Lua Bindings (НЕ вызывать напрямую — принимают lua_State*)
// =============================================================================

pub mod lua_bindings {
    /// IDA: `0x140C_6FF70` (`M2DE_Lua_Game_GetActivePlayer`)
    pub const GET_ACTIVE_PLAYER: usize = 0xC6_FF70;
    /// IDA: `0x1410_B5230` (`M2DE_Lua_Player_InventoryAddWeapon`)
    pub const INVENTORY_ADD_WEAPON: usize = 0x10B_5230;
    /// IDA: `0x1410_B5B70` (`M2DE_Lua_Player_InventoryGetMoney`)
    pub const INVENTORY_GET_MONEY: usize = 0x10B_5B70;
    /// IDA: `0x1410_B5BB0` (`M2DE_Lua_Player_InventoryAddMoney`)
    pub const INVENTORY_ADD_MONEY: usize = 0x10B_5BB0;
    /// IDA: `0x1407_B1A50` (`M2DE_Lua_GetPlayerFromStack`)
    pub const GET_PLAYER_FROM_STACK: usize = 0x7B_1A50;
}

// =============================================================================
//  GameCallbackManager VTable Methods
// =============================================================================

pub mod callback_manager {
    /// Деструктор.
    /// IDA: `0x1403_9FC70`
    pub const DESTRUCTOR: usize = 0x39_FC70;
    /// GetSize() -> 8.
    /// IDA: `0x1403_AC3D0`
    pub const GET_SIZE: usize = 0x3A_C3D0;
    /// RegisterFunction.
    /// IDA: `0x1403_A06D0`
    pub const REGISTER_FUNCTION: usize = 0x3A_06D0;
}

// =============================================================================
//  GarageManager VTable Methods
// =============================================================================

pub mod garage_manager_methods {
    /// GetSize() -> 7.
    /// IDA: `0x1400_A78D0`
    pub const GET_SIZE: usize = 0xA_78D0;
    /// GetClassName() -> "C_GarageManager".
    /// IDA: `0x1410_22F70`
    pub const GET_CLASS_NAME: usize = 0x102_2F70;
    /// GetSomeFloat() -> 0.005f.
    /// IDA: `0x1400_A7D80`
    pub const GET_FLOAT: usize = 0xA_7D80;
    /// Unknown method.
    /// IDA: `0x1410_272D0`
    pub const METHOD1: usize = 0x102_72D0;
}

// =============================================================================
//  Raw Lua API (Mafia II: DE Lua 5.1.2, modified)
// =============================================================================

pub mod lua {
    /// `int(lua_State* L, const char* buff, size_t sz, const char* name, int extra)`
    ///
    /// В этой игре wrapper не стандартный: есть 5-й параметр.
    /// Всегда передаём `0`.
    ///
    /// IDA: `0x1405CD470`
    pub const LOADBUFFER: usize = 0x5CD_470;

    /// `int(lua_State* L, const char* s)`
    ///
    /// IDA: `0x1405CD6A0`
    pub const LOADSTRING: usize = 0x5CD_6A0;

    /// `int(lua_State* L, int nargs, int nresults, int errfunc)`
    ///
    /// IDA: `0x1405CB600`
    pub const PCALL: usize = 0x5CB_600;

    /// `const char*(lua_State* L, int idx, size_t* len)`
    ///
    /// IDA: `0x1405CC130`
    pub const TOLSTRING: usize = 0x5CC_130;

    /// `void(lua_State* L, int idx)`
    ///
    /// IDA: `0x1405CBF50`
    pub const SETTOP: usize = 0x5CB_F50;

    /// `int(lua_State* L)`
    ///
    /// IDA: `0x1405CB230`
    pub const GETTOP: usize = 0x5CB_230;

    /// `void(lua_State* L, const char* s)`
    ///
    /// IDA: `0x1405CB8D0`
    pub const PUSHSTRING: usize = 0x5CB_8D0;
}

pub mod script_machine {
    /// `bool ScriptMachine::CallString(this, code_like_input)`
    ///
    /// Для Lua-консоли лучше НЕ использовать как основной путь.
    ///
    /// IDA: `0x140A1C530`
    pub const CALL_STRING: usize = 0xA1C_530;
}

// =============================================================================
// Callbacks
// =============================================================================

pub mod callbacks {
    /// Регистрирует новый тип callback-события в `GameCallbackManager`.
    ///
    /// Аргументы:
    /// - `this`
    /// - `event_id`
    /// - `event_name`
    ///
    /// Отклоняет дубликаты по ID и имени.
    ///
    /// IDA: `0x1403A08F0`
    pub const REGISTER_EVENT_TYPE: usize = 0x3A_08F0;

    /// Регистрирует callback function на событие.
    ///
    /// Аргументы примерно такие:
    /// - `this`
    /// - `event_id`
    /// - `priority`
    /// - `callback_object`
    /// - `callback_function`
    /// - `float_param`
    /// - `config_mask`
    /// - `int_param`
    ///
    /// IDA: `0x1403A06D0`
    pub const REGISTER_FUNCTION: usize = 0x3A_06D0;

    /// Public fire path: запускает одно событие по `event_id`.
    ///
    /// Используется как удобная lifecycle hook-точка, потому что через неё
    /// проходят такие события как:
    /// - `No Game Start`
    /// - `Mission Before Open`
    /// - `Mission After Open`
    /// - `Game Paused`
    /// - `Game Unpaused`
    ///
    /// IDA: `0x1403A15E0`
    pub const FIRE_EVENT_BY_ID: usize = 0x3A_15E0;

    /// Внутренний dispatcher одного события по уже найденному индексу descriptor’а.
    ///
    /// Делает:
    /// - lock `in_dispatch`
    /// - строит временный список callback’ов
    /// - вызывает callback’и
    /// - flush pending ops
    ///
    /// IDA: `0x1403A16A0`
    pub const DISPATCH_SINGLE_EVENT_BY_INDEX: usize = 0x3A_16A0;

    /// Внутренний multi-event dispatcher.
    ///
    /// В runtime чаще всего видны batch'и:
    /// - `Game Tick Always`
    /// - `Game Render`
    ///
    /// IDA: `0x1403A1A00`
    pub const DISPATCH_EVENTS_INTERNAL: usize = 0x3A_1A00;

    /// Добавляет отложенную операцию в pending queue callback manager’а.
    ///
    /// Используется, когда callback list нельзя менять прямо сейчас,
    /// потому что событие уже dispatch’ится.
    ///
    /// IDA: `0x1403A0A90`
    pub const QUEUE_PENDING_FUNCTION_OP: usize = 0x3A_0A90;

    /// Снимает один callback:
    /// - по `event_id`
    /// - `callback_object`
    /// - `callback_function`
    ///
    /// Специальный случай: `event_id == -1` -> удалить из всех событий.
    ///
    /// IDA: `0x1403A55A0`
    pub const UNREGISTER_FUNCTION: usize = 0x3A_55A0;

    /// Снимает все callback’и, принадлежащие одному `callback_object`.
    ///
    /// IDA: `0x1403A57E0`
    pub const UNREGISTER_FUNCTIONS_BY_OBJECT: usize = 0x3A_57E0;

    /// Изменяет флаги callback entry.
    ///
    /// Находит callback по:
    /// - `event_id`
    /// - `callback_object`
    /// - `callback_function`
    ///
    /// Затем обновляет байт флагов.
    ///
    /// IDA: `0x1403A64F0`
    pub const SET_FUNCTION_FLAGS: usize = 0x3A_64F0;

    /// Ищет `event_id` по имени события (case-insensitive).
    ///
    /// Возвращает `-1`, если имя не найдено.
    ///
    /// IDA: `0x1403AD080`
    pub const GET_EVENT_ID_BY_NAME: usize = 0x3A_D080;

    /// Flush deferred add/remove операций после завершения dispatch.
    ///
    /// IDA: `0x1403B4AD0`
    pub const FLUSH_PENDING_FUNCTION_OPS: usize = 0x3B_4AD0;

    /// Удобная main-thread hook-точка:
    /// существующий callback из события `Game Tick Always`.
    ///
    /// Runtime подтверждение:
    /// - event id = 5
    /// - callback вызывается на главном игровом потоке
    ///
    /// IDA: `0x14015B5F0`
    pub const GAME_TICK_ALWAYS_CB_CANDIDATE: usize = 0x15_B5F0;
}

pub mod human_messages {
    /// Lua-диспетчер метода:
    /// `C_WrapperEntity:RegisterToMessages(...)`
    ///
    /// Поддерживает overload'ы:
    /// - `RegisterToMessages(guid)`
    /// - `RegisterToMessages(guid, event_type)`
    /// - `RegisterToMessages(guid, event_type, message_id)`
    ///
    /// IDA: `0x1410AD030`
    pub const LUA_REGISTER_TO_MESSAGES: usize = 0x10A_D030;

    /// Lua-диспетчер метода:
    /// `C_WrapperEntity:UnregisterFromMessages(...)`
    ///
    /// Поддерживает overload'ы:
    /// - `UnregisterFromMessages(guid)`
    /// - `UnregisterFromMessages(guid, event_type)`
    /// - `UnregisterFromMessages(guid, event_type, message_id)`
    ///
    /// IDA: `0x1410AD410`
    pub const LUA_UNREGISTER_FROM_MESSAGES: usize = 0x10A_D410;

    /// Lua-overload helper:
    /// `RegisterToMessages(guid)`
    ///
    /// Читает `C_EntityGuid` из Lua аргумента #2 и
    /// форвардит в native wrapper path.
    ///
    /// IDA: `0x1410BB620`
    pub const LUA_REGISTER_GUID: usize = 0x10B_B620;

    /// Lua-overload helper:
    /// `RegisterToMessages(guid, event_type)`
    ///
    /// Читает:
    /// - `guid` из Lua arg #2
    /// - `event_type` из Lua arg #3
    ///
    /// Затем форвардит в wrapper-side register helper,
    /// где `message_id = 0`.
    ///
    /// IDA: `0x1410BB6E0`
    pub const LUA_REGISTER_GUID_EVENT_TYPE: usize = 0x10B_B6E0;

    /// Lua-overload helper:
    /// `RegisterToMessages(guid, event_type, message_id)`
    ///
    /// Читает:
    /// - `guid` из Lua arg #2
    /// - `event_type` из Lua arg #3
    /// - `message_id` из Lua arg #4
    ///
    /// IDA: `0x1410BB820`
    pub const LUA_REGISTER_GUID_EVENT_TYPE_MESSAGE: usize = 0x10B_B820;

    /// Wrapper-side helper:
    /// `RegisterToMessages(guid, 0, 0)`
    ///
    /// То есть подписка на все сообщения выбранной сущности.
    ///
    /// IDA: `0x1410CDE70`
    pub const WRAPPER_REGISTER_GUID: usize = 0x10C_DE70;

    /// Wrapper-side helper:
    /// `RegisterToMessages(guid, event_type, 0)`
    ///
    /// То есть подписка на все сообщения конкретного event_type
    /// для выбранной сущности.
    ///
    /// IDA: `0x1410CDE80`
    pub const WRAPPER_REGISTER_GUID_EVENT_TYPE: usize = 0x10C_DE80;

    /// Native wrapper-side register implementation.
    ///
    /// Проверяет валидность wrapper object, затем форвардит
    /// в engine-level entity message registry registration path.
    ///
    /// IDA: `0x1410CDE90`
    pub const WRAPPER_REGISTER_IMPL: usize = 0x10C_DE90;

    /// Engine-level регистрация подписки в entity message registry.
    ///
    /// Логическая сигнатура:
    /// - listener_owner
    /// - target_entity_guid
    /// - event_type  (`0 = любой`)
    /// - message_id  (`0 = любой`)
    ///
    /// Перегрузки по смыслу:
    /// - `guid only`                    -> подписка на все сообщения сущности
    /// - `guid + event_type`           -> подписка на все сообщения данного типа
    /// - `guid + event_type + message` -> подписка на одно конкретное сообщение
    ///
    /// IDA: `0x1403B7420`
    pub const REGISTRY_REGISTER: usize = 0x3B_7420;

    /// Вспомогательная функция обратного учёта target guid.
    ///
    /// Добавляет или увеличивает refcount для target_entity_guid
    /// во вторичном дереве учёта у listener owner.
    ///
    /// IDA: `0x1403A0D30`
    pub const ADD_TARGET_GUID_REF: usize = 0x3A_0D30;

    /// Lua-overload helper:
    /// `UnregisterFromMessages(guid)`
    ///
    /// Удаляет все подписки на сообщения выбранной сущности.
    ///
    /// IDA: `0x1410BB680`
    pub const LUA_UNREGISTER_GUID: usize = 0x10B_B680;

    /// Lua-overload helper:
    /// `UnregisterFromMessages(guid, event_type)`
    ///
    /// Удаляет все подписки на сообщения данного event_type
    /// для выбранной сущности.
    ///
    /// IDA: `0x1410BB780`
    pub const LUA_UNREGISTER_GUID_EVENT_TYPE: usize = 0x10B_B780;

    /// Lua-overload helper:
    /// `UnregisterFromMessages(guid, event_type, message_id)`
    ///
    /// Удаляет подписку на одно конкретное сообщение.
    ///
    /// IDA: `0x1410BB910`
    pub const LUA_UNREGISTER_GUID_EVENT_TYPE_MESSAGE: usize = 0x10B_B910;

    /// Wrapper-side helper:
    /// `UnregisterFromMessages(guid, 0, 0)`
    ///
    /// То есть снятие всех подписок для target guid.
    ///
    /// IDA: `0x1410D2320`
    pub const WRAPPER_UNREGISTER_GUID: usize = 0x10D_2320;

    /// Wrapper-side helper:
    /// `UnregisterFromMessages(guid, event_type, 0)`
    ///
    /// То есть снятие всех подписок одного event_type
    /// для target guid.
    ///
    /// IDA: `0x1410D2330`
    pub const WRAPPER_UNREGISTER_GUID_EVENT_TYPE: usize = 0x10D_2330;

    /// Native wrapper-side unregister implementation.
    ///
    /// Проверяет валидность wrapper object, затем форвардит
    /// в engine-level unregister path.
    ///
    /// IDA: `0x1410D2340`
    pub const WRAPPER_UNREGISTER_IMPL: usize = 0x10D_2340;

    /// Engine-level вход в unregister path.
    ///
    /// Ищет world/entity registry и передаёт управление
    /// в low-level unregister implementation.
    ///
    /// IDA: `0x1403BCCF0`
    pub const REGISTRY_UNREGISTER: usize = 0x3B_CCF0;

    /// Низкоуровневая реализация снятия подписки.
    ///
    /// Поддерживает варианты:
    /// - guid only
    /// - guid + event_type
    /// - guid + event_type + message_id
    ///
    /// При необходимости удаляет пустые внутренние узлы дерева.
    ///
    /// IDA: `0x1403BFA90`
    pub const REGISTRY_UNREGISTER_IMPL: usize = 0x3B_FA90;

    /// Путь десериализации / загрузки entity message registry из потока.
    ///
    /// Сначала очищает существующие подписки,
    /// затем восстанавливает guid -> event_type -> message tree.
    ///
    /// IDA: `0x1403A9D00`
    pub const REGISTRY_LOAD_FROM_STREAM: usize = 0x3A_9D00;
}

pub mod entity_messages {
    /// Центральный путь широковещательной доставки entity/human сообщений.
    ///
    /// Это основной delivery path, через который движок рассылает
    /// уже сконструированные сообщения подписчикам.
    ///
    /// Именно эту функцию удобно hook'ать для runtime-анализа
    /// сообщений локального игрока.
    ///
    /// IDA: `0x1403A6DB0`
    pub const BROADCAST: usize = 0x3A_6DB0;

    /// Включает или выключает entity-message listener object.
    ///
    /// По факту пишет bool-флаг в `listener + 0x3D8`.
    ///
    /// IDA: `0x1403B7110`
    pub const LISTENER_SET_ENABLED: usize = 0x3B_7110;

    /// Один из подтверждённых sender-path'ов human-сообщения,
    /// связанного с входом в транспорт / use-vehicle сценарием.
    ///
    /// По текущему reverse'у:
    /// конструирует и рассылает сообщение `ENTER_VEHICLE` (`0xD001B`).
    ///
    /// IDA: `0x140DA4C80`
    pub const HUMAN_SEND_ENTER_VEHICLE_LIKE: usize = 0xDA_4C80;
}

// =============================================================================
//  Render Device
// =============================================================================

pub mod render {
    /// Полный конструктор `M2DE_C_RenderDeviceD3D11`.
    ///
    /// IDA: `0x1409C36A0`
    pub const RENDER_DEVICE_D3D11_CTOR: usize = 0x9C_36A0;

    /// Промежуточный конструктор render-device цепочки.
    ///
    /// IDA: `0x1409C3E20`
    pub const RENDER_DEVICE_MID_CTOR: usize = 0x9C_3E20;

    /// Базовый конструктор render-device.
    ///
    /// Именно здесь игра сохраняет глобальный singleton
    /// в `qword_141CD5D18`.
    ///
    /// IDA: `0x1408CA120`
    pub const RENDER_DEVICE_BASE_CTOR: usize = 0x8C_A120;

    /// Teardown / cleanup path для `M2DE_C_RenderDeviceD3D11`.
    ///
    /// IDA: `0x1409C5040`
    pub const RENDER_DEVICE_D3D11_TEARDOWN: usize = 0x9C_5040;

    /// Scalar deleting destructor.
    ///
    /// IDA: `0x1409C60D0`
    pub const RENDER_DEVICE_D3D11_SCALAR_DTOR: usize = 0x9C_60D0;

    /// Возвращает строку `"D3D11 Rendering Device"`.
    ///
    /// IDA: `0x1409CBEA0`
    pub const RENDER_DEVICE_D3D11_GET_NAME: usize = 0x9C_BEA0;

    /// Основная DX11 инициализация.
    ///
    /// Создаёт:
    /// - `ID3D11Device`
    /// - `ID3D11DeviceContext`
    /// - DXGI factory
    /// - swapchain wrapper
    /// - базовые render states
    ///
    /// IDA: `0x1409CC4B0`
    pub const RENDER_DEVICE_D3D11_INIT: usize = 0x9C_C4B0;

    /// Создание swapchain wrapper и raw `IDXGISwapChain1`.
    ///
    /// Внутри вызывает `IDXGIFactory4::CreateSwapChainForHwnd`.
    ///
    /// IDA: `0x1409C9EE0`
    pub const CREATE_SWAPCHAIN: usize = 0x9C_9EE0;

    /// Создание RTV / SRV / depth texture / DSV для swapchain wrapper.
    ///
    /// IDA: `0x1409CD1A0`
    pub const SWAPCHAIN_WRAPPER_CREATE_VIEWS: usize = 0x9C_D1A0;

    /// Recreate/resize path для swapchain.
    ///
    /// Если HWND тот же — идёт resize path.
    /// Если HWND другой — создаётся новый wrapper/swapchain.
    ///
    /// IDA: `0x1409CE650`
    pub const RENDER_DEVICE_D3D11_RECREATE_SWAPCHAIN: usize = 0x9C_E650;

    /// Установка default render states.
    ///
    /// IDA: `0x14098E620`
    pub const RENDER_DEVICE_D3D11_SET_DEFAULT_STATES: usize = 0x98_E620;

    /// Создание dynamic VB/IB буферов.
    ///
    /// IDA: `0x140A12FC0`
    pub const RENDER_DEVICE_D3D11_CREATE_DYNAMIC_BUFFERS: usize = 0xA1_2FC0;

    /// Проверка debug device режима.
    ///
    /// В retail-сборке возвращает 0.
    ///
    /// IDA: `0x140A13AE0`
    pub const IS_DEBUG_DEVICE: usize = 0xA1_3AE0;

    /// Перечисление адаптеров DXGI.
    ///
    /// IDA: `0x1409CD2B0`
    pub const RENDER_DEVICE_D3D11_ENUM_ADAPTERS: usize = 0x9C_D2B0;

    /// Thunk на `D3D11CreateDevice`.
    ///
    /// IDA: `0x14153F09D`
    pub const D3D11_CREATE_DEVICE_THUNK: usize = 0x153_F09D;

    /// Thunk на `CreateDXGIFactory1`.
    ///
    /// IDA: `0x14153F0B5`
    pub const CREATE_DXGI_FACTORY1_THUNK: usize = 0x153_F0B5;
}

// =============================================================================
//  Camera System
// =============================================================================

pub mod camera {
    /// Инициализация всей камерной системы.
    /// Загружает все XML-конфиги, создаёт camera component.
    ///
    /// `void(SystemContext*)`
    ///
    /// IDA: `0x141008230`
    pub const SYSTEM_INIT: usize = 0x100_8230;

    /// Switch-диспетчер загрузки камерного конфига по ID.
    ///
    /// `__int64(CameraManager*, int config_id)`
    ///
    /// IDA: `0x140E767E0`
    pub const LOAD_CONFIG_BY_ID: usize = 0xE7_67E0;

    /// Загрузка playerCamera.xml.
    ///
    /// `void(CameraManager*, char use_defaults_only)`
    ///
    /// IDA: `0x140E75CC0`
    pub const LOAD_PLAYER_CAMERA: usize = 0xE7_5CC0;

    /// Парсинг одного CameraView (Interier/Exterier) из XML.
    ///
    /// `char(CameraManager*, XMLNode*, CameraView*)`
    ///
    /// IDA: `0x140E7D020`
    pub const VIEW_PARSE_FROM_XML: usize = 0xE7_D020;

    /// Generic загрузчик для car/other camera конфигов.
    ///
    /// `__int64(CameraManager*, const char* path, DWORD* params, DWORD* effects, const char* names, int count)`
    ///
    /// IDA: `0x140E755E0`
    pub const LOAD_GENERIC_CONFIG: usize = 0xE7_55E0;

    /// Копирование DefaultParams/Speeds во все 15 states.
    ///
    /// `void(CameraView*, int)`
    ///
    /// IDA: `0x140E6BC00`
    pub const VIEW_COPY_DEFAULTS_TO_STATES: usize = 0xE6_BC00;

    /// Основная функция обновления камеры.
    /// Читает mouse delta и обновляет rotation/position камеры.
    /// Вызывается каждый кадр.
    ///
    /// `void(Camera*, float dt, char flag)`
    ///
    /// IDA: `0x14029BAC0` (`M2DE_Camera_Update`)
    pub const UPDATE: usize = 0x29_BAC0;
}

// =============================================================================
//  DirectInput System
// =============================================================================

pub mod input {
    /// Инициализация DirectInput8 manager.
    /// Создаёт DirectInput8 context и сохраняет в input manager.
    ///
    /// `char(InputManager*, HWND)`
    ///
    /// IDA: `0x14079FA60` (`M2DE_InputManager_Init`)
    pub const MANAGER_INIT: usize = 0x79_FA60;

    /// Создание DirectInput8 устройств (клавиатура, мышь, геймпад).
    /// Вызывается из MANAGER_INIT.
    ///
    /// `char(InputManager*, HWND)`
    ///
    /// IDA: `0x14079F770` (`M2DE_InputManager_CreateDevices`)
    pub const MANAGER_CREATE_DEVICES: usize = 0x79_F770;

    /// Обновление состояния input устройства.
    /// Вызывает GetDeviceState через vtable+104.
    /// Вызывается каждый кадр для каждого устройства.
    ///
    /// `char(InputDevice*)`
    ///
    /// IDA: `0x1407A3EB0` (`M2DE_InputDevice_Update`)
    pub const DEVICE_UPDATE: usize = 0x7A_3EB0;

    /// Опрос DirectInput устройства.
    /// Вызывает vtable функции для получения текущего состояния.
    ///
    /// `char(InputDevice*)`
    ///
    /// IDA: `0x14079F560` (`M2DE_InputDevice_Poll`)
    pub const DEVICE_POLL: usize = 0x79_F560;

    /// Конструктор mouse input устройства.
    /// Инициализирует буферы состояния мыши.
    ///
    /// `__int64(MouseDevice*, InputManager*, CallbackObject*)`
    ///
    /// IDA: `0x14079A6C0` (`M2DE_MouseDevice_Constructor`)
    pub const MOUSE_DEVICE_CTOR: usize = 0x79_A6C0;

    /// Конструктор keyboard input устройства.
    /// Читает настройки клавиатуры Windows через SystemParametersInfo.
    ///
    /// `__int64(KeyboardDevice*, InputManager*, CallbackObject*)`
    ///
    /// IDA: `0x14079A890` (`M2DE_KeyboardDevice_Constructor`)
    pub const KEYBOARD_DEVICE_CTOR: usize = 0x79_A890;
}

// =============================================================================
//  Human / Health
// =============================================================================

pub mod human {
    /// `bool(C_Human*)` — is entity dead.
    /// vtable[47]. return *(uint8*)(this + 0x161).
    /// IDA: `0x1400C4690`
    pub const IS_DEATH: usize = 0x0C_4690;

    /// `char(C_Human*, EntityMessage*)` — process incoming damage.
    /// vtable[82]. Saves health, calls damage processor, handles death.
    /// IDA: `0x1400C00B0`
    pub const PROCESS_DAMAGE: usize = 0x0C_00B0;

    /// `char(C_Human*, EntityMessage*)` — core damage calculation.
    /// Guards: IsDeath() || invulnerability(+0x160).
    /// Subtracts from health(+0x148). Checks demigod(+0x162).
    /// IDA: `0x140D93B80`
    pub const APPLY_DAMAGE: usize = 0xD9_3B80;

    /// `void(C_Human*, EntityMessage*)` — death handler.
    /// Called when health <= 0 and !demigod.
    /// IDA: `0x140DD2460`
    pub const PROCESS_DEATH: usize = 0xDD_2460;

    /// `float(C_Human*)` — get current health.
    /// return *(float*)(*(component) + 0x148).
    /// IDA: `0x140DA3C30`
    pub const GET_HEALTH: usize = 0xDA_3C30;

    /// `float(ComponentRef*)` — get healthmax.
    /// Player: reads g_PlayerData+0x00. NPC: reads entity+0x14C.
    /// IDA: `0x140DA3C50`
    pub const GET_HEALTH_MAX: usize = 0xDA_3C50;

    /// `uint8(ComponentRef*)` — get invulnerability flag.
    /// return *(uint8*)(*(a1) + 0x160).
    /// IDA: `0x140DA5460`
    pub const GET_INVULNERABILITY: usize = 0xDA_5460;

    /// `void(ComponentRef*, uint8)` — set invulnerability flag.
    /// *(*(a1) + 0x160) = value.
    /// IDA: `0x140DD0300`
    pub const SET_INVULNERABILITY: usize = 0xDD_0300;

    /// `void*(void)` — get global player data singleton.
    /// Returns &g_M2DE_PlayerData.
    /// IDA: `0x1400C33C0`
    pub const GET_PLAYER_INSTANCE: usize = 0x0C_33C0;
}

//=============================================================================
//  Physics Provider
//=============================================================================

pub mod physics {
    /// Получить текущий physics state через property accessor.
    /// Внутри: *(player+0x258)->vtable[53]
    /// IDA: `0x140DCCEC0`
    pub const GET_PHYS_STATE: usize = 0xDC_CEC0;

    /// Установить physics state.
    /// Проверяет текущий, если отличается -> provider->vtable[52].
    /// IDA: `0x140DCD100`
    pub const SET_PHYS_STATE: usize = 0xDC_D100;
}

/// Значения PhysicsState
pub mod physics_state {
    pub const DYNAMIC: u32 = 0;
    pub const ENABLE: u32 = 1;
    pub const DISABLED: u32 = 2;
    pub const KINEMATIC: u32 = 3;
}

//=============================================================================
// Entity
//=============================================================================

pub mod entity_manager {
    /// Найти/создать wrapper по имени (FNV-1 64-bit хеш -> кеш -> БД -> factory).
    /// `__int64(ScriptWrapperManager*, const char* name)`
    /// Возвращает script wrapper или NULL.
    /// IDA: `0x1410C7070`
    pub const FIND_BY_NAME: usize = 0x10C_7070;

    /// Найти/создать wrapper по tableID.
    /// `__int64(ScriptWrapperManager*, uint32 tableID)`
    /// IDA: `0x1410C6E60`
    pub const GET_OR_CREATE_WRAPPER: usize = 0x10C_6E60;

    /// Хеш или парсинг имени. FNV-1 64-bit.
    /// `void(uint64* out, const char* name)`
    /// IDA: `0x140A76940`
    pub const PARSE_ID_OR_FNV1_64: usize = 0xA7_6940;

    /// Создать script wrapper из DB record.
    /// `__int64(ScriptWrapperManager*, EntityDBRecord*)`
    /// IDA: `0x1410CDC90`
    pub const CREATE_SCRIPT_WRAPPER: usize = 0x10C_DC90;

    /// Поиск в БД по tableID.
    /// `__int64(EntityDatabase*, uint32 tableID)`
    /// IDA: `0x1403E92A0`
    pub const DB_LOOKUP_BY_TABLE_ID: usize = 0x3E_92A0;

    /// Поиск в БД по FNV-1 64-bit name hash (vtable[8]).
    /// `__int64(EntityDatabase*, uint64 nameHash)`
    /// IDA: `0x1403E92E0`
    pub const DB_LOOKUP_BY_NAME_HASH: usize = 0x3E_92E0;

    /// Внутренний hash table lookup.
    /// `__int64(HashTable*, uint32 key24bit)`
    /// IDA: `0x1403E1C90`
    pub const HASH_TABLE_LOOKUP: usize = 0x3E_1C90;

    /// Master init — создаёт все менеджеры.
    /// IDA: `0x1403DD280`
    pub const INIT_ALL_MANAGERS: usize = 0x3D_D280;

    /// GetModuleNameById — switch/case для 49 модулей.
    /// `const char*(uint32 module_id)`
    /// IDA: `0x14044FB40`
    pub const GET_MODULE_NAME_BY_ID: usize = 0x44_FB40;

    /// C_ServiceIdentity::Init
    /// `void(ServiceIdentity*, uint32 module_id)`
    /// IDA: `0x1404444F0`
    pub const SERVICE_IDENTITY_INIT: usize = 0x44_44F0;
}

pub mod npc {
    /// Native Follow implementation.
    /// `void(PropertyAccessor*, SmartPtr* out, Entity* target, int speed,
    ///       float min_dist, float max_dist, bool flag1, bool flag2)`
    /// IDA: `0x140DC7B90`
    pub const FOLLOW_CORE: usize = 0xDC_7B90;

    /// Follow task constructor (96 bytes).
    /// IDA: `0x140D70780`
    pub const FOLLOW_TASK_CONSTRUCT: usize = 0xD7_0780;

    /// SetAggressivity native.
    /// Пишет в *(*(entity+0xA8) + 4) = value.
    /// IDA: `0x140DCE6D0`
    pub const SET_AGGRESSIVITY: usize = 0xDC_E6D0;
}

pub mod sds {
    /// Native ActivateStreamMapLine.
    /// IDA: `0x1403F4F30`
    pub const ACTIVATE_STREAM_MAP_LINE: usize = 0x3F_4F30;

    /// Парсер /sdsconfig.bin.
    /// IDA: `0x1403F0640`
    pub const PARSE_CONFIG: usize = 0x3F_0640;

    /// SDSManager constructor. Module ID = 0x0D.
    /// IDA: `0x1403D1D40`
    pub const MANAGER_CONSTRUCTOR: usize = 0x3D_1D40;

    /// Per-frame SDS tick (priority 2900).
    /// IDA: `0x1403FA210`
    pub const MANAGER_TICK: usize = 0x3F_A210;

    /// Frame resource name resolver.
    /// IDA: `0x1403EF480`
    pub const FRAME_RESOURCE_RESOLVE: usize = 0x3E_F480;

    /// Scene graph frame attach by hash.
    /// IDA: `0x1406E6FB0`
    pub const SCENE_GRAPH_ATTACH_FRAME: usize = 0x6E_6FB0;

    /// SDS file loader.
    /// IDA: `0x1403F0EB0`
    pub const LOAD_SDS_FILE: usize = 0x3F_0EB0;

    /// SDS file loader core (via CarManager).
    /// IDA: `0x1403F0BB0`
    pub const LOAD_SDS_FILE_CORE: usize = 0x3F_0BB0;
}

pub mod type_registry {
    /// Создать native entity по type ID.
    /// Linked list lookup -> call create function.
    /// IDA: `0x1403A4DE0`
    pub const CREATE_BY_TYPE_ID: usize = 0x3A_4DE0;
}

pub mod car_spawn {
    /// Создать C_Car entity из загруженного SDS slot.
    /// IDA: `0x1403EDDB0`
    pub const CREATE_WORLD_ENTITY_TYPE18: usize = 0x3E_DDB0;

    /// Привязать entity к frame node по имени.
    /// IDA: `0x1403B9570`
    pub const ENTITY_ATTACH_TO_FRAME: usize = 0x3B_9570;

    /// Установить entity ID (+ WorldEntityManager registration).
    /// IDA: `0x1403B91C0`
    pub const ENTITY_SET_ID: usize = 0x3B_91C0;
}

// =============================================================================
//  Entity Constructors
// =============================================================================

pub mod entity_constructors {
    /// C_Entity base конструктор. Sets type -> 0x01 (intermediate).
    /// IDA: `0x14039B710`
    pub const BASE_ENTITY: usize = 0x39_B710;

    /// C_Actor constructor. Sets type -> 0x03 -> 0x05 (intermediate).
    /// IDA: `0x14039A7E0`
    pub const ACTOR_ENTITY: usize = 0x39_A7E0;

    /// C_Human base constructor. Allocates 2648B component block.
    /// IDA: `0x140D730B0`
    pub const HUMAN_BASE: usize = 0xD7_30B0;

    /// C_HumanNPC constructor. Sets type -> 0x0E.
    /// IDA: `0x140D712E0`
    pub const HUMAN_NPC: usize = 0xD7_12E0;

    /// C_Player constructor. Sets type -> 0x10.
    /// IDA: `0x1400B9160`
    pub const PLAYER: usize = 0x0B_9160;

    /// C_Car constructor. Sets type -> 0x12. xor r14d,r14d; lea edx,[r14+12h].
    /// IDA: `0x1400EE6C0`
    pub const CAR: usize = 0x0E_E6C0;

    /// C_Car CreateInstance (alloc + construct).
    /// IDA: `0x140109030`
    pub const CAR_CREATE_INSTANCE: usize = 0x10_9030;

    /// C_CutsceneEnt constructor. Sets type -> 0x68. xor ecx,ecx; lea edx,[rcx+68h].
    /// IDA: `0x1400EDF30`
    pub const CUTSCENE_ENT: usize = 0x0E_DF30;

    /// C_CutsceneEnt CreateInstance (alloc 0xC8 + construct).
    /// IDA: `0x140109000`
    pub const CUTSCENE_ENT_CREATE_INSTANCE: usize = 0x10_9000;

    /// C_Door constructor. Sets type -> 0x26.
    /// IDA: `0x1400EF4F0`
    pub const DOOR: usize = 0x0E_F4F0;

    /// C_Lift constructor. Sets type -> 0x28.
    /// IDA: `0x1400F00B0`
    pub const LIFT: usize = 0x0F_00B0;

    /// C_Telephone constructor. Sets type -> 0x5F.
    /// IDA: `0x1400F1750`
    pub const TELEPHONE: usize = 0x0F_1750;

    /// C_TrafficCar constructor. Sets type -> 0x15.
    /// IDA: `0x140C125B0`
    pub const TRAFFIC_CAR: usize = 0xC1_25B0;

    /// StaticEntity constructor. Sets type -> 0x6C.
    /// IDA: `0x140C0E870`
    pub const STATIC_ENTITY: usize = 0xC0_E870;

    /// C_Cutscene constructor. Sets type -> 0x49.
    /// IDA: `0x140C781A0`
    pub const CUTSCENE: usize = 0xC7_81A0;

    /// FrameWrapper constructor. Sets type -> 0x37.
    /// IDA: `0x140C78330`
    pub const FRAME_WRAPPER: usize = 0xC7_8330;

    /// C_CleanEntity constructor. Sets type -> 0x6F.
    /// IDA: `0x140C78590`
    pub const CLEAN_ENTITY: usize = 0xC7_8590;

    /// LightEntity constructor. Sets type -> 0x47.
    /// IDA: `0x140DF2410`
    pub const LIGHT_ENTITY: usize = 0xDF_2410;

    /// C_Pinup constructor. Sets type -> 0x6A.
    /// IDA: `0x140DF2750`
    pub const PINUP: usize = 0xDF_2750;

    /// C_ActionPointCrossing constructor. Sets type -> 0x34.
    /// IDA: `0x140DF2B30`
    pub const ACTION_POINT_CROSSING: usize = 0xDF_2B30;

    /// C_StaticParticle constructor. Sets type -> 0x42.
    /// IDA: `0x140DF2CC0`
    pub const STATIC_PARTICLE: usize = 0xDF_2CC0;

    /// C_ActionPointSearch constructor. Sets type -> 0x3F.
    /// IDA: `0x140DF1490`
    pub const ACTION_POINT_SEARCH: usize = 0xDF_1490;

    /// C_StaticWeapon constructor. Sets type -> 0x30.
    /// IDA: `0x1410186B0`
    pub const STATIC_WEAPON: usize = 0x101_86B0;

    /// C_ActorDetector constructor. Sets type -> 0x65.
    /// IDA: `0x14045E5E0`
    pub const ACTOR_DETECTOR: usize = 0x45_E5E0;

    /// C_FireTarget constructor. Sets type -> 0x46.
    /// IDA: `0x140E455A0`
    pub const FIRE_TARGET: usize = 0xE4_55A0;

    /// C_Wardrobe constructor. Sets type -> 0x25.
    /// IDA: `0x140FF7AF0`
    pub const WARDROBE: usize = 0xFF_7AF0;

    /// C_PhysicsScene constructor. Sets type -> 0x73.
    /// IDA: `0x140FE0030`
    pub const PHYSICS_SCENE: usize = 0xFE_0030;

    /// TranslocatedCar constructor. Sets type -> 0x71.
    /// IDA: `0x14039BCA0`
    pub const TRANSLOCATED_CAR: usize = 0x39_BCA0;

    /// C_ScriptEntity constructor. Sets type -> 0x62.
    /// IDA: `0x14039BDE0`
    pub const SCRIPT_ENTITY: usize = 0x39_BDE0;

    /// C_ActionPointRoadBlock constructor. Sets type -> 0x1A.
    /// IDA: `0x140C270F0`
    pub const ACTION_POINT_ROADBLOCK: usize = 0xC2_70F0;

    /// M2DE_Entity_SetTypeID — clears low byte of +0x24, then ORs new type.
    /// `void(C_Entity* this_rcx, u32 type_edx)`
    /// IDA: `0x1403B99F0`
    pub const SET_TYPE_ID: usize = 0x3B_99F0;
}

// =============================================================================
//  ScriptEntity family
// =============================================================================

pub mod script_entity {
    /// Базовый top-level constructor `C_ScriptEntity`.
    ///
    /// Final type = `0x62`
    /// Base alloc size = `0x90`
    ///
    /// IDA: `0x14039BDE0`
    pub const BASE_CONSTRUCT: usize = 0x39_BDE0;

    /// Инициализация уже выделенного блока как ScriptEntity-like object.
    ///
    /// Используется перед заменой vtable у child/direct derived paths.
    ///
    /// IDA: `0x14039BE40`
    pub const INIT_IN_PLACE: usize = 0x39_BE40;

    /// Direct police-script child create-instance path (Sub5).
    ///
    /// Alloc size = `0x90`
    ///
    /// IDA: `0x140EBFD00`
    pub const POLICE_CHILD_CREATE_INSTANCE: usize = 0xEB_FD00;

    /// Direct police-script child ctor (Sub5).
    ///
    /// IDA: `0x1400B3B50`
    pub const POLICE_CHILD_CONSTRUCT: usize = 0x0B_3B50;

    /// Secondary init path for police child.
    ///
    /// IDA: `0x1400B3B80`
    pub const POLICE_CHILD_CONSTRUCT2: usize = 0x0B_3B80;

    /// Scalar deleting dtor for police child.
    ///
    /// IDA: `0x1400B3B90`
    pub const POLICE_CHILD_SCALAR_DTOR: usize = 0x0B_3B90;

    /// Heavy add/init Lua bridge path.
    ///
    /// Observed Lua-side sequence:
    /// - load `scripts[this+0x78]`
    /// - call `onGameInit`
    /// - call `AddPoliceman(self, guid_a, guid_b, number, vec3)`
    ///
    /// IDA: `0x1400B3DA0`
    pub const POLICE_CHILD_INIT_AND_ADD_POLICEMAN: usize = 0x0B_3DA0;

    /// Remove-path Lua bridge.
    ///
    /// Observed Lua-side sequence:
    /// - load `scripts[this+0x78]`
    /// - call `RemovePoliceman(self, guid)`
    ///
    /// IDA: `0x1400B4300`
    pub const POLICE_CHILD_CALL_REMOVE_POLICEMAN_BY_GUID: usize = 0x0B_4300;

    /// Lua bridge helper:
    /// alloc 4 bytes, copy `u32`, wrap as typed `C_EntityGuid`, push to Lua.
    ///
    /// IDA: `0x1403B1630`
    pub const LUA_PUSH_ENTITY_GUID_COMPONENT: usize = 0x3B_1630;
}

// =============================================================================
//  PoliceScriptOwner singleton path (provisional reverse names)
// =============================================================================

pub mod police_script_owner {
    /// Lazy singleton create/get path.
    ///
    /// If global owner is NULL:
    /// - alloc `0x18`
    /// - init owner storage
    /// - store in global
    /// - register atexit shutdown callback
    ///
    /// IDA: `0x1400B3A50`
    pub const GET_OR_CREATE: usize = 0x0B_3A50;

    /// Initializes owner storage object (`0x18` bytes).
    ///
    /// This is NOT a classic C++ ctor with vtable.
    /// It allocates and initializes a `0x30`-byte root/sentinel node object,
    /// then sets owner fields:
    /// - `owner+0x00 = root`
    /// - `owner+0x08 = 0`
    /// - `owner+0x10 = 0`
    ///
    /// IDA: `0x140EAC480`
    pub const INIT: usize = 0xEA_C480;

    /// Atexit shutdown path for singleton owner.
    ///
    /// IDA: `0x1400B3250`
    pub const ATEXIT_SHUTDOWN: usize = 0x0B_3250;

    /// Recursive free helper for `0x30`-byte owner nodes.
    ///
    /// IDA: `0x1400B3AF0`
    pub const FREE_NODE_SUBTREE: usize = 0x0B_3AF0;

    /// Owner dispatch by code from `[rdx+8]`.
    ///
    /// Observed:
    /// - `code == 2` -> destroy child
    /// - `code == 6` -> init child
    ///
    /// IDA: `0x140EC4A10`
    pub const DISPATCH: usize = 0xEC_4A10;

    /// Child destroy/reset path.
    ///
    /// IDA: `0x140EC4220`
    pub const DESTROY_CHILD: usize = 0xEC_4220;

    /// Child init path.
    ///
    /// Creates child using `/scripts/common/Police/`,
    /// stores it at `owner+0x10`, then calls generic entity activation helper.
    ///
    /// IDA: `0x140EC4330`
    pub const INIT_CHILD: usize = 0xEC_4330;

    /// Trivial dispatch branch: returns success immediately.
    ///
    /// IDA: `0x140EC50A0`
    pub const CODE1_NOOP: usize = 0xEC_50A0;

    /// Tiny getter: `return *(u32*)(rcx + 0x10)`.
    ///
    /// In this owner/police-related cluster this behaves like:
    /// - entity guid
    /// - packed identity
    /// - code/key used for later Lua/entity lookup
    ///
    /// IDA: `0x140EC8350`
    pub const NODE_GET_ENTITY_GUID: usize = 0xEC_8350;

    /// Owner-side forwarder into active child:
    /// - child = owner->active_child (`[rcx+0x10]`)
    /// - tail-jump into child-side remove path
    ///
    /// IDA: `0x140EE14A0`
    pub const REMOVE_POLICEMAN_BY_GUID: usize = 0xEE_14A0;
}
