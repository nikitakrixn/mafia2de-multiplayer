//! Игровые константы: ID оружия, машин, слотов, и т.д.

/// ID оружия для `Player::add_weapon`.
pub mod weapons {
    // ПИСТОЛЕТЫ / РЕВОЛЬВЕРЫ
    pub const EMPTY_HANDS: u32 = 1;
    pub const MODEL_12_REVOLVER: u32 = 2; // ".38 MP2"
    pub const MAUSER_C96: u32 = 3; // "Mauser C-96"
    pub const COLT_M1911A1: u32 = 4; // "Colt 1911"
    pub const COLT_M1911_SPECIAL: u32 = 5; // "Colt 1911 Ext"
    pub const MODEL_19_REVOLVER: u32 = 6; // ".357 Magnum"

    // ДРОБОВИКИ / ПП
    pub const REMINGTON_870: u32 = 8; // "Remington 870"
    pub const M3_GREASE_GUN: u32 = 9; // "M3 Grease gun"
    pub const MP40: u32 = 10; // "MP 40"
    pub const THOMPSON_1928: u32 = 11; // "Thompson 1928"
    pub const M1A1_THOMPSON: u32 = 12; // "Thomson M1A1"
    pub const BERETTA_38A: u32 = 13; // "Beretta 38A"

    // ВИНТОВКИ / ТЯЖЁЛЫЕ
    pub const M1_GARAND: u32 = 15; // "M1 Garand"
    pub const KAR98K: u32 = 17; // "Mauser 98k"
    pub const MG42: u32 = 14; // "MG42"
    pub const KAR98K_SNIPER: u32 = 18; // "Mauser 98k sniper"
    pub const BAZOOKA: u32 = 19; // "Bazooka"

    // ГРАНАТЫ
    pub const GRENADE_SICILY: u32 = 7; // "Grenade Sicily"
    pub const MK2_FRAG_GRENADE: u32 = 20; // "Grenade MkII"
    pub const MOLOTOV_COCKTAIL: u32 = 21; // "Molotov"
    pub const STIELHANDGRANATE: u32 = 16; // "Stielhandgranate 24"
    pub const TANK_EXP: u32 = 125; // "TankExp"
}

pub mod items {
    // ОСНОВНЫЕ ПРЕДМЕТЫ
    pub const SUITCASE_VITO: u32 = 46; // "kufrik" — чемодан миссии 2!
    pub const MONEY: u32 = 39; // "money"
    pub const KEYS: u32 = 45; // "POklice"
    pub const NEWSPAPER: u32 = 47; // "Newspaper"
    pub const LETTER: u32 = 123; // "Letter_City"

    // ВЗРЫВЧАТКА / ГАЗ
    pub const GAS_CANISTER: u32 = 38; // "Gasoline container"
    pub const GAS_STATION: u32 = 119; // "benzinka"
    pub const C4_CHARGE: u32 = 83; // "Charge"
    pub const C4_DETONATOR: u32 = 88; // "Roznetka"

    // БЛИЖНИЙ БОЙ
    pub const KNIFE: u32 = 22; // "Knife"
    pub const KNUCKLES: u32 = 23; // "Knuckleduster"
    pub const WRENCH: u32 = 24; // "Wrench"
    pub const PIPE: u32 = 25; // "Pipe"
    pub const CROWBAR: u32 = 27; // "Crowbar"
    pub const BASEBALL_BAT: u32 = 32; // "Baseball bat"
    pub const HAMMER: u32 = 116; // "kladivo"

    // ЕДА / ПИТЬЁ / СИГАРЕТЫ
    pub const HOTDOG: u32 = 48; // "HotDog"
    pub const COFFEE: u32 = 49; // "Coffee"
    pub const BEER_FULL: u32 = 50; // "Beer"
    pub const CIGARETTES: u32 = 61; // "Cigaret"
    pub const LIGHTER: u32 = 62; // "lighter"

    // МАГАЗИННЫЕ / МИССИОННЫЕ
    pub const POLICE_TICKET: u32 = 55; // "LTC"
    pub const SEAGIFT_BOX: u32 = 51; // "Seagift Box"

    // ИНСТРУМЕНТЫ / БЫТОВУХА
    pub const GLASS_CUTTER: u32 = 96; // "glass-cutter"
    pub const SHOVEL: u32 = 34; // "Shovel"
}

/// Индексы слотов инвентаря.
pub mod slots {
    /// Слоты оружия (weapon item попадает в один из них).
    pub const WEAPON_1: usize = 2;
    pub const WEAPON_2: usize = 3;
    /// Слот боеприпасов (отдельно от оружия).
    pub const AMMO: usize = 4;
    /// Слот с деньгами.
    pub const MONEY: usize = 5;
}

/// Цвета HUD уведомлений о деньгах (как signed i32).
pub mod hud_colors {
    /// Зелёный `#A0FF00` (прибыль).
    pub const MONEY_GAIN: i32 = -6_226_016;
    /// Красный `#FFA000` (расход).
    pub const MONEY_LOSS: i32 = -24_416;
}

/// Состояния Vehicle.
pub mod vehicle_state {
    /// Полностью инициализирован и заспавнен.
    pub const ACTIVE: i32 = 1;
    /// Создан, но не заспавнен.
    pub const DEFERRED: i32 = 5;
    /// Только что сконструирован.
    pub const INITIAL: i32 = 6;
}

