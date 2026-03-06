//! Адреса функций (RVA).
//!
//! Calling convention: Microsoft x64 (`extern "C"` в Rust).
//! - RCX = 1-й аргумент (ptr/int) или XMM0 (float)
//! - RDX = 2-й аргумент или XMM1
//! - R8  = 3-й аргумент или XMM2
//! - R9  = 4-й аргумент или XMM3

// ═══════════════════════════════════════════════════════════════════════════
//  Core System
// ═══════════════════════════════════════════════════════════════════════════

pub mod core {
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

    /// Обработка ввода игрока.
    /// IDA: `0x1400FE640`
    pub const PLAYER_INPUT_HANDLER: usize = 0xFE640;

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

// ═══════════════════════════════════════════════════════════════════════════
//  Player
// ═══════════════════════════════════════════════════════════════════════════

pub mod player {
    // ── WrapperPlayer-level (НЕ вызывать с Player.ptr!) ─────────────

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

    // ── Inventory-level (принимает Inventory*) ──────────────────────
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

    // ── Slot/Core level ─────────────────────────────────────────────

    /// `void(MoneySlot*, i64 cents)` — добавить в слот
    /// IDA: `0x140D7E800`
    pub const MONEY_SLOT_ADD: usize = 0xD7_E800;

    /// `void(MoneyCore*, i64 new_cents)` — установить + notify
    /// IDA: `0x140DCE920`
    pub const MONEY_CORE_SET: usize = 0xDC_E920;

    // ── Deprecated ──────────────────────────────────────────────────

    #[deprecated = "Use WRAPPER_ADD_WEAPON — needs WrapperPlayer*"]
    pub const ADD_WEAPON: usize = 0x10C_9560;
    #[deprecated = "Use get_money_cents() with direct memory read"]
    pub const GET_MONEY: usize = 0x10C_96B0;
    #[deprecated = "Use INVENTORY_ADD_MONEY_NOTIFY from game thread"]
    pub const ADD_MONEY: usize = 0x10C_9520;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Garage
// ═══════════════════════════════════════════════════════════════════════════

pub mod garage {
    /// Получить ID машины по имени (0–33).
    /// Регистрирует все 34 машины.
    ///
    /// `i32(const char* name)` → vehicle ID или -1
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

// ═══════════════════════════════════════════════════════════════════════════
//  Managers
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
//  HUD
// ═══════════════════════════════════════════════════════════════════════════

pub mod hud {
    /// Показать уведомление о деньгах (± $).
    ///
    /// IDA: `0x140D_45B50` (`M2DE_HUD_ShowMoneyNotification`)
    pub const SHOW_MONEY_NOTIFICATION: usize = 0xD4_5B50;

    /// Загрузить иконку HUD.
    ///
    /// IDA: `0x140A_76940` (`M2DE_HUD_LoadIcon`)
    pub const LOAD_ICON: usize = 0xA7_6940;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tables
// ═══════════════════════════════════════════════════════════════════════════

pub mod tables {
    /// Конструктор TableManager — загружает все .tbl файлы.
    ///
    /// IDA: `0x1400_F1480` (`M2DE_TableManager_Constructor`)
    pub const CONSTRUCTOR: usize = 0xF_1480;

    /// Загрузка /tables/vehicles.tbl → TableManager+0x60.
    ///
    /// IDA: `0x1400_DEAD0`
    pub const LOAD_VEHICLES: usize = 0xD_EAD0;

    /// Загрузка /tables/weapons.tbl → TableManager+0x40.
    ///
    /// IDA: `0x1400_DEBE0`
    pub const LOAD_WEAPONS: usize = 0xD_EBE0;

    /// Загрузка /tables/police_offences.tbl → TableManager+0x38.
    ///
    /// IDA: `0x1400_DE140`
    pub const LOAD_POLICE_OFFENCES: usize = 0xD_E140;

    /// Загрузка /tables/attack_params.tbl → TableManager+0x50.
    ///
    /// IDA: `0x1400_DC490`
    pub const LOAD_ATTACK_PARAMS: usize = 0xD_C490;

    /// Парсинг name_or_id (число или FNV-1a хеш).
    ///
    /// IDA: `0x140A_76940`
    pub const PARSE_NAME_OR_ID: usize = 0xA7_6940;

    /// Получить загрузчик ресурсов.
    ///
    /// IDA: `0x1401_85480`
    pub const GET_RESOURCE_LOADER: usize = 0x18_5480;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Profiling
// ═══════════════════════════════════════════════════════════════════════════

pub mod profiling {
    /// IDA: `0x1404_0E7F0`
    pub const BEGIN_PROFILE: usize = 0x40_E7F0;
    /// IDA: `0x1404_130D0`
    pub const END_PROFILE: usize = 0x41_30D0;
    /// IDA: `0x1404_11BE0`
    pub const CREATE_PROFILE: usize = 0x41_1BE0;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Lua Bindings (НЕ вызывать напрямую — принимают lua_State*)
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
//  GameCallbackManager VTable Methods
// ═══════════════════════════════════════════════════════════════════════════

pub mod callback_manager {
    /// Деструктор.
    /// IDA: `0x1403_9FC70`
    pub const DESTRUCTOR: usize = 0x39_FC70;
    /// GetSize() → 8.
    /// IDA: `0x1403_AC3D0`
    pub const GET_SIZE: usize = 0x3A_C3D0;
    /// RegisterCallback.
    /// IDA: `0x1403_A08F0`
    pub const REGISTER_CALLBACK: usize = 0x3A_08F0;
    /// RegisterFunction.
    /// IDA: `0x1403_A06D0`
    pub const REGISTER_FUNCTION: usize = 0x3A_06D0;
}

// ═══════════════════════════════════════════════════════════════════════════
//  GarageManager VTable Methods
// ═══════════════════════════════════════════════════════════════════════════

pub mod garage_manager_methods {
    /// GetSize() → 7.
    /// IDA: `0x1400_A78D0`
    pub const GET_SIZE: usize = 0xA_78D0;
    /// GetClassName() → "C_GarageManager".
    /// IDA: `0x1410_22F70`
    pub const GET_CLASS_NAME: usize = 0x102_2F70;
    /// GetSomeFloat() → 0.005f.
    /// IDA: `0x1400_A7D80`
    pub const GET_FLOAT: usize = 0xA_7D80;
    /// Unknown method.
    /// IDA: `0x1410_272D0`
    pub const METHOD1: usize = 0x102_72D0;
}