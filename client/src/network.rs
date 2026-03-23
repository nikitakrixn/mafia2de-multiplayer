//! Network layer v0.
//!
//! ВАЖНО:
//! Это не полноценный TCP/UDP transport.
//!
//! Это multiplayer-ready сетевой слой клиента:
//! - хранит connection state
//! - держит outbound/inbound очереди
//! - принимает local snapshots/events
//! - умеет применять incoming remote snapshots/events
//!
//! Следующим этапом сюда будет вставлен реальный transport thread.
//! Но архитектура уже будет правильной и не потребует переделки gameplay слоя.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use common::logger;
use protocol::{ClientPacket, NetPlayerEvent, NetPlayerSnapshot, PlayerId, ServerPacket};

#[derive(Debug)]
struct NetworkState {
    connected: bool,
    local_player_id: Option<PlayerId>,
    nickname: String,
    server_addr: String,

    /// Исходящие пакеты локального игрока.
    outbound: VecDeque<ClientPacket>,

    /// Входящие пакеты от сервера / тестовой инъекции.
    inbound: VecDeque<ServerPacket>,
}

static NETWORK: OnceLock<Mutex<NetworkState>> = OnceLock::new();

fn state() -> &'static Mutex<NetworkState> {
    NETWORK.get_or_init(|| {
        Mutex::new(NetworkState {
            connected: false,
            local_player_id: None,
            nickname: String::new(),
            server_addr: String::new(),
            outbound: VecDeque::new(),
            inbound: VecDeque::new(),
        })
    })
}

/// Инициализация network subsystem.
pub fn init() {
    let _ = state();
}

/// Подключены ли мы к серверу.
///
/// В v0 это состояние multiplayer session, а не полноценного сокета.
pub fn is_connected() -> bool {
    state().lock().map(|s| s.connected).unwrap_or(false)
}

/// Локальный player_id в сессии.
pub fn local_player_id() -> Option<PlayerId> {
    state().lock().ok().and_then(|s| s.local_player_id)
}

/// Подключение к серверу.
///
/// ВАЖНО:
/// v0 — это пока multiplayer session scaffold.
/// Реального transport thread здесь ещё нет.
/// Но все верхние слои клиента уже начинают жить в правильной архитектуре.
pub fn connect(ip: &str, port: u16, nickname: &str) -> bool {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in connect");
            return false;
        }
    };

    if guard.connected {
        logger::warn("[network] уже подключены");
        return false;
    }

    guard.connected = true;
    guard.local_player_id = Some(1);
    guard.nickname = nickname.to_string();
    guard.server_addr = format!("{ip}:{port}");
    guard.outbound.clear();
    guard.inbound.clear();

    drop(guard);

    crate::overlay::multiplayer_ui::clear_players();
    crate::overlay::multiplayer_ui::set_connection_status(true, format!("Сессия v0: {ip}:{port}"));
    crate::overlay::multiplayer_ui::add_player(1, nickname.to_string(), 0, true);
    crate::overlay::multiplayer_ui::add_system_message(
        "Подключение v0 активно. Transport thread будет добавлен следующим этапом.".to_string(),
    );

    logger::info(&format!(
        "[network] session started: {}:{} as {}",
        ip, port, nickname
    ));

    true
}

/// Отключение от сессии.
pub fn disconnect() -> bool {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in disconnect");
            return false;
        }
    };

    if !guard.connected {
        return false;
    }

    guard.connected = false;
    guard.local_player_id = None;
    guard.nickname.clear();
    guard.server_addr.clear();
    guard.outbound.clear();
    guard.inbound.clear();

    drop(guard);

    crate::remote_players::clear_all();
    crate::overlay::multiplayer_ui::clear_players();
    crate::overlay::multiplayer_ui::set_connection_status(false, "Отключен".to_string());
    crate::overlay::multiplayer_ui::add_system_message("Сессия завершена".to_string());

    logger::info("[network] session stopped");
    true
}

/// Очередь outbound snapshot.
pub fn push_local_snapshot(snapshot: NetPlayerSnapshot) {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in push_local_snapshot");
            return;
        }
    };

    if !guard.connected {
        return;
    }

    guard.outbound.push_back(ClientPacket::Snapshot(snapshot));
}

/// Очередь outbound event.
pub fn push_local_event(event: NetPlayerEvent) {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in push_local_event");
            return;
        }
    };

    if !guard.connected {
        return;
    }

    guard.outbound.push_back(ClientPacket::Event(event));
}