/// ID машин для гаража (0–33).
///
/// Только эти 34 машины могут храниться в гараже.
/// Регистрируются через `GetVehicleIDByName` (`0x14101D0A0`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageVehicleId {
    AscotBaileys200Pha = 0,
    BerkleyKingfisherPha = 1,
    DeliziaGrandeamerica = 2,
    HotRod1 = 3,
    HotRod2 = 4,
    HotRod3 = 5,
    HoustonWaspPha = 6,
    Isw508 = 7,
    Jeep = 8,
    JeepCivil = 9,
    JeffersonFuturaPha = 10,
    JeffersonProvincial = 11,
    Lassiter69 = 12,
    Lassiter75Pha = 13,
    PotomacIndian = 14,
    QuicksilverWindsorPha = 15,
    QuicksilverWindsorTaxi = 16,
    Shubert38 = 17,
    ShubertBeverly = 18,
    ShubertFrigatePha = 19,
    ShubertHearse = 20,
    ShubertPanel = 21,
    ShubertPickup = 22,
    ShubertTaxi = 23,
    Smith200PPha = 24,
    Smith200Pha = 25,
    SmithMainlinePha = 26,
    SmithStingrayPha = 27,
    SmithV8 = 28,
    SmithWagonPha = 29,
    UlverNewyorker = 30,
    UlverNewyorkerP = 31,
    WalkerRocket = 32,
    WalterCoupe = 33,
}

impl GarageVehicleId {
    /// Имя машины для передачи в игровые функции.
    pub fn name(self) -> &'static str {
        match self {
            Self::AscotBaileys200Pha => "Ascot_Baileys_200_pha",
            Self::BerkleyKingfisherPha => "Berkley_Kingfisher_pha",
            Self::DeliziaGrandeamerica => "Delizia_Grandeamerica",
            Self::HotRod1 => "Hot_Rod_1",
            Self::HotRod2 => "Hot_Rod_2",
            Self::HotRod3 => "Hot_Rod_3",
            Self::HoustonWaspPha => "Houston_Wasp_pha",
            Self::Isw508 => "ISW_508",
            Self::Jeep => "Jeep",
            Self::JeepCivil => "Jeep_civil",
            Self::JeffersonFuturaPha => "Jefferson_Futura_pha",
            Self::JeffersonProvincial => "Jefferson_Provincial",
            Self::Lassiter69 => "Lassiter_69",
            Self::Lassiter75Pha => "Lassiter_75_pha",
            Self::PotomacIndian => "Potomac_Indian",
            Self::QuicksilverWindsorPha => "Quicksilver_Windsor_pha",
            Self::QuicksilverWindsorTaxi => "Quicksilver_Windsor_taxi_pha",
            Self::Shubert38 => "Shubert_38",
            Self::ShubertBeverly => "Shubert_Beverly",
            Self::ShubertFrigatePha => "Shubert_Frigate_pha",
            Self::ShubertHearse => "Shubert_Hearse",
            Self::ShubertPanel => "Shubert_Panel",
            Self::ShubertPickup => "Shubert_Pickup",
            Self::ShubertTaxi => "Shubert_Taxi",
            Self::Smith200PPha => "Smith_200_p_pha",
            Self::Smith200Pha => "Smith_200_pha",
            Self::SmithMainlinePha => "Smith_Mainline_pha",
            Self::SmithStingrayPha => "Smith_Stingray_pha",
            Self::SmithV8 => "Smith_V8",
            Self::SmithWagonPha => "Smith_Wagon_pha",
            Self::UlverNewyorker => "Ulver_Newyorker",
            Self::UlverNewyorkerP => "Ulver_Newyorker_p",
            Self::WalkerRocket => "Walker_Rocket",
            Self::WalterCoupe => "Walter_Coupe",
        }
    }
}

/// Подтверждённые ID событий GameCallbackManager.
/// Получены из runtime-дампа (39 событий).
pub mod game_events {
    pub const SYSTEM_INIT: i32 = 1;
    pub const SYSTEM_DONE: i32 = 2;
    pub const GAME_TICK: i32 = 3;
    pub const GAME_TICK_PAUSED: i32 = 4;
    pub const GAME_TICK_ALWAYS: i32 = 5;
    pub const GAME_RENDER: i32 = 7;
    pub const MISSION_QUIT: i32 = 8;
    pub const MISSION_BEFORE_OPEN: i32 = 9;
    pub const MISSION_AFTER_OPEN: i32 = 10;
    pub const MISSION_BEFORE_CLOSE: i32 = 11;
    pub const MISSION_AFTER_CLOSE: i32 = 12;
    pub const GAME_INIT: i32 = 13;
    pub const GAME_DONE: i32 = 14;
    pub const INVALIDATE_ENTITY: i32 = 15;
    pub const INVALIDATE_FRAME: i32 = 16;
    pub const WRITE_GAME_INFO: i32 = 17;
    pub const READ_GAME_INFO: i32 = 18;
    pub const GAME_RESTORE: i32 = 19;
    pub const NO_GAME_START: i32 = 20;
    pub const NO_GAME_END: i32 = 21;
    pub const NO_GAME_TICK: i32 = 22;
    pub const NO_GAME_RENDER: i32 = 23;
    pub const NO_GAME_AFTER_GAME_LOOP: i32 = 24;
    pub const COLLISIONS_LOADED: i32 = 25;
    pub const APACK_FROM_SDS_LOADED: i32 = 26;
    pub const REGISTER_GAME_SAVE_CB: i32 = 27;
    pub const GAMEPARAMS_CHANGED: i32 = 28;
    pub const GAMEPARAMS_PRESAVE: i32 = 29;
    pub const APP_DEACTIVATE: i32 = 30;
    pub const APP_ACTIVATE: i32 = 31;
    pub const LOADING_PROCESS_STARTED: i32 = 32;
    pub const LOADING_PROCESS_FINISHED: i32 = 33;
    pub const GAME_PAUSED: i32 = 34;
    pub const GAME_UNPAUSED: i32 = 35;
    pub const LOADING_FADE_FINISHED: i32 = 36;
    pub const SLOT_WAITING_TICK: i32 = 37;
    pub const SLOT_WAITING_RENDER: i32 = 38;
    pub const SHUTDOWN: i32 = 40;
    pub const WEATHER_MANAGER_CREATED: i32 = 4097;
}

