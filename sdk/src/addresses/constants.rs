//! Игровые константы: ID оружия, машин, слотов.

/// ID оружия для `Player::add_weapon`.
pub mod weapons {
    // Пистолеты
    pub const MODEL_12_REVOLVER: u32    = 2;
    pub const MAUSER_C96: u32           = 3;
    pub const COLT_M1911A1: u32         = 4;
    pub const COLT_M1911_SPECIAL: u32   = 5;
    pub const MODEL_19_REVOLVER: u32    = 6;

    // Дробовики
    pub const REMINGTON_870: u32        = 8;

    // Пистолеты-пулемёты
    pub const M3_GREASE_GUN: u32        = 9;
    pub const MP40: u32                 = 10;
    pub const THOMPSON_1928: u32        = 11;
    pub const M1A1_THOMPSON: u32        = 12;
    pub const BERETTA_38A: u32          = 13;

    // Винтовки
    pub const M1_GARAND: u32            = 15;
    pub const KAR98K: u32               = 17;

    // Гранаты
    pub const MK2_FRAG_GRENADE: u32     = 20;
    pub const MOLOTOV_COCKTAIL: u32     = 21;
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
    pub const ACTIVE: i32   = 1;
    /// Создан, но не заспавнен.
    pub const DEFERRED: i32 = 5;
    /// Только что сконструирован.
    pub const INITIAL: i32  = 6;
}

/// ID машин для гаража (0–33).
///
/// Только эти 34 машины могут храниться в гараже.
/// Регистрируются через `GetVehicleIDByName` (`0x14101D0A0`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageVehicleId {
    AscotBaileys200Pha       = 0,
    BerkleyKingfisherPha     = 1,
    DeliziaGrandeamerica     = 2,
    HotRod1                  = 3,
    HotRod2                  = 4,
    HotRod3                  = 5,
    HoustonWaspPha           = 6,
    Isw508                   = 7,
    Jeep                     = 8,
    JeepCivil                = 9,
    JeffersonFuturaPha       = 10,
    JeffersonProvincial      = 11,
    Lassiter69               = 12,
    Lassiter75Pha            = 13,
    PotomacIndian            = 14,
    QuicksilverWindsorPha    = 15,
    QuicksilverWindsorTaxi   = 16,
    Shubert38                = 17,
    ShubertBeverly           = 18,
    ShubertFrigatePha        = 19,
    ShubertHearse            = 20,
    ShubertPanel             = 21,
    ShubertPickup            = 22,
    ShubertTaxi              = 23,
    Smith200PPha             = 24,
    Smith200Pha              = 25,
    SmithMainlinePha         = 26,
    SmithStingrayPha         = 27,
    SmithV8                  = 28,
    SmithWagonPha            = 29,
    UlverNewyorker           = 30,
    UlverNewyorkerP          = 31,
    WalkerRocket             = 32,
    WalterCoupe              = 33,
}

