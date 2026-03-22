//! Система типов сущностей — два уровня идентификации.
//!
//! # Два мира типов
//!
//! Движок Mafia II: DE использует **две разные** системы типизации сущностей:
//!
//! 1. **Lua `E_EntityType`** — перечисление для Lua скриптов.
//!    Значения из float-констант в getter-функциях. 69 значений (0..117, разреженные).
//!    Используется: `enums.EntityType.CAR`, `enums.EntityType.HUMAN`.
//!
//! 2. **Factory type byte** — внутренний движковый тип, хранится в `entity+0x24` (младший байт).
//!    Используется движком для: создания обёрток (WrapperFactory), диспетчеризации конструкторов,
//!    определения класса сущности в рантайме.
//!
//! # Несовпадения
//!
//! Для большинства типов значения **совпадают** (HUMAN=14, PLAYER=16, DOOR=38).
//! Ключевые **несовпадения**:
//!
//! | Класс         | Lua E_EntityType | Factory byte | Примечание               |
//! |---------------|:----------------:|:------------:|--------------------------|
//! | C_Car (паркинг)| 18 (CAR)        | 0x12 (18)    | Совпадают числом, но...  |
//! | C_CarVehicle  | 18 (CAR)         | 0x70 (112)   | Управляемая машина!      |
//! | C_DamageZone  | —                | 0x1E (30)    | Нет Lua enum             |
//! | C_Blocker     | —                | 0x64 (100)   | Нет Lua enum             |
//! | C_CleanEntity | —                | 0x6F (111)   | Нет Lua enum             |
//!
//! # Источники
//!
//! - IDA Python: float-константы в getter-функциях C_WrapE_EntityType (69 значений)
//! - IDA: SetTypeID xref analysis (30+ конструкторов)
//! - Runtime: EntityDatabase scan (2415 entities, FreeRide mode)
//! - Runtime: WrapperFactory RB-tree dump (36 entries)

use crate::addresses::constants::factory_types;

// =============================================================================
//  Lua E_EntityType — используется скриптами
// =============================================================================

/// Lua-перечисление типов сущностей (`enums.EntityType`).
///
/// Значения подтверждены через IDA Python скрипт — анализ float-констант
/// в getter-функциях `C_WrapE_EntityType`. 69 значений из 117 (разреженный enum).
///
/// Эти значения видны ТОЛЬКО в Lua. Движок внутри использует [`FactoryType`].
///
/// # Использование в Lua
///
/// ```lua
/// local et = enums.EntityType
/// if entity:GetType() == et.HUMAN then ... end
/// ```
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Unknown = 0,
    Entity = 1,
    MultiplayerEntity = 2,
    EntityPos = 3,
    EntityDummy = 4,
    Actor = 5,
    ActorMultiframe = 6,
    TickedModule = 7,
    TickedModuleManager = 8,
    SystemInitDone = 9,
    Game = 10,
    Mission = 11,
    EditorComm = 12,
    BaseHuman = 13,
    Human = 14,
    // 15 — пропущен
    Player = 16,
    // 17 — пропущен
    Car = 18,
    Train = 19,
    CrashObject = 20,
    TrafficCar = 21,
    TrafficHuman = 22,
    TrafficTrain = 23,
    TrafficBase = 24,
    ActionPoint = 25,
    // 26..32 — пропущены
    GlassManager = 33,
    MultiplayerSpawnPoint = 34,
    MultiplayerSpawnNest = 35,
    Item = 36,
    Glassbreaking = 37,
    Door = 38,
    Tree = 39,
    Lift = 40,
    Sound = 41,
    SoundSelector = 42,
    SoundMixer = 43,
    Animator = 44,
    Grenade = 45,
    // 46..54 — пропущены
    FrameWrapper = 55,
    // 56 — пропущен
    MafiaFirst = 57,
    Pathgen = 58,
    Music = 59,
    // 60 — пропущен
    MoscowFirst = 61,
    Trigger = 62,
    // 63 — пропущен
    Explosion = 64,
    StaticWeapon = 65,
    StaticParticle = 66,
    Demolition = 67,
    Fracturing = 68,
    Helicopter = 69,
    Firetarget = 70,
    LightEntity = 71,
    Barrel = 72,
    Cutscene = 73,
    AnimEntity = 74,
    TvCamera = 75,
    Monitor = 76,
    MissionState = 77,
    ClockSimple = 78,
    ClockCrashObj = 79,
    RespawnPoint = 80,
    PressureTank = 81,
    // 82..94 — пропущены
    Telephone = 95,
    HitechFirst = 96,
    ScriptMachine = 97,
    Script = 98,
    // 99..105 — пропущены
    Pinup = 106,
    // 107..108 — пропущены
    DummyDoor = 109,
    FramesController = 110,
    // 111..116 — пропущены
    LastId = 117,
}