pub mod event_types {
    /// enums.EventType.HUMAN
    pub const HUMAN: i32 = 3;
}

pub mod human_messages {
    pub const DAMAGE: u32 = 851_984; // 0xD0010
    pub const DEATH: u32 = 851_988; // 0xD0014
    pub const ANIM_NOTIFY: u32 = 851_989; // 0xD0015
    pub const ENTER_VEHICLE: u32 = 851_995; // 0xD001B
    pub const LEAVE_VEHICLE: u32 = 851_996; // 0xD001C
    pub const ENTER_VEHICLE_DONE: u32 = 851_997; // 0xD001D
    pub const LEAVE_VEHICLE_DONE: u32 = 851_998; // 0xD001E
    pub const PLAYER_WEAPON_SELECT: u32 = 852_004; // 0xD0024
    pub const PLAYER_WEAPON_HIDE: u32 = 852_005; // 0xD0025
    pub const SHOT: u32 = 852_071; // 0xD0057

    pub const WEAPON_HOLSTER: u32 = 851_999; // 0xD001F
    pub const WEAPON_DRAW: u32 = 852_000; // 0xD0020
    pub const STANCE_CHANGE: u32 = 852_001; // 0xD0021
    pub const STANCE_CLEANUP: u32 = 852_002; // 0xD0022
    pub const STANCE_SECONDARY: u32 = 852_003; // 0xD0023
    pub const HEAD_DAMAGE: u32 = 852_033; // 0xD0041
    pub const BODY_DAMAGE: u32 = 852_034; // 0xD0042
    pub const KILL_DAMAGE: u32 = 852_035; // 0xD0043

    pub const HUMAN_MODE_CHANGE: u32 = 851_972; // 0xD0004
    pub const HUMAN_TICK: u32 = 851_985; // 0xD0011
    pub const HUMAN_SETTLED: u32 = 851_994; // 0xD001A

    /// Убрать оружие (holster). Для player — отправляется как entity message.
    /// Для NPC — пишется напрямую в behavior+0x248.
    pub const HOLSTER_WEAPON: u32 = 852_053; // 0xD0055
}

/// Диапазоны event_type для быстрой классификации message_id.
pub mod message_ranges {
    /// Event type 5: actor-level messages (DAMAGE, ACTIVATE, SCRIPT, etc.)
    pub const ACTOR_BASE: u32 = 0x50000;

    /// Event type 3: HUMAN messages.
    pub const HUMAN_BASE: u32 = 0xD0000;

    /// Event type 18: traffic/AI messages.
    pub const TRAFFIC_BASE: u32 = 0x120000;

    pub fn event_type_of(msg_id: u32) -> u32 {
        msg_id >> 16
    }
}

/// Индексы параметров камеры в массиве DefaultParams/State.Params.
///
/// Строковая таблица: `0x1418ED230`, 27 записей по 0x23 байт.
/// Формула: `offset = base + index * 4` (для float массива).
pub mod camera_params {
    pub const DISTANCE: usize = 0;
    pub const MIN_DISTANCE: usize = 1;
    pub const MAX_OFF_WALL_SHIFT_DISTANCE: usize = 2;
    pub const DISTANCE_EXPONENT: usize = 3;
    pub const FOV: usize = 4;
    pub const MIN_ROT_Z: usize = 5;
    pub const MAX_ROT_Z: usize = 6;
    pub const SLOPE: usize = 7;
    pub const CIRCLE_RADIUS: usize = 8;
    pub const CIRCLE_HEIGHT: usize = 9;
    pub const CIRCLE_SIDE_RIGHT: usize = 10;
    pub const CIRCLE_SIDE_LEFT: usize = 11;
    pub const CIRCLE_MIN_RADIUS: usize = 12;
    pub const ELLIPSE_MAJOR_AXIS: usize = 13;
    pub const ELLIPSE_MINOR_AXIS: usize = 14;
    pub const ELLIPSE_MIN_ANGLE: usize = 15;
    pub const ELLIPSE_MAX_ANGLE: usize = 16;
    pub const SWITCH_SIDE_SPEED: usize = 17;
    pub const FOV_DOWN_START_ANGLE: usize = 18;
    pub const FOV_DOWN_MAX: usize = 19;
    pub const FOLLOW_SPEED: usize = 20;
    pub const AUTO_Z_SPEED: usize = 21;
    pub const PREDICTION: usize = 22;
    pub const DEFAULT_Z_ROT: usize = 23;
    pub const Z_RESPONSE: usize = 24;
    pub const Z_RESP_LIMIT_UP: usize = 25;
    pub const Z_RESP_LIMIT_DOWN: usize = 26;

    /// Общее количество параметров.
    pub const COUNT: usize = 27;
}

/// Native entity factory type bytes — хранятся в entity+0x24 (low byte).
///
/// ЭТО ДВИЖКОВЫЕ ТИПЫ, не Lua enum'ы!
/// Используются WrapperFactory для создания Lua-обёрток.
///
/// Подтверждено: runtime DB scan (2415 entities), IDA SetTypeID xrefs,
/// WrapperFactory RB-tree dump (36 entries).
///
/// Для Lua E_EntityType — см. lua_entity_types.
pub mod factory_types {
    // =========================================================================
    //  Базовые типы (промежуточные, устанавливаются в конструкторной цепочке)
    // =========================================================================

    /// C_Entity base — устанавливается BaseEntity_Construct (0x14039B710)
    /// Промежуточный тип, перезаписывается дочерними конструкторами.
    pub const ENTITY: u8 = 0x01; // Lua=1, wrapper: C_ScriptWrapper