impl GarageVehicleId {
    /// Имя машины для передачи в игровые функции.
    pub fn name(self) -> &'static str {
        match self {
            Self::AscotBaileys200Pha     => "Ascot_Baileys_200_pha",
            Self::BerkleyKingfisherPha   => "Berkley_Kingfisher_pha",
            Self::DeliziaGrandeamerica   => "Delizia_Grandeamerica",
            Self::HotRod1                => "Hot_Rod_1",
            Self::HotRod2                => "Hot_Rod_2",
            Self::HotRod3                => "Hot_Rod_3",
            Self::HoustonWaspPha         => "Houston_Wasp_pha",
            Self::Isw508                 => "ISW_508",
            Self::Jeep                   => "Jeep",
            Self::JeepCivil              => "Jeep_civil",
            Self::JeffersonFuturaPha     => "Jefferson_Futura_pha",
            Self::JeffersonProvincial    => "Jefferson_Provincial",
            Self::Lassiter69             => "Lassiter_69",
            Self::Lassiter75Pha          => "Lassiter_75_pha",
            Self::PotomacIndian          => "Potomac_Indian",
            Self::QuicksilverWindsorPha  => "Quicksilver_Windsor_pha",
            Self::QuicksilverWindsorTaxi => "Quicksilver_Windsor_taxi_pha",
            Self::Shubert38              => "Shubert_38",
            Self::ShubertBeverly         => "Shubert_Beverly",
            Self::ShubertFrigatePha      => "Shubert_Frigate_pha",
            Self::ShubertHearse          => "Shubert_Hearse",
            Self::ShubertPanel           => "Shubert_Panel",
            Self::ShubertPickup          => "Shubert_Pickup",
            Self::ShubertTaxi            => "Shubert_Taxi",
            Self::Smith200PPha           => "Smith_200_p_pha",
            Self::Smith200Pha            => "Smith_200_pha",
            Self::SmithMainlinePha       => "Smith_Mainline_pha",
            Self::SmithStingrayPha       => "Smith_Stingray_pha",
            Self::SmithV8                => "Smith_V8",
            Self::SmithWagonPha          => "Smith_Wagon_pha",
            Self::UlverNewyorker         => "Ulver_Newyorker",
            Self::UlverNewyorkerP        => "Ulver_Newyorker_p",
            Self::WalkerRocket           => "Walker_Rocket",
            Self::WalterCoupe            => "Walter_Coupe",
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
    pub const DAMAGE: u32                = 851_984;  // 0xD0010
    pub const DEATH: u32                 = 851_988;  // 0xD0014
    pub const ANIM_NOTIFY: u32           = 851_989;  // 0xD0015
    pub const ENTER_VEHICLE: u32         = 851_995;  // 0xD001B
    pub const LEAVE_VEHICLE: u32         = 851_996;  // 0xD001C
    pub const ENTER_VEHICLE_DONE: u32    = 851_997;  // 0xD001D
    pub const LEAVE_VEHICLE_DONE: u32    = 851_998;  // 0xD001E
    pub const PLAYER_WEAPON_SELECT: u32  = 852_004;  // 0xD0024
    pub const PLAYER_WEAPON_HIDE: u32    = 852_005;  // 0xD0025
    pub const SHOT: u32                  = 852_071;  // 0xD0057

    pub const WEAPON_HOLSTER: u32        = 851_999;  // 0xD001F  conf 78%
    pub const WEAPON_DRAW: u32           = 852_000;  // 0xD0020  conf 80%
    pub const STANCE_CHANGE: u32         = 852_001;  // 0xD0021  conf 72%
    pub const STANCE_CLEANUP: u32        = 852_002;  // 0xD0022  conf 68%
    pub const STANCE_SECONDARY: u32      = 852_003;  // 0xD0023  conf 68%
    pub const HEAD_DAMAGE: u32           = 852_033;  // 0xD0041  conf 85%
    pub const BODY_DAMAGE: u32           = 852_034;  // 0xD0042  conf 85%
    pub const KILL_DAMAGE: u32           = 852_035;  // 0xD0043  conf 80%

    pub const HUMAN_MODE_CHANGE: u32     = 851_972;  // 0xD0004  conf 70%
    pub const HUMAN_TICK: u32            = 851_985;  // 0xD0011  conf 78%
    pub const HUMAN_SETTLED: u32         = 851_994;  // 0xD001A  conf 72%
}

/// Диапазоны event_type для быстрой классификации message_id.
pub mod message_ranges {
    /// Event type 5: low-level entity messages.
    /// message_id & 0xFFFF0000 == 0x50000
    pub const LOW_LEVEL_BASE: u32 = 0x50000;

    /// Event type 3: HUMAN messages.
    /// message_id & 0xFFFF0000 == 0xD0000
    pub const HUMAN_BASE: u32 = 0xD0000;

    /// Event type 18: traffic/AI messages.
    /// message_id & 0xFFFF0000 == 0x120000
    pub const TRAFFIC_BASE: u32 = 0x120000;

    /// Быстрая классификация по event_type.
    pub fn event_type_of(msg_id: u32) -> u32 {
        msg_id >> 16
    }
}