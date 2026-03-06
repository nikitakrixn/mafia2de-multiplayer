//! Сетевой протокол для Mafia II: DE Multiplayer.
//!
//! Этот крейт содержит определения пакетов и типов,
//! общих для клиента и сервера.

/// Версия протокола. Клиент и сервер должны совпадать.
pub const PROTOCOL_VERSION: u32 = 1;

/// Порт сервера по умолчанию.
pub const DEFAULT_PORT: u16 = 7788;

/// Максимальное количество игроков.
pub const MAX_PLAYERS: usize = 32;

/// Идентификатор игрока на сервере.
pub type PlayerId = u16;

/// Пакет от клиента к серверу.
#[derive(Debug, Clone)]
pub enum ClientPacket {
    /// Запрос на подключение.
    Connect { name: String, version: u32 },
    /// Отключение.
    Disconnect,
    /// Обновление позиции игрока.
    PlayerUpdate { position: [f32; 3], rotation: f32 },
    /// Текстовое сообщение в чат.
    ChatMessage { text: String },
}

/// Пакет от сервера к клиенту.
#[derive(Debug, Clone)]
pub enum ServerPacket {
    /// Подтверждение подключения.
    ConnectAccepted { player_id: PlayerId },
    /// Отказ в подключении.
    ConnectRejected { reason: String },
    /// Информация о другом игроке.
    PlayerSpawn {
        player_id: PlayerId,
        name: String,
        position: [f32; 3],
    },
    /// Обновление позиции другого игрока.
    PlayerUpdate {
        player_id: PlayerId,
        position: [f32; 3],
        rotation: f32,
    },
    /// Игрок отключился.
    PlayerDisconnect { player_id: PlayerId },
    /// Сообщение в чат.
    ChatMessage {
        player_id: PlayerId,
        text: String,
    },
}