    /// Entity position — промежуточный тип в ActorEntity_Construct (0x14039A7E0)
    pub const ENTITY_POS: u8 = 0x03; // Lua=3, wrapper: C_WrapperEntityPos

    /// Entity dummy — промежуточный тип (SetTypeID 0x04 в sub_14039B830)
    pub const ENTITY_DUMMY: u8 = 0x04; // Lua=4

    /// C_Actor base — промежуточный тип в ActorEntity_Construct
    pub const ACTOR: u8 = 0x05; // Lua=5, wrapper: C_WrapperActor

    // =========================================================================
    //  Human типы (runtime confirmed)
    // =========================================================================

    /// C_HumanNPC — NPC персонажи (Joe, Henry, враги, полиция)
    /// Конструктор: 0x140D712E0, lea edx,[rsi+0Eh] -> 0x0E
    /// Runtime: 86 entities, vtable 0x1418E5188
    pub const HUMAN: u8 = 0x0E; // 14, Lua=14

    /// C_Player — игрок (Vito). Доступ: GameManager+0x180
    /// Конструктор: 0x1400B9160, lea edx,[rsi+10h] -> 0x10
    /// Runtime: 1 entity, vtable 0x14184C060
    pub const PLAYER: u8 = 0x10; // 16, Lua=16

    // =========================================================================
    //  Транспорт (runtime confirmed)
    // =========================================================================

    /// C_Car — базовый класс машины (припаркованные, стоящие)
    /// Конструктор: M2DE_CarEntity_Construct (0x1400EE6C0), xor r14d,r14d; lea edx,[r14+12h]
    /// Runtime: 41 entity, vtable 0x141850030
    /// WrapperFactory: C_WrapperEntityPos (не C_WrapperCar!)
    pub const CAR: u8 = 0x12; // 18, Lua=18

    /// CAR_VEHICLE — расширенный класс машины (управляемый транспорт)
    /// Конструктор: M2DE_CCarVehicle_Construct (0x140DF3360)
    /// Runtime: 1 entity, vtable 0x1418EAAC8
    /// Размер: 0x2F0 (752 bytes)
    /// 5 vtable (multiple inheritance): +0x00, +0xA8, +0xB0, +0xB8, +0xC0
    /// ВНИМАНИЕ: Lua E_EntityType.CAR=18 маппится как на 0x12, так и на 0x70.
    pub const CAR_VEHICLE: u8 = 0x70; // 112, Lua=18, wrapper: C_WrapperCar

    /// C_CrashObject — разрушаемые объекты (бочки, ящики, мебель)
    /// Runtime: 528 entities, vtable 0x1418E8D00
    pub const CRASH_OBJECT: u8 = 0x14; // 20, Lua=20

    /// C_TrafficCar — AI машины трафика
    /// Конструктор: 0x140C125B0, mov edx, 0x15
    /// Runtime: 13 entities, vtable 0x1418D1EC8
    pub const TRAFFIC_CAR: u8 = 0x15; // 21, Lua=21

    /// C_TrafficHuman — AI пешеходы трафика
    /// Runtime: 13 entities, vtable 0x1418D2320
    pub const TRAFFIC_HUMAN: u8 = 0x16; // 22, Lua=22

    /// C_TrafficTrain — AI поезда
    /// Runtime: 1 entity, vtable 0x1418D2090
    pub const TRAFFIC_TRAIN: u8 = 0x17; // 23, Lua=23

    /// TranslocatedCar — перемещённая машина (конструктор 0x14039BCA0, type=0x71)
    pub const TRANSLOCATED_CAR: u8 = 0x71; // 113

    // =========================================================================
    //  ActionPoint типы (runtime confirmed)
    // =========================================================================

    /// C_ActionPoint — WrapperFactory entry для создания C_WrapperActionPoint
    /// НЕ встречается как native type в FreeRide runtime!
    /// Возможно используется в миссиях.
    pub const ACTION_POINT: u8 = 0x19; // 25, Lua=25, wrapper: C_WrapperActionPoint

    /// C_ActionPointRoadBlock — блокпосты
    /// Конструктор: 0x140C270F0, mov edx, 0x1A
    pub const ACTION_POINT_ROADBLOCK: u8 = 0x1A; // 26

    /// C_ActionPoint (base) — базовые точки действия
    /// Runtime: 70 entities, vtable 0x1418CFC20
    pub const ACTION_POINT_BASE: u8 = 0x33; // 51

    /// C_ActionPointCrossing — перекрёстки
    /// Конструктор: 0x140DF2B30, mov edx, 0x34
    /// Runtime: 113 entities, vtable 0x1418E9568
    pub const ACTION_POINT_CROSSING: u8 = 0x34; // 52

    /// C_ActionPointSideWalk — тротуары
    /// Runtime: 91 entities, vtable 0x1418EA8B8
    pub const ACTION_POINT_SIDEWALK: u8 = 0x35; // 53

    /// C_ActionPointScript — скриптовые точки
    /// Runtime: 1 entity, vtable 0x14190CD70
    pub const ACTION_POINT_SCRIPT: u8 = 0x36; // 54

    /// C_ActionPointSearch — точки поиска
    /// Конструктор: 0x140DF1490, mov edx, 0x3F
    /// Runtime: 1 entity, vtable 0x1418EA470
    pub const ACTION_POINT_SEARCH: u8 = 0x3F; // 63

    // =========================================================================
    //  Мир: предметы, двери, деревья (runtime confirmed)
    // =========================================================================

    /// C_DamageZone — внешняя world entity.
    ///
    /// Конструктор: `M2DE_CDamageZone_Construct` (`0x140C0E8A0`)
    /// Runtime: 109 entities, primary vtable `0x1418D0A78`.
    ///
    /// Внутри выделяет дочерний script-entity-like объект размером `0xA0`
    /// с vtable `0x1418D05D8`, затем сохраняет указатель на него в outer object.
    ///
    /// ВАЖНО:
    /// - это native factory type
    /// - Lua mapping для этого типа пока НЕ подтверждён
    /// - наличие отдельного Lua wrapper для DamageZone пока НЕ подтверждено
    pub const DAMAGE_ZONE: u8 = 0x1E; // 30

