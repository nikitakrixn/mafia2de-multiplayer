//! Сетевой протокол для Mafia II: DE Multiplayer.
//!
//! ВАЖНО:
//! Это пока transport-agnostic уровень:
//! - только типы пакетов
//! - без сериализации
//! - без сокетов

/// Версия протокола.
pub const PROTOCOL_VERSION: u32 = 2;

/// Порт сервера по умолчанию.
pub const DEFAULT_PORT: u16 = 7788;

/// Максимальное количество игроков.
pub const MAX_PLAYERS: usize = 32;

/// Идентификатор игрока на сервере.
pub type PlayerId = u16;

/// Простой сетевой Vec3.
///
/// Не тянем сюда `sdk::game::player::Vec3`,
/// потому что protocol должен оставаться независимым от SDK.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NetVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Снапшот игрока для синхронизации по сети.
///
/// Это минимальный набор подтверждённых reverse'ом полей,
/// которые реально полезны для proxy remote-player.
///
/// Намеренно НЕ включаем сюда:
/// - сырые указатели
/// - неизвестные поля
/// - внутренности ActionCodeManager
/// - всё подряд из player state cluster
#[derive(Debug, Clone)]
pub struct NetPlayerSnapshot {
    /// Локальный tick/sequence number отправителя.
    pub tick: u64,

    /// ID игрока в сессии.
    pub player_id: PlayerId,

    /// Мировая позиция игрока.
    pub position: NetVec3,

    /// Forward direction.
    ///
    /// Достаточно для разворота remote proxy.
    pub forward: NetVec3,

    /// Здоровье.
    pub health: f32,

    /// Мёртв ли игрок.
    pub is_dead: bool,

    /// Главный state code (`player + 0x430`).
    pub state_code_430: u32,

    /// Player state/flags (`player + 0x3D8`).
    pub state_flags_3d8: u32,

    /// Player bitfield (`player + 0x490`).
    pub state_flags_490: u32,

    /// `player.sub45c.state` (`player + 0x464`).
    pub sub45c_state: u32,

    /// Находится ли игрок в машине.
    pub in_vehicle: bool,
}

/// Высокоуровневые сетевые события игрока.
#[derive(Debug, Clone)]
pub enum NetPlayerEvent {
    EnterVehicle,
    EnterVehicleDone,
    LeaveVehicle,
    LeaveVehicleDone,

    Damage,
    Death,

    Shot,
    WeaponSelect,
    WeaponHide,

    /// Триггер PlayerFx-style события.
    Fx(u16),
}

/// Пакет от клиента к серверу.
#[derive(Debug, Clone)]
pub enum ClientPacket {
    /// Запрос на подключение.
    Connect {
        name: String,
        version: u32,
    },

    /// Отключение.
    Disconnect,

    /// Snapshot локального игрока.
    Snapshot(NetPlayerSnapshot),

    /// Event локального игрока.
    Event(NetPlayerEvent),

    /// Чат.
    ChatMessage {
        text: String,
    },

    /// Старый packet. TODO: Убрать
    PlayerUpdate {
        position: [f32; 3],
        rotation: f32,
    },
}

/// Пакет от сервера к клиенту.
#[derive(Debug, Clone)]
pub enum ServerPacket {
    /// Подключение принято.
    ConnectAccepted {
        player_id: PlayerId,
    },

    /// Подключение отвергнуто.
    ConnectRejected {
        reason: String,
    },

    /// Информация о появлении нового игрока.
    PlayerSpawn {
        player_id: PlayerId,
        name: String,
    },

    /// Игрок отключился.
    PlayerDespawn {
        player_id: PlayerId,
    },

    /// Snapshot удалённого игрока.
    Snapshot(NetPlayerSnapshot),

    /// Event удалённого игрока.
    Event {
        player_id: PlayerId,
        event: NetPlayerEvent,
    },

    /// Чат.
    ChatMessage {
        player_id: PlayerId,
        text: String,
    },

    /// Старый packet. TODO: Убрать
    PlayerUpdate {
        player_id: PlayerId,
        position: [f32; 3],
        rotation: f32,
    },
}
