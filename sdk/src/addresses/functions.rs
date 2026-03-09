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

    // ── Weapon — inventory-level ────────────────────────────────────

    /// `char(Inventory*, u32 weapon_id, i32 ammo)` — добавить оружие.
    ///
    /// Логика:
    /// - Если оружие уже есть → добавляет только патроны
    /// - Если нового типа → создаёт WeaponItem, пробует slot[2] и slot[3]
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
    pub const SET_LOCKED: usize = 0xDB_1B40;

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
    ///   `entity + 0x78` → frame
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

// ═══════════════════════════════════════════════════════════════════════════
//  Raw Lua API (Mafia II: DE Lua 5.1.2, modified)
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// Callbacks
// ═══════════════════════════════════════════════════════════════════════════

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
    /// Специальный случай: `event_id == -1` → удалить из всех событий.
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