    /// Telephone registration entity.
    ///
    /// Конструктор: `M2DE_CTelephoneReg_Construct` (`0x140C0E9F0`)
    /// Runtime: 1 entity, primary vtable `0x1418D0D80`.
    ///
    /// Внутри содержит embedded script-entity-like child по смещению `+0x100`,
    /// которому назначается vtable `0x1418D0C58`.
    ///
    /// Дополнительно выделяет вспомогательный объект `0x28` байт
    /// и сохраняет его в области дочернего script child.
    ///
    /// ВАЖНО:
    /// - это native factory type
    /// - Lua mapping для этого типа пока НЕ подтверждён
    /// - наличие отдельного Lua wrapper для TelephoneReg пока НЕ подтверждено
    pub const TELEPHONE_REG: u8 = 0x20; // 32

    /// C_Item — предметы (подбираемые)
    /// Runtime: 35 entities, vtable 0x1418E9C38
    pub const ITEM: u8 = 0x24; // 36, Lua=36, wrapper: C_WrapperItem

    /// C_Wardrobe — гардероб/одежда
    /// Конструктор: 0x140FF7AF0, mov edx, 0x25
    /// Runtime: 1 entity, vtable 0x141909A60
    pub const WARDROBE: u8 = 0x25; // 37

    /// C_Door — двери
    /// Конструктор: M2DE_CDoor_Construct (0x1400EF4F0)
    /// Runtime: 40 entities, vtable 0x141851BD0
    pub const DOOR: u8 = 0x26; // 38, Lua=38

    /// Tree — деревья
    /// Runtime: 99 entities, vtable 0x1418E9700
    pub const TREE: u8 = 0x27; // 39, Lua=39

    /// C_Lift — лифты
    /// Конструктор: M2DE_CLift_Construct (0x1400F00B0), mov edx, 0x28
    /// Runtime: не замечен в FreeRide
    pub const LIFT: u8 = 0x28; // 40, Lua=40

    /// C_Grenade — гранаты
    /// Runtime: 27 entities, vtable 0x14190BB00
    pub const GRENADE: u8 = 0x2F; // 47, Lua=45

    // =========================================================================
    //  Звук (runtime confirmed)
    // =========================================================================

    /// C_Sound — звуковые сущности
    /// Runtime: 247 entities, vtable 0x1418E89D0
    pub const SOUND: u8 = 0x29; // 41, Lua=41, wrapper: C_WrapperSound

    /// C_SoundMixer — микшеры
    /// Runtime: 1 entity, vtable 0x1418EB868
    pub const SOUND_MIXER: u8 = 0x2B; // 43, Lua=43, wrapper: C_WrapperSoundMixer

    // =========================================================================
    //  Оружие / боевые (runtime confirmed)
    // =========================================================================

    /// C_StaticWeapon — статическое оружие (entity type)
    /// Конструктор: 0x1410186B0, mov edx, 0x30
    /// Runtime: 1 entity, vtable 0x14190C720
    pub const STATIC_WEAPON: u8 = 0x30; // 48

    /// C_StaticWeapon — wrapper factory entry (может быть другой тип!)
    /// WrapperFactory key=0x41 -> C_WrapperStaticWeapon
    pub const STATIC_WEAPON_WRAPPER: u8 = 0x41; // 65, Lua=65

    /// C_FireTarget — огневые цели
    /// Конструктор: 0x140E455A0, mov edx, 0x46
    pub const FIRE_TARGET: u8 = 0x46; // 70, Lua=70

    // =========================================================================
    //  Визуальные / эффекты (runtime confirmed)
    // =========================================================================

    /// FrameWrapper — обёртки для фреймов
    /// Конструктор: 0x140C78330, mov edx, 0x37
    /// Runtime: 165 entities, vtable 0x1418D3D58
    pub const FRAME_WRAPPER: u8 = 0x37; // 55, Lua=55

    /// C_ShopMenuScriptEntity
    pub const SHOP_MENU_SCRIPT: u8 = 0x38; // 56

    /// C_StaticParticle — статические частицы
    /// Конструктор: 0x140DF2CC0, mov edx, 0x42
    /// Runtime: 38 entities, vtable 0x1418E9A38
    pub const STATIC_PARTICLE: u8 = 0x42; // 66, Lua=66

    /// LightEntity — источники света
    /// Конструктор: 0x140DF2410, mov edx, 0x47
    /// Runtime: 437 entities, vtable 0x1418E84F0
    pub const LIGHT_ENTITY: u8 = 0x47; // 71, Lua=71, wrapper: C_WrapperLightEntity

    /// C_Pinup — постеры/плакаты
    /// Конструктор: 0x140DF2750, mov edx, 0x6A
    /// Runtime: 18 entities, vtable 0x1418EB198
    pub const PINUP: u8 = 0x6A; // 106, Lua=106

    // =========================================================================
    //  Специальные (runtime confirmed)
    // =========================================================================

    /// C_Cutscene — катсцены (WrapperFactory entry)
    /// Конструктор: 0x140C781A0, mov edx, 0x49
    pub const CUTSCENE: u8 = 0x49; // 73, Lua=73

    /// C_Cutscene entity — в мире
    /// Runtime: 2 entities, vtable 0x1418519C8
    pub const CUTSCENE_ENT: u8 = 0x68; // 104

    /// C_Jukebox — музыкальный автомат
    pub const JUKEBOX: u8 = 0x5C; // 92