/// Отправка сообщения в чат.
///
/// В v0:
/// - кладёт packet в outbound queue
/// - локально эхо-показывает сообщение в UI
pub fn send_chat_message(text: String) {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in send_chat_message");
            return;
        }
    };

    if !guard.connected {
        crate::overlay::multiplayer_ui::add_system_message(
            "Нельзя отправить сообщение: нет подключения".to_string(),
        );
        return;
    }

    let nickname = guard.nickname.clone();
    guard
        .outbound
        .push_back(ClientPacket::ChatMessage { text: text.clone() });
    drop(guard);

    // Временное локальное эхо. Реальный сервер потом разрулит broadcast.
    crate::overlay::multiplayer_ui::add_chat_message(nickname, text);
}

/// Инъекция удалённого игрока для тестов / будущего transport thread.
pub fn inject_player_spawn(player_id: PlayerId, name: String) {
    if let Ok(mut s) = state().lock() {
        s.inbound
            .push_back(ServerPacket::PlayerSpawn { player_id, name });
    }
}

/// Инъекция remote snapshot.
pub fn inject_remote_snapshot(snapshot: NetPlayerSnapshot) {
    if let Ok(mut s) = state().lock() {
        s.inbound.push_back(ServerPacket::Snapshot(snapshot));
    }
}

/// Инъекция remote event.
pub fn inject_remote_event(player_id: PlayerId, event: NetPlayerEvent) {
    if let Ok(mut s) = state().lock() {
        s.inbound
            .push_back(ServerPacket::Event { player_id, event });
    }
}

/// Инъекция despawn.
pub fn inject_player_despawn(player_id: PlayerId) {
    if let Ok(mut s) = state().lock() {
        s.inbound
            .push_back(ServerPacket::PlayerDespawn { player_id });
    }
}

/// Вызывается на game thread.
///
/// В v0:
/// - дренит outbound queue (пока просто лог/выкидывание)
/// - применяет inbound queue к remote player subsystem
pub fn poll_main_thread() {
    let (connected, local_id, outbound, inbound) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => {
                logger::error("[network] mutex poisoned in poll_main_thread");
                return;
            }
        };

        let connected = guard.connected;
        let local_id = guard.local_player_id;
        let outbound: Vec<_> = guard.outbound.drain(..).collect();
        let inbound: Vec<_> = guard.inbound.drain(..).collect();

        (connected, local_id, outbound, inbound)
    };

    if !connected {
        return;
    }

    // -------------------------------------------------------------------------
    // OUTBOUND
    // -------------------------------------------------------------------------
    //
    // Пока transport не реализован, просто дреним очередь.
    // Это важно, чтобы верхние слои уже работали как multiplayer-ready,
    // но очередь не росла бесконечно.
    if !outbound.is_empty() {
        logger::debug(&format!(
            "[network] drained {} outbound packet(s) (transport v0 stub)",
            outbound.len()
        ));
    }

    // -------------------------------------------------------------------------
    // INBOUND
    // -------------------------------------------------------------------------
    for packet in inbound {
        match packet {
            ServerPacket::PlayerSpawn { player_id, name } => {
                if Some(player_id) == local_id {
                    continue;
                }

                crate::overlay::multiplayer_ui::add_player(
                    player_id as u32,
                    name.clone(),
                    42,
                    false,
                );
                crate::overlay::multiplayer_ui::add_system_message(format!(
                    "{} присоединился к игре",
                    name
                ));
                crate::remote_players::ensure_binding(player_id, &name);
            }

            ServerPacket::PlayerDespawn { player_id } => {
                crate::remote_players::remove_binding(player_id);
                crate::overlay::multiplayer_ui::remove_player(player_id as u32);
            }

            ServerPacket::Snapshot(snapshot) => {
                if Some(snapshot.player_id) == local_id {
                    continue;
                }
                crate::remote_players::apply_snapshot(snapshot);
            }

            ServerPacket::Event { player_id, event } => {
                if Some(player_id) == local_id {
                    continue;
                }
                crate::remote_players::apply_event(player_id, event);
            }

            ServerPacket::ChatMessage { player_id, text } => {
                let author = format!("Player#{player_id}");
                crate::overlay::multiplayer_ui::add_chat_message(author, text);
            }

            ServerPacket::ConnectAccepted { player_id } => {
                logger::info(&format!(
                    "[network] connect accepted: player_id={player_id}"
                ));
            }

            ServerPacket::ConnectRejected { reason } => {
                logger::warn(&format!("[network] connect rejected: {reason}"));
            }

            ServerPacket::PlayerUpdate { .. } => {
                // legacy path — игнорируем в v0
            }
        }
    }
}