impl EntityType {
    /// Lua-имя типа (как в `enums.EntityType.XXX`).
    pub fn lua_name(self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::Entity => "ENTITY",
            Self::MultiplayerEntity => "MULTIPLAYER_ENTITY",
            Self::EntityPos => "ENTITY_POS",
            Self::EntityDummy => "ENTITY_DUMMY",
            Self::Actor => "ACTOR",
            Self::ActorMultiframe => "ACTOR_MULTIFRAME",
            Self::TickedModule => "TICKED_MODULE",
            Self::TickedModuleManager => "TICKED_MODULE_MANAGER",
            Self::SystemInitDone => "SYSTEM_INIT_DONE",
            Self::Game => "GAME",
            Self::Mission => "MISSION",
            Self::EditorComm => "EDITOR_COMM",
            Self::BaseHuman => "BASE_HUMAN",
            Self::Human => "HUMAN",
            Self::Player => "PLAYER",
            Self::Car => "CAR",
            Self::Train => "TRAIN",
            Self::CrashObject => "CRASH_OBJECT",
            Self::TrafficCar => "TRAFFIC_CAR",
            Self::TrafficHuman => "TRAFFIC_HUMAN",
            Self::TrafficTrain => "TRAFFIC_TRAIN",
            Self::TrafficBase => "TRAFFIC_BASE",
            Self::ActionPoint => "ACTION_POINT",
            Self::GlassManager => "GLASS_MANAGER",
            Self::MultiplayerSpawnPoint => "MULTIPLAYER_SPAWN_POINT",
            Self::MultiplayerSpawnNest => "MULTIPLAYER_SPAWN_NEST",
            Self::Item => "ITEM",
            Self::Glassbreaking => "GLASSBREAKING",
            Self::Door => "DOOR",
            Self::Tree => "TREE",
            Self::Lift => "LIFT",
            Self::Sound => "SOUND",
            Self::SoundSelector => "SOUND_SELECTOR",
            Self::SoundMixer => "SOUND_MIXER",
            Self::Animator => "ANIMATOR",
            Self::Grenade => "GRENADE",
            Self::FrameWrapper => "FRAME_WRAPPER",
            Self::MafiaFirst => "MAFIA_FIRST",
            Self::Pathgen => "PATHGEN",
            Self::Music => "MUSIC",
            Self::MoscowFirst => "MOSCOW_FIRST",
            Self::Trigger => "TRIGGER",
            Self::Explosion => "EXPLOSION",
            Self::StaticWeapon => "STATIC_WEAPON",
            Self::StaticParticle => "STATIC_PARTICLE",
            Self::Demolition => "DEMOLITION",
            Self::Fracturing => "FRACTURING",
            Self::Helicopter => "HELICOPTER",
            Self::Firetarget => "FIRETARGET",
            Self::LightEntity => "LIGHTENTITY",
            Self::Barrel => "BARREL",
            Self::Cutscene => "CUTSCENE",
            Self::AnimEntity => "ANIMENTITY",
            Self::TvCamera => "TVCAMERA",
            Self::Monitor => "MONITOR",
            Self::MissionState => "MISSIONSTATE",
            Self::ClockSimple => "CLOCK_SIMPLE",
            Self::ClockCrashObj => "CLOCK_CRASHOBJ",
            Self::RespawnPoint => "RESPAWNPOINT",
            Self::PressureTank => "PRESSURE_TANK",
            Self::Telephone => "TELEPHONE",
            Self::HitechFirst => "HITECH_FIRST",
            Self::ScriptMachine => "SCRIPTMACHINE",
            Self::Script => "SCRIPT",
            Self::Pinup => "PINUP",
            Self::DummyDoor => "DUMMY_DOOR",
            Self::FramesController => "FRAMES_CONTROLLER",
            Self::LastId => "LAST_ID",
        }
    }

    /// Основной factory type byte для данного Lua-типа.
    ///
    /// Возвращает `None` для типов без прямого маппинга на native entity
    /// (управленческие типы: Game, Mission, TickedModule и т.д.).
    ///
    /// # Особый случай: CAR
    ///
    /// `EntityType::Car` (18) маппится на `factory_types::CAR` (0x12) —
    /// это **припаркованные** машины. Для **управляемых** машин движок
    /// использует `factory_types::CAR_VEHICLE` (0x70), но Lua всё равно
    /// видит их как `EntityType::Car`.
    pub fn to_primary_factory_type(self) -> Option<u8> {
        match self {
            Self::Entity => Some(factory_types::ENTITY),
            Self::EntityPos => Some(factory_types::ENTITY_POS),
            Self::EntityDummy => Some(factory_types::ENTITY_DUMMY),
            Self::Actor => Some(factory_types::ACTOR),
            Self::Human => Some(factory_types::HUMAN),
            Self::Player => Some(factory_types::PLAYER),
            Self::Car => Some(factory_types::CAR),
            Self::CrashObject => Some(factory_types::CRASH_OBJECT),
            Self::TrafficCar => Some(factory_types::TRAFFIC_CAR),
            Self::TrafficHuman => Some(factory_types::TRAFFIC_HUMAN),
            Self::TrafficTrain => Some(factory_types::TRAFFIC_TRAIN),
            Self::ActionPoint => Some(factory_types::ACTION_POINT),
            Self::Item => Some(factory_types::ITEM),
            Self::Door => Some(factory_types::DOOR),
            Self::Tree => Some(factory_types::TREE),
            Self::Lift => Some(factory_types::LIFT),
            Self::Sound => Some(factory_types::SOUND),
            Self::SoundMixer => Some(factory_types::SOUND_MIXER),
            Self::Grenade => Some(factory_types::GRENADE),
            Self::FrameWrapper => Some(factory_types::FRAME_WRAPPER),
            Self::StaticWeapon => Some(factory_types::STATIC_WEAPON_WRAPPER),
            Self::StaticParticle => Some(factory_types::STATIC_PARTICLE),
            Self::Firetarget => Some(factory_types::FIRE_TARGET),
            Self::LightEntity => Some(factory_types::LIGHT_ENTITY),
            Self::Cutscene => Some(factory_types::CUTSCENE),
            Self::Telephone => Some(factory_types::TELEPHONE),
            Self::Script => Some(factory_types::SCRIPT_ENTITY),
            Self::Pinup => Some(factory_types::PINUP),
            Self::DummyDoor => Some(factory_types::DUMMY_DOOR),
            Self::FramesController => Some(factory_types::FRAMES_CONTROLLER),
            _ => None,
        }
    }

    /// Создаёт `EntityType` из factory type byte.
    ///
    /// Обратный маппинг — может быть **неоднозначным**:
    /// - `0x12` (C_Car) и `0x70` (C_CarVehicle) оба дают `Car`
    /// - `0x30` (StaticWeapon entity) и `0x41` (StaticWeapon wrapper) оба дают `StaticWeapon`
    /// - `0x49` (Cutscene) и `0x68` (CutsceneEnt) оба дают `Cutscene`
    pub fn from_factory_type(ft: u8) -> Option<Self> {
        match ft {
            factory_types::ENTITY => Some(Self::Entity),
            factory_types::ENTITY_POS => Some(Self::EntityPos),
            factory_types::ENTITY_DUMMY => Some(Self::EntityDummy),
            factory_types::ACTOR => Some(Self::Actor),
            factory_types::HUMAN => Some(Self::Human),
            factory_types::PLAYER => Some(Self::Player),
            factory_types::CAR | factory_types::CAR_VEHICLE => Some(Self::Car),
            factory_types::CRASH_OBJECT => Some(Self::CrashObject),
            factory_types::TRAFFIC_CAR => Some(Self::TrafficCar),
            factory_types::TRAFFIC_HUMAN => Some(Self::TrafficHuman),
            factory_types::TRAFFIC_TRAIN => Some(Self::TrafficTrain),
            factory_types::ACTION_POINT => Some(Self::ActionPoint),
            factory_types::ITEM => Some(Self::Item),
            factory_types::DOOR => Some(Self::Door),
            factory_types::TREE => Some(Self::Tree),
            factory_types::LIFT => Some(Self::Lift),
            factory_types::SOUND => Some(Self::Sound),
            factory_types::SOUND_MIXER => Some(Self::SoundMixer),
            factory_types::GRENADE => Some(Self::Grenade),
            factory_types::FRAME_WRAPPER => Some(Self::FrameWrapper),
            factory_types::STATIC_WEAPON | factory_types::STATIC_WEAPON_WRAPPER => {
                Some(Self::StaticWeapon)
            }
            factory_types::STATIC_PARTICLE => Some(Self::StaticParticle),
            factory_types::FIRE_TARGET => Some(Self::Firetarget),
            factory_types::LIGHT_ENTITY => Some(Self::LightEntity),
            factory_types::CUTSCENE | factory_types::CUTSCENE_ENT => Some(Self::Cutscene),
            factory_types::TELEPHONE => Some(Self::Telephone),
            factory_types::SCRIPT_ENTITY => Some(Self::Script),
            factory_types::PINUP => Some(Self::Pinup),
            factory_types::DUMMY_DOOR => Some(Self::DummyDoor),
            factory_types::FRAMES_CONTROLLER => Some(Self::FramesController),
            _ => None,
        }
    }

    /// Проверяет, является ли тип «человекоподобным» (Human или Player).
    pub fn is_humanoid(self) -> bool {
        matches!(self, Self::Human | Self::Player | Self::BaseHuman)
    }

    /// Проверяет, является ли тип транспортом.
    pub fn is_vehicle(self) -> bool {
        matches!(
            self,
            Self::Car | Self::TrafficCar | Self::Train | Self::TrafficTrain
        )
    }

    /// Проверяет, является ли тип «управленческим» (нет native entity).
    pub fn is_system_type(self) -> bool {
        matches!(
            self,
            Self::TickedModule
                | Self::TickedModuleManager
                | Self::SystemInitDone
                | Self::Game
                | Self::Mission
                | Self::EditorComm
                | Self::MafiaFirst
                | Self::Pathgen
                | Self::Music
                | Self::MoscowFirst
                | Self::HitechFirst
                | Self::ScriptMachine
        )
    }
}

