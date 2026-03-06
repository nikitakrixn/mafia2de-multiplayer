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