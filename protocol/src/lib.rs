//! Сетевой протокол для Mafia II: DE Multiplayer.
//!
//! v1 transport:
//! - JSON packets over TCP
//! - line-delimited framing (`\n`)
//!
//! ВАЖНО:
//! protocol не зависит от sdk, поэтому используем собственные простые типы.

use serde::{Deserialize, Serialize};

/// Версия протокола.
///
/// v4: добавлены поля `is_aiming` / `aim_dir` в `NetPlayerSnapshot`
/// для синхронизации прицеливания через `C_Human2::SetupAimDir`.
/// v5: добавлено поле `is_moving` (ground truth движения вместо расстояния
///      между snapshot'ами — фикс walking-on-spot при стоянии).
/// v6: добавлено поле `movement_mode` — сырой байт режима шага (DE), см. SDK
///     `Player::get_movement_mode_byte` / `fields::shuman_command_move_dir`.
pub const PROTOCOL_VERSION: u32 = 6;

/// Порт сервера по умолчанию.
pub const DEFAULT_PORT: u16 = 7788;

/// Максимальное количество игроков.
pub const MAX_PLAYERS: usize = 32;

/// Идентификатор игрока на сервере.
pub type PlayerId = u16;

/// Простой сетевой Vec3.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct NetVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Snapshot игрока.
///
/// Минимальный multiplayer-useful набор подтверждённых reverse'ом данных.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub state_code: u32,

    /// Player state/flags (`player + 0x3D8`).
    pub car_wrapper_state: u8,

    /// Player bitfield (`player + 0x490`).
    pub ctrl_style_mask: u32,

    /// `player.sub45c.state` (`player + 0x464`).
    pub sub45c_state: u32,

    /// Находится ли игрок в машине.
    pub in_vehicle: bool,

    /// Прицеливается ли игрок сейчас (LMB / aim hold / cover-aim / mounted weapon).
    ///
    /// Источник: бит `0x2` в `state_flags_510` (CPlayer +0x510)
    /// ИЛИ `state_code in {4, 9}` (cover, mounted weapon).
    pub is_aiming: bool,

    /// Направление прицела (нормализованный 3D-вектор).
    ///
    /// `None` если игрок не целится (`is_aiming = false`).
    /// `Some(dir)` — направление, в которое смотрит ствол / прицел.
    pub aim_dir: Option<NetVec3>,

    /// Реально ли игрок сейчас идёт.
    ///
    /// Источник правды — `PlayerTracker.moving`, который основан на
    /// hysteresis по distance + STOP_TICKS. В отличие от distance per snapshot,
    /// это поле не реагирует на capsule-drift при стоянии и фикс'ит
    /// walking-on-spot анимацию у remote NPC.
    pub is_moving: bool,

    /// Сырой байт режима движения (DE): младший байт `S_HumanCommandMoveDir+0x68`
    /// у локального игрока после `UpdateInput`/`UpdateMoveCmd`.
    ///
    /// Литералы (walk/jog/sprint) в SDK **не** именованы — только pass-through для MP.
    /// Старые клиенты без поля получают `0` (`serde(default)`).
    #[serde(default)]
    pub movement_mode: u8,
}

/// Высокоуровневые события игрока.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    /// Первый пакет после подключения.
    Connect { name: String, version: u32 },

    /// Явное отключение.
    Disconnect,

    /// Snapshot локального игрока.
    Snapshot(NetPlayerSnapshot),

    /// Event локального игрока.
    Event(NetPlayerEvent),

    /// Сообщение чата.
    ChatMessage { text: String },
}

/// Пакет от сервера к клиенту.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    /// Подключение принято.
    ConnectAccepted { player_id: PlayerId },

    /// Подключение отвергнуто.
    ConnectRejected { reason: String },

    /// Спавн удалённого игрока.
    PlayerSpawn { player_id: PlayerId, name: String },

    /// Удалённый игрок отключился.
    PlayerDespawn { player_id: PlayerId },

    /// Snapshot удалённого игрока.
    Snapshot(NetPlayerSnapshot),

    /// Событие удалённого игрока.
    Event {
        player_id: PlayerId,
        event: NetPlayerEvent,
    },

    /// Чат-сообщение.
    ChatMessage { player_id: PlayerId, text: String },
}