    /// Telephone — телефонные будки (entity)
    /// Runtime: 4 entities, vtable 0x1418529D0
    pub const TELEPHONE: u8 = 0x5F; // 95, Lua=95, wrapper: C_WrapperTelephone

    /// Базовое семейство `C_ScriptEntity`.
    ///
    /// Базовый конструктор: `M2DE_CScriptEntity_Construct` (`0x14039BDE0`)
    /// Final type: `0x62`
    /// Базовый alloc size: `0x90`
    /// Базовая vtable: `0x14186E170`
    ///
    /// Runtime: в мире найдено 200 объектов family `0x62`.
    ///
    /// ВАЖНО:
    /// не все связанные с этим семейством vtable принадлежат standalone top-level entity.
    /// Часть из них используется как дочерние script-entity-like объекты внутри других классов:
    ///
    /// - base `C_ScriptEntity`:        `0x14186E170`
    /// - DamageZone child:             `0x1418D05D8`
    /// - TelephoneReg child:           `0x1418D0C58`
    /// - PhoneCalls child / host path: `0x1418EAFF8`
    /// - direct Sub5 path:             `0x14184B230`
    ///
    /// То есть `SCRIPT_ENTITY` сейчас нужно понимать как family/базу,
    /// а не как один-единственный concrete class.
    ///
    /// Lua mapping:
    /// - `Lua EntityType.SCRIPT = 98` подтверждён
    /// - наличие отдельного `C_WrapperScriptEntity` пока НЕ подтверждено
    pub const SCRIPT_ENTITY: u8 = 0x62; // 98, Lua=98

    /// C_Blocker — блокирующие объекты
    /// Runtime: 7 entities, vtable 0x1418739A0
    pub const BLOCKER: u8 = 0x64; // 100

    /// C_ActorDetector — детекторы актёров
    /// Конструктор: 0x14045E5E0, mov edx, 0x65
    /// Runtime: 7 entities, vtable 0x141873AB0
    pub const ACTOR_DETECTOR: u8 = 0x65; // 101

    /// C_Boat — лодки
    pub const BOAT: u8 = 0x67; // 103

    /// C_DummyDoor — фальшивые двери
    /// Runtime: 2 entities, vtable 0x14190D2C0
    pub const DUMMY_DOOR: u8 = 0x6B; // 107

    /// StaticEntity — статические объекты
    /// Конструктор: 0x140C0E870, mov edx, 0x6C
    /// Runtime: 13 entities, vtable 0x1418D0F40
    pub const STATIC_ENTITY: u8 = 0x6C; // 108

    /// C_FramesController — контроллер фреймов
    pub const FRAMES_CONTROLLER: u8 = 0x6E; // 110, Lua=110

    /// C_CleanEntity — «чистая» сущность (минимальный набор компонентов)
    /// Конструктор: 0x140C78590, mov edx, 0x6F
    /// Runtime: 1 entity, vtable 0x1418D34B0
    pub const CLEAN_ENTITY: u8 = 0x6F; // 111

    /// C_CleanEntity — альтернативный тип (WrapperFactory entry)
    /// WrapperFactory key=0x72 -> C_WrapperCleanEntity
    pub const CLEAN_ENTITY_ALT: u8 = 0x72; // 114

    // =========================================================================
    //  Редкие / не подтверждённые в FreeRide
    // =========================================================================

    /// Неизвестный тип 0x13 — конструктор 0x14103A136
    pub const UNKNOWN_13: u8 = 0x13; // 19

    /// C_PhysicsScene? — конструктор 0x140FE0030, mov edx, 0x73
    pub const PHYSICS_SCENE: u8 = 0x73; // 115

    /// C_Train
    pub const TRAIN: u8 = 0x76; // 118

    /// C_Radio
    pub const RADIO: u8 = 0x77; // 119

    /// C_Garage
    pub const GARAGE: u8 = 0x78; // 120

    /// C_SpikeStrip
    pub const SPIKE_STRIP: u8 = 0x79; // 121
}