// =============================================================================
//  Lua E_EntityMessageType — типы сообщений для скриптов
// =============================================================================

/// Lua-перечисление типов сообщений (`enums.EntityMessageType`).
///
/// Используется для подписки на сообщения через
/// `RegisterToMessages(guid, event_type, message_id)`.
///
/// 14 значений (0..14, пропуск на 6). Подтверждено через IDA float-константы.
///
/// # Формат message_id
///
/// Полный ID сообщения: `(event_type << 16) | sub_id`.
///
/// Примеры:
/// - `Human` (3): `0xD0014` = death, `0xD0057` = shot
/// - `Actor` (2): `0x50001` = damage, `0x50004` = activate
/// - `Car`   (4): `0x40000+` = car-specific messages
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityMessageType {
    /// Базовое сообщение (event_type = 0).
    BaseMessage = 0,
    /// Пробуждение скрипта (event_type = 1).
    ScriptWake = 1,
    /// Сообщения актёра: урон, активация, скрипт (event_type = 2, base = 0x50000).
    Actor = 2,
    /// Сообщения человека: смерть, выстрел, вход в машину (event_type = 3, base = 0xD0000).
    Human = 3,
    /// Сообщения машины: столкновение, под водой (event_type = 4).
    Car = 4,
    /// Сообщения двери: открытие, закрытие (event_type = 5).
    Door = 5,
    // 6 — пропущен
    /// Квестовые сообщения (event_type = 7).
    Quest = 7,
    /// Состояния AI (event_type = 8).
    AiState = 8,
    /// Огневая цель (event_type = 9).
    FireTarget = 9,
    /// Синхронизация объектов (event_type = 10).
    SyncObjectEvent = 10,
    /// Телефон (event_type = 11).
    Telephone = 11,
    /// Детектор актёров (event_type = 12).
    ActorDetector = 12,
    /// Постер/пинап (event_type = 13).
    Pinup = 13,
    /// Пропуск скриптовой сцены (event_type = 14).
    SkipScriptScene = 14,
}

impl EntityMessageType {
    /// Базовый ID для сообщений этого типа: `(event_type << 16)`.
    pub fn message_base(self) -> u32 {
        (self as u32) << 16
    }

    /// Lua-имя типа сообщения.
    pub fn lua_name(self) -> &'static str {
        match self {
            Self::BaseMessage => "BASEMESSAGE",
            Self::ScriptWake => "SCRIPTWAKE",
            Self::Actor => "ACTOR",
            Self::Human => "HUMAN",
            Self::Car => "CAR",
            Self::Door => "DOOR",
            Self::Quest => "QUEST",
            Self::AiState => "AISTATE",
            Self::FireTarget => "FIRETARGET",
            Self::SyncObjectEvent => "SYNCOBJECTEVENT",
            Self::Telephone => "TELEPHONE",
            Self::ActorDetector => "ACTORDETECTOR",
            Self::Pinup => "PINUP",
            Self::SkipScriptScene => "SKIPSCRIPTSCENE",
        }
    }
}

// =============================================================================
//  Factory Type — перечисление движковых типов
// =============================================================================

/// Движковый factory type byte — хранится в `entity+0x24` (младший байт packed table_id).
///
/// Это **рантайм-идентификатор** класса сущности. Устанавливается конструктором
/// через `M2DE_Entity_SetTypeID` (0x1403B99F0). Используется:
/// - WrapperFactory для создания Lua-обёрток
/// - Рантайм type-check через `(entity+0x24) & 0xFF`
/// - TypeRegistry для десериализации из SDS
///
/// Подтверждено: runtime DB scan (2415 entities), IDA SetTypeID xrefs (30+ конструкторов).
///
/// # Отличия от Lua EntityType
///
/// Для большинства типов factory byte == Lua enum значение.
/// Исключения задокументированы в [`EntityType::to_primary_factory_type`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactoryType {
    // === Базовые (промежуточные, перезаписываются дочерним конструктором) ===
    Entity = 0x01,
    EntityPos = 0x03,
    EntityDummy = 0x04,
    Actor = 0x05,

    // === Люди ===
    /// C_HumanNPC — NPC (86 в FreeRide). Vtable: 0x1418E5188.
    HumanNpc = 0x0E,
    /// C_Player — игрок (1). Vtable: 0x14184C060.
    Player = 0x10,

    // === Транспорт ===
    /// C_Car — припаркованная/статичная машина (41). Vtable: 0x141850030.
    /// Размер: ~0x1258+ байт. Конструктор: 0x1400EE6C0.
    Car = 0x12,
    /// C_CrashObject — разрушаемые объекты (528). Vtable: 0x1418E8D00.
    CrashObject = 0x14,
    /// C_TrafficCar — AI трафик машины (13). Vtable: 0x1418D1EC8.
    TrafficCar = 0x15,
    /// C_TrafficHuman — AI трафик пешеходы (13). Vtable: 0x1418D2320.
    TrafficHuman = 0x16,
    /// C_TrafficTrain — AI поезд (1). Vtable: 0x1418D2090.
    TrafficTrain = 0x17,

    // === ActionPoints ===
    ActionPoint = 0x19,
    ActionPointRoadBlock = 0x1A,
    ActionPointBase = 0x33,
    ActionPointCrossing = 0x34,
    ActionPointSideWalk = 0x35,
    ActionPointScript = 0x36,
    ActionPointSearch = 0x3F,

    // === Мир ===
    DamageZone = 0x1E,
    TelephoneReg = 0x20,
    Item = 0x24,
    Wardrobe = 0x25,
    Door = 0x26,
    Tree = 0x27,
    Lift = 0x28,
    Grenade = 0x2F,

    // === Звук ===
    Sound = 0x29,
    SoundMixer = 0x2B,

    // === Оружие / боевые ===
    /// C_StaticWeapon entity (тип 0x30). НЕ путать с wrapper 0x41.
    StaticWeapon = 0x30,
    /// C_StaticWeapon wrapper factory key. Lua StaticWeapon = 65.
    StaticWeaponWrap = 0x41,
    FireTarget = 0x46,

    // === Визуальные ===
    FrameWrapper = 0x37,
    ShopMenuScript = 0x38,
    StaticParticle = 0x42,
    /// LightEntity — источники света (437!). Vtable: 0x1418E84F0.
    LightEntity = 0x47,
    Pinup = 0x6A,

    // === Специальные ===
    Cutscene = 0x49,
    Jukebox = 0x5C,
    Telephone = 0x5F,
    /// C_ScriptEntity — 5 подклассов! (200 entities, 5 vtable).
    ScriptEntity = 0x62,
    Blocker = 0x64,
    ActorDetector = 0x65,
    Boat = 0x67,
    /// C_Cutscene entity в мире (2). Vtable: 0x1418519C8.
    CutsceneEnt = 0x68,
    DummyDoor = 0x6B,
    StaticEntity = 0x6C,
    FramesController = 0x6E,
    CleanEntity = 0x6F,
    /// C_CarVehicle — управляемая машина (1). Vtable: 0x1418EAAC8.
    CarVehicle = 0x70,
    TranslocatedCar = 0x71,
    CleanEntityAlt = 0x72,
    PhysicsScene = 0x73,
    Train = 0x76,
    Radio = 0x77,
    Garage = 0x78,
    SpikeStrip = 0x79,
}

