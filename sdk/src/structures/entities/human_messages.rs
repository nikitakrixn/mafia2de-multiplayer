//! Message IDs обрабатываемые `C_Human2::RecvMessage` и `C_Player2::RecvMessage`.
//!
//! ## Как работает система сообщений
//!
//! `C_Human2::RecvMessage(C_EntityMessage*)` — главный роутер сообщений гуманоида.
//! Каждое сообщение имеет 32-битный ID (`*(msg + 0x20)`), по которому ведётся
//! dispatch через switch-statement.
//!
//! ## Диапазоны ID
//!
//! | Диапазон | Hex | Назначение |
//! |:---------|:----|:-----------|
//! | 327681 | `0x50001` | Damage entry -> `DoDamage(C_EntityMessageDamage*)` |
//! | 851973..851988 | `0xD0005..0xD0014` | AI percept-сообщения (`AIController::NewPercept`) |
//! | 851983 | `0xD000F` | Reset detector (sight range changed) |
//! | 851984 | `0xD0010` | Damage forwarding to registered |
//! | 852033 | `0xD0041` | Damage type 16 special broadcast |
//! | 852034 | `0xD0042` | Damage type 17 special broadcast |
//! | 852038 | `0xD0046` | `HumanCoat::ApplyCorrectionPose` |
//! | 852056 | `0xD0058` | Activate/Deactivate emitter slot 31 |
//! | 852057 | `0xD0059` | Create/destroy optional emitter slot 32 |
//! | 852058 | `0xD005A` | Update emitter slot 32 parameter |
//! | 852059..852076 | `0xD005B..0xD006C` | Secondary AI percept diapazon |
//! | 852071..852075 | `0xD0067..0xD006B` | AI percepts (повторный диапазон) |
//! | 852077 | `0xD006D` | Navigation minimap back/fore color update |

/// Константы ID сообщений для `C_Human2::RecvMessage`.
#[allow(dead_code)]
pub mod human_message_id {
    /// Урон — точка входа в `DoDamage(C_EntityMessageDamage*)`.
    ///
    /// Это стандартный `C_EntityMessage` с damage payload.
    pub const DAMAGE: u32 = 0x50001;

    /// Начало диапазона AI percept-сообщений.
    pub const AI_PERCEPT_BEGIN: u32 = 0xD0005;
    /// Конец диапазона AI percept-сообщений (включительно).
    pub const AI_PERCEPT_END: u32 = 0xD0014;

    /// Сброс detector'а при изменении дальности видимости.
    pub const SIGHT_RESET_DETECTOR: u32 = 0xD000F;

    /// Форвард сообщения об уроне в зарегистрированные объекты.
    pub const DAMAGE_FORWARD: u32 = 0xD0010;

    /// Специальный broadcast для типа урона 16 (смерть игрока?).
    pub const DAMAGE_TYPE16_BROADCAST: u32 = 0xD0041;
    /// Специальный broadcast для типа урона 17.
    pub const DAMAGE_TYPE17_BROADCAST: u32 = 0xD0042;

    /// Вызов `HumanCoat::ApplyCorrectionPose` (физика пальто/плаща).
    pub const COAT_CORRECTION_POSE: u32 = 0xD0046;

    /// Включить/выключить AI emitter слот 31 (sight emitter).
    pub const EMITTER31_TOGGLE: u32 = 0xD0058;
    /// Создать/удалить опциональный emitter слот 32.
    pub const EMITTER32_CREATE_DESTROY: u32 = 0xD0059;
    /// Обновить параметр emitter слота 32.
    pub const EMITTER32_UPDATE_PARAM: u32 = 0xD005A;

    /// Начало вторичного диапазона AI percept.
    pub const AI_PERCEPT2_BEGIN: u32 = 0xD005B;
    /// Конец вторичного диапазона AI percept (включительно).
    pub const AI_PERCEPT2_END: u32 = 0xD006C;

    /// Обновление цвета иконки гуманоида на миникарте навигации.
    pub const NAVIGATION_MAP_COLOR: u32 = 0xD006D;
}

/// Alias для наиболее часто используемого ID — стандартный урон.
pub const DAMAGE_MSG_ID: u32 = human_message_id::DAMAGE;