/// Lua E_EntityType enum values. Используются скриптами, значения из float констант.
/// Некоторые типы (например, 18=car) не совпадают с factory types, т.к. в Lua они определяются по другим правилам.
pub mod lua_entity_types {
    pub const UNKNOWN: u32 = 0;
    pub const ENTITY: u32 = 1;
    pub const MULTIPLAYER_ENTITY: u32 = 2;
    pub const ENTITY_POS: u32 = 3;
    pub const ENTITY_DUMMY: u32 = 4;
    pub const ACTOR: u32 = 5;
    pub const ACTOR_MULTIFRAME: u32 = 6;
    pub const TICKED_MODULE: u32 = 7;
    pub const TICKED_MODULE_MANAGER: u32 = 8;
    pub const SYSTEM_INIT_DONE: u32 = 9;
    pub const GAME: u32 = 10;
    pub const MISSION: u32 = 11;
    pub const EDITOR_COMM: u32 = 12;
    pub const BASE_HUMAN: u32 = 13;
    pub const HUMAN: u32 = 14;
    pub const PLAYER: u32 = 16;
    pub const CAR: u32 = 18;
    pub const TRAIN: u32 = 19;
    pub const CRASH_OBJECT: u32 = 20;
    pub const TRAFFIC_CAR: u32 = 21;
    pub const TRAFFIC_HUMAN: u32 = 22;
    pub const TRAFFIC_TRAIN: u32 = 23;
    pub const TRAFFIC_BASE: u32 = 24;
    pub const ACTION_POINT: u32 = 25;
    pub const GLASS_MANAGER: u32 = 33;
    pub const MULTIPLAYER_SPAWN_POINT: u32 = 34;
    pub const MULTIPLAYER_SPAWN_NEST: u32 = 35;
    pub const ITEM: u32 = 36;
    pub const GLASSBREAKING: u32 = 37;
    pub const DOOR: u32 = 38;
    pub const TREE: u32 = 39;
    pub const LIFT: u32 = 40;
    pub const SOUND: u32 = 41;
    pub const SOUND_SELECTOR: u32 = 42;
    pub const SOUND_MIXER: u32 = 43;
    pub const ANIMATOR: u32 = 44;
    pub const GRENADE: u32 = 45;
    pub const FRAME_WRAPPER: u32 = 55;
    pub const MAFIA_FIRST: u32 = 57;
    pub const PATHGEN: u32 = 58;
    pub const MUSIC: u32 = 59;
    pub const MOSCOW_FIRST: u32 = 61;
    pub const TRIGGER: u32 = 62;
    pub const EXPLOSION: u32 = 64;
    pub const STATIC_WEAPON: u32 = 65;
    pub const STATIC_PARTICLE: u32 = 66;
    pub const DEMOLITION: u32 = 67;
    pub const FRACTURING: u32 = 68;
    pub const HELICOPTER: u32 = 69;
    pub const FIRETARGET: u32 = 70;
    pub const LIGHTENTITY: u32 = 71;
    pub const BARREL: u32 = 72;
    pub const CUTSCENE: u32 = 73;
    pub const ANIMENTITY: u32 = 74;
    pub const TVCAMERA: u32 = 75;
    pub const MONITOR: u32 = 76;
    pub const MISSIONSTATE: u32 = 77;
    pub const CLOCK_SIMPLE: u32 = 78;
    pub const CLOCK_CRASHOBJ: u32 = 79;
    pub const RESPAWNPOINT: u32 = 80;
    pub const PRESSURE_TANK: u32 = 81;
    pub const TELEPHONE: u32 = 95;
    pub const HITECH_FIRST: u32 = 96;
    pub const SCRIPTMACHINE: u32 = 97;
    pub const SCRIPT: u32 = 98;
    pub const PINUP: u32 = 106;
    pub const DUMMY_DOOR: u32 = 109;
    pub const FRAMES_CONTROLLER: u32 = 110;
    pub const LAST_ID: u32 = 117;
}

/// Lua E_EntityMessageType enum values.
/// используется скриптами, значения из float констант.
pub mod lua_entity_message_types {
    pub const BASEMESSAGE: u32 = 0;
    pub const SCRIPTWAKE: u32 = 1;
    pub const ACTOR: u32 = 2;
    pub const HUMAN: u32 = 3;
    pub const CAR: u32 = 4;
    pub const DOOR: u32 = 5;
    // 6 = gap
    pub const QUEST: u32 = 7;
    pub const AISTATE: u32 = 8;
    pub const FIRETARGET: u32 = 9;
    pub const SYNCOBJECTEVENT: u32 = 10;
    pub const TELEPHONE: u32 = 11;
    pub const ACTORDETECTOR: u32 = 12;
    pub const PINUP: u32 = 13;
    pub const SKIPSCRIPTSCENE: u32 = 14;
}

/// Actor message IDs (packed: event_type=5, base=0x50000).
/// These are the SAME format as human_messages (event_type << 16 | sub_id).
pub mod actor_messages {
    pub const DAMAGE: u32 = 0x50001; // 327681
    pub const INNERSPACE: u32 = 0x50002; // 327682
    pub const MPCHANGECONTROL: u32 = 0x50003; // 327683
    pub const ACTIVATE: u32 = 0x50004; // 327684
    pub const SCRIPT: u32 = 0x50005; // 327685
    pub const HUMANCAMPARAM: u32 = 0x50006; // 327686
    pub const HUMANAIM: u32 = 0x50007; // 327687
    pub const CARCAMPARAM: u32 = 0x50008; // 327688
    pub const SUSPEND: u32 = 0x50009; // 327689
    pub const HITINFO: u32 = 0x5000A; // 327690
    pub const SET_SCRIPT_MACHINE: u32 = 0x5000B; // 327691
    pub const TEST_ACTION: u32 = 0x50015; // 327701
}

/// DLC типы (0–10), используется в M2DE_GetDLCNameById и в Lua `enums.DLCType`.
pub mod dlc_types {
    pub const CAR: u32 = 0;
    pub const SUIT: u32 = 1;
    pub const MISSION_PACK: u32 = 2;
    pub const FAMILY_ALBUM: u32 = 3;
    pub const WALLPAPER: u32 = 4;
    pub const CAR_CHARGER: u32 = 5;
    pub const CAR_PAINTING: u32 = 6;
    pub const PLAYER: u32 = 7;
    pub const MEDAL_PACK: u32 = 8;
    pub const PRESENCE_PACK: u32 = 9;
    pub const UNKNOWN: u32 = 10;
}