impl FactoryType {
    /// Создаёт из сырого байта. Возвращает None для неизвестных значений.
    pub fn from_byte(b: u8) -> Option<Self> {
        // Проверяем все известные значения
        match b {
            0x01 => Some(Self::Entity),
            0x03 => Some(Self::EntityPos),
            0x04 => Some(Self::EntityDummy),
            0x05 => Some(Self::Actor),
            0x0E => Some(Self::HumanNpc),
            0x10 => Some(Self::Player),
            0x12 => Some(Self::Car),
            0x14 => Some(Self::CrashObject),
            0x15 => Some(Self::TrafficCar),
            0x16 => Some(Self::TrafficHuman),
            0x17 => Some(Self::TrafficTrain),
            0x19 => Some(Self::ActionPoint),
            0x1A => Some(Self::ActionPointRoadBlock),
            0x1E => Some(Self::DamageZone),
            0x20 => Some(Self::TelephoneReg),
            0x24 => Some(Self::Item),
            0x25 => Some(Self::Wardrobe),
            0x26 => Some(Self::Door),
            0x27 => Some(Self::Tree),
            0x28 => Some(Self::Lift),
            0x29 => Some(Self::Sound),
            0x2B => Some(Self::SoundMixer),
            0x2F => Some(Self::Grenade),
            0x30 => Some(Self::StaticWeapon),
            0x33 => Some(Self::ActionPointBase),
            0x34 => Some(Self::ActionPointCrossing),
            0x35 => Some(Self::ActionPointSideWalk),
            0x36 => Some(Self::ActionPointScript),
            0x37 => Some(Self::FrameWrapper),
            0x38 => Some(Self::ShopMenuScript),
            0x3F => Some(Self::ActionPointSearch),
            0x41 => Some(Self::StaticWeaponWrap),
            0x42 => Some(Self::StaticParticle),
            0x46 => Some(Self::FireTarget),
            0x47 => Some(Self::LightEntity),
            0x49 => Some(Self::Cutscene),
            0x5C => Some(Self::Jukebox),
            0x5F => Some(Self::Telephone),
            0x62 => Some(Self::ScriptEntity),
            0x64 => Some(Self::Blocker),
            0x65 => Some(Self::ActorDetector),
            0x67 => Some(Self::Boat),
            0x68 => Some(Self::CutsceneEnt),
            0x6A => Some(Self::Pinup),
            0x6B => Some(Self::DummyDoor),
            0x6C => Some(Self::StaticEntity),
            0x6E => Some(Self::FramesController),
            0x6F => Some(Self::CleanEntity),
            0x70 => Some(Self::CarVehicle),
            0x71 => Some(Self::TranslocatedCar),
            0x72 => Some(Self::CleanEntityAlt),
            0x73 => Some(Self::PhysicsScene),
            0x76 => Some(Self::Train),
            0x77 => Some(Self::Radio),
            0x78 => Some(Self::Garage),
            0x79 => Some(Self::SpikeStrip),
            _ => None,
        }
    }

    /// Имя типа для отладки.
    pub fn display_name(self) -> &'static str {
        crate::game::entity::factory_type_name(self as u8)
    }

    /// Конвертирует в Lua EntityType (если есть маппинг).
    pub fn to_lua_type(self) -> Option<EntityType> {
        EntityType::from_factory_type(self as u8)
    }

    /// Является ли тип «человекоподобным».
    pub fn is_humanoid(self) -> bool {
        matches!(self, Self::HumanNpc | Self::Player)
    }

    /// Является ли тип транспортом.
    pub fn is_vehicle(self) -> bool {
        matches!(
            self,
            Self::Car
                | Self::CarVehicle
                | Self::TrafficCar
                | Self::Train
                | Self::TrafficTrain
                | Self::Boat
        )
    }

    /// Является ли тип ActionPoint (любого вида).
    pub fn is_action_point(self) -> bool {
        matches!(
            self,
            Self::ActionPoint
                | Self::ActionPointRoadBlock
                | Self::ActionPointBase
                | Self::ActionPointCrossing
                | Self::ActionPointSideWalk
                | Self::ActionPointScript
                | Self::ActionPointSearch
        )
    }
}

// =============================================================================
//  Packed table_id helpers
// =============================================================================

/// Извлечь factory type byte из packed table_id.
#[inline]
pub const fn table_id_factory_type(table_id: u32) -> u8 {
    (table_id & 0xFF) as u8
}

/// Извлечь instance id из packed table_id.
#[inline]
pub const fn table_id_instance_id(table_id: u32) -> u32 {
    table_id >> 8
}