/// Моудьные ID для M2DE_GetModuleNameById (0–47).
pub mod module_ids {
    pub const GAME_MAIN_FIRST: u32 = 0;
    pub const STREAMING_POS_MANAGER: u32 = 1;
    pub const STREAM_MAPA: u32 = 2;
    pub const SYSTEM_INIT_DONE: u32 = 3;
    pub const GAME_GUI_MODULE: u32 = 4;
    pub const GFX_EFF_MODULE: u32 = 5;
    pub const APACK_SCRIPT_MACHINE_MGR: u32 = 6;
    pub const ITEM_MANAGER: u32 = 7;
    pub const SHOP_MENU_MANAGER: u32 = 8;
    pub const ENTITY_LIST: u32 = 9;
    pub const SCRIPT_MODULE: u32 = 10;
    pub const SLOT_MGR1: u32 = 11;
    pub const GAME_MAIN1: u32 = 12;
    pub const SDS_LOADING_TABLE: u32 = 13;
    pub const SLOT_MGR: u32 = 14;
    pub const TRANSLOCATOR_MGR: u32 = 15;
    pub const DELAYED_SO_MANAGER: u32 = 16;
    pub const SCRIPT_DATA_STORAGE: u32 = 17;
    pub const GARAGE_MANAGER: u32 = 18;
    pub const TEAMAI_MODULE: u32 = 19;
    pub const GAMEAI_MODULE: u32 = 20;
    pub const POLICE_AI: u32 = 21;
    pub const RANGE_METER: u32 = 22;
    pub const HUD: u32 = 23;
    pub const GAME_MAIN: u32 = 24;
    pub const PLAYER_MODEL_MANAGER: u32 = 25;
    pub const GUI_NAVIGATION: u32 = 26;
    pub const MAP: u32 = 27;
    pub const QUEST_MANAGER: u32 = 28;
    pub const ENTITY_MSG_DISPATCHER: u32 = 29;
    pub const SDS_LOAD_UNLOAD_NOTIFY: u32 = 30;
    pub const CUTSCENES_MODULE: u32 = 31;
    pub const TELEPHONE_SPEECH_MANAGER: u32 = 32;
    pub const CITY_SHOPS: u32 = 33;
    pub const DRIVER_AI: u32 = 34;
    pub const CAMERA_MODULE: u32 = 35;
    pub const WARDROBE_MODULE: u32 = 36;
    pub const NAVIGATION_MODULE: u32 = 37;
    pub const SHOT_MANAGER: u32 = 38;
    pub const PLAYER_GUI_HELPERS: u32 = 39;
    pub const SPEECH_SLOT_MANAGER: u32 = 40;
    pub const GAME_TRAFFIC_MODULE: u32 = 41;
    pub const TEXT_DATABASE: u32 = 42;
    pub const GAME_AUDIO_MODULE: u32 = 43;
    pub const PHYSICS_MODULE: u32 = 44;
    pub const RTR_MODULE: u32 = 45;
    pub const MODEL_HILIGHTING_MODULE: u32 = 46;
    // 47 = gap
    pub const LAST: u32 = 48;
}



/// Битовая разметка player+0x490.
///
/// Семантика битов пока не названа,
/// но структура bitfield подтверждена decompile'ом.
pub mod player_state_flags_490 {
    /// Биты [1..3]
    pub const MASK_BITS_1_3: u32 = 0x0000_000E;

    /// Биты [4..6]
    pub const MASK_BITS_4_6: u32 = 0x0000_0070;

    /// Биты [7..13]
    pub const MASK_BITS_7_13: u32 = 0x0000_3F80;

    /// Бит 14
    pub const BIT_14: u32 = 0x0000_4000;

    /// Бит 15
    pub const BIT_15: u32 = 0x0000_8000;
}

// =============================================================================
//  C_Car message codes
// =============================================================================

/// Коды сообщений C_Car (из IDA decompile vtable C_Car).
///
/// Формат: event_type=4 (CAR), base=0x40000.
/// Используются в entity message dispatch системе.
pub mod car_messages {
    /// Базовое entity event (0x40001).
    pub const ENTITY_EVENT: u32 = 0x40001;

    /// Перенаправить в behavior (0x40002).
    pub const FORWARD_TO_BEHAVIOR: u32 = 0x40002;

    /// Создать физику (0x40003).
    pub const CREATE_PHYSICS: u32 = 0x40003;

    /// Создать wrapper (0x40004).
    pub const CREATE_WRAPPER: u32 = 0x40004;

    /// Перенаправить из behavior (0x40005).
    pub const BEHAVIOR_FORWARD: u32 = 0x40005;

    /// Пустая операция (0x40006).
    pub const NOOP: u32 = 0x40006;

    /// Урон до смерти (0x40007).
    pub const KILL_DAMAGE: u32 = 0x40007;

    /// Маркер физики (0x40008).
    pub const PHYSICS_MARKER: u32 = 0x40008;

    /// Переключить флаг 41 (0x40009).
    pub const TOGGLE_FLAG_41: u32 = 0x40009;
}

/// Типы деталей повреждений C_Car (crash part codes).
/// Используются в CCar::CreateCrashPart (vtable[89]).
pub mod car_crash_parts {
    pub const BODY: u32 = 0;
    pub const BODY_ARMORED: u32 = 1;
    pub const WHEEL: u32 = 2;
    pub const LID: u32 = 3;        // капот/багажник
    pub const DOOR: u32 = 4;
    pub const WINDOW: u32 = 5;
    pub const COVER: u32 = 6;
    pub const BUMPER: u32 = 7;
    // 8-10 = gap
    pub const DOOR_PART: u32 = 11;  // 0xB, фрагмент двери
    pub const EXHAUST: u32 = 12;    // 0xC
    pub const MOTOR: u32 = 13;      // 0xD
    pub const TYRE: u32 = 14;       // 0xE
    pub const SNOW: u32 = 15;       // 0xF, снежное покрытие
    pub const PLOW: u32 = 16;       // 0x10, снегоуборщик

    /// Размеры аллокации по типу.
    pub const fn alloc_size(part_type: u32) -> usize {
        match part_type {
            0 | 1 | 11 | 15 | 16 => 336,
            2 | 3 | 14 => 360,
            4 => 504,
            5 | 12 => 424,
            6 => 480,
            7 => 368,
            13 => 352,
            _ => 0,
        }
    }
}

/// Флаги crash part (+0x10).
pub mod car_crash_flags {
    /// Деталь может полностью отделиться от кузова.
    pub const DETACHABLE: u32 = 0x80000000;
    /// Физическая деталь (требует физический контакт для повреждения).
    pub const PHYSICAL: u32 = 0x08;
    /// Специальная деталь (снегоуборщик и т.д.).
    pub const SPECIAL: u32 = 0x20000000;
    /// Разбиваемое стекло (окно может разбиться, но не отделиться).
    pub const BREAKABLE: u32 = 0x10;
}