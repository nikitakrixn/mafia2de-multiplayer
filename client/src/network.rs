//! Network layer v1.
//!
//! Реальный transport:
//! - TCP
//! - line-delimited JSON
//! - один transport thread
//!
//! Архитектура:
//! - game thread складывает outbound packets в очередь
//! - transport thread забирает их и пишет в сокет
//! - transport thread читает входящие пакеты и кладёт их в inbound очередь
//! - game thread в `poll_main_thread()` применяет inbound packets
//!
//! ВАЖНО:
//! - gameplay/NPC application происходит ТОЛЬКО на game thread
//! - transport thread не лезет в движок напрямую

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use common::logger;
use protocol::{ClientPacket, NetPlayerEvent, NetPlayerSnapshot, PlayerId, ServerPacket};

/// Транспорт считается активным, пока transport thread крутится.
static TRANSPORT_RUNNING: AtomicBool = AtomicBool::new(false);

/// Флаг запроса на остановку transport thread.
static TRANSPORT_STOP: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
struct NetworkState {
    connected: bool,
    local_player_id: Option<PlayerId>,
    nickname: String,
    server_addr: String,

    /// Очередь исходящих пакетов.
    outbound: VecDeque<ClientPacket>,

    /// Очередь входящих пакетов.
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
/// 1. Открывает TCP
/// 2. Запускает transport thread
/// 3. Кладёт `Connect` packet в outbound queue
pub fn connect(ip: &str, port: u16, nickname: &str) -> bool {
    let addr = format!("{ip}:{port}");

    {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => {
                logger::error("[network] mutex poisoned in connect");
                return false;
            }
        };

        if guard.connected || TRANSPORT_RUNNING.load(Ordering::Acquire) {
            logger::warn("[network] уже подключены / transport уже работает");
            return false;
        }

        guard.connected = true;
        guard.local_player_id = None;
        guard.nickname = nickname.to_string();
        guard.server_addr = addr.clone();
        guard.outbound.clear();
        guard.inbound.clear();
    }

    crate::overlay::multiplayer_ui::set_connection_status(
        false,
        format!("Подключение к {addr}..."),
    );

    let stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            logger::error(&format!("[network] connect({addr}) failed: {e}"));

            if let Ok(mut guard) = state().lock() {
                guard.connected = false;
                guard.local_player_id = None;
            }

            crate::overlay::multiplayer_ui::set_connection_status(
                false,
                format!("Ошибка подключения: {e}"),
            );
            return false;
        }
    };

    if let Err(e) = stream.set_nodelay(true) {
        logger::warn(&format!("[network] set_nodelay failed: {e}"));
    }

    if let Err(e) = stream.set_nonblocking(true) {
        logger::warn(&format!("[network] set_nonblocking failed: {e}"));
    }

    TRANSPORT_STOP.store(false, Ordering::Release);
    TRANSPORT_RUNNING.store(true, Ordering::Release);

    {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => {
                logger::error("[network] mutex poisoned before transport start");
                return false;
            }
        };

        guard.outbound.push_back(ClientPacket::Connect {
            name: nickname.to_string(),
            version: protocol::PROTOCOL_VERSION,
        });
    }

    thread::spawn(move || {
        transport_thread_main(stream);
    });

    logger::info(&format!(
        "[network] TCP connected to {addr}, transport thread started"
    ));

    true
}

/// Отключение от сервера.
pub fn disconnect() -> bool {
    {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => {
                logger::error("[network] mutex poisoned in disconnect");
                return false;
            }
        };

        if !guard.connected && !TRANSPORT_RUNNING.load(Ordering::Acquire) {
            return false;
        }

        guard.outbound.push_back(ClientPacket::Disconnect);
        guard.connected = false;
        guard.local_player_id = None;
    }

    TRANSPORT_STOP.store(true, Ordering::Release);

    crate::remote_players::clear_all();
    crate::overlay::multiplayer_ui::clear_players();
    crate::overlay::multiplayer_ui::set_connection_status(false, "Отключен".to_string());
    crate::overlay::multiplayer_ui::add_system_message("Отключено от сервера".to_string());

    logger::info("[network] session stopped");
    true
}

/// Положить snapshot локального игрока в outbound queue.
pub fn push_local_snapshot(snapshot: NetPlayerSnapshot) {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in push_local_snapshot");
            return;
        }
    };

    if !guard.connected || guard.local_player_id.is_none() {
        return;
    }

    guard.outbound.push_back(ClientPacket::Snapshot(snapshot));
}

/// Положить event локального игрока в outbound queue.
pub fn push_local_event(event: NetPlayerEvent) {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            logger::error("[network] mutex poisoned in push_local_event");
            return;
        }
    };

    if !guard.connected || guard.local_player_id.is_none() {
        return;
    }

    guard.outbound.push_back(ClientPacket::Event(event));
}

/// Отправить сообщение чата.
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

    guard
        .outbound
        .push_back(ClientPacket::ChatMessage { text: text.clone() });
}

pub fn inject_player_spawn(player_id: PlayerId, name: String) {
    if let Ok(mut s) = state().lock() {
        s.inbound.push_back(ServerPacket::PlayerSpawn { player_id, name });
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
        s.inbound.push_back(ServerPacket::Event { player_id, event });
    }
}

/// Инъекция despawn.
pub fn inject_player_despawn(player_id: PlayerId) {
    if let Ok(mut s) = state().lock() {
        s.inbound.push_back(ServerPacket::PlayerDespawn { player_id });
    }
}

/// Вызывается на game thread — применяет inbound packets к runtime.
pub fn poll_main_thread() {
    let inbound = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => {
                logger::error("[network] mutex poisoned in poll_main_thread");
                return;
            }
        };

        guard.inbound.drain(..).collect::<Vec<_>>()
    };

    if inbound.is_empty() {
        return;
    }

    for packet in inbound {
        handle_incoming_packet(packet);
    }
}

fn handle_incoming_packet(packet: ServerPacket) {
    match packet {
        ServerPacket::ConnectAccepted { player_id } => {
            let nickname = {
                let mut guard = match state().lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                guard.local_player_id = Some(player_id);
                guard.nickname.clone()
            };

            crate::overlay::multiplayer_ui::clear_players();
            crate::overlay::multiplayer_ui::set_connection_status(
                true,
                format!("Подключен как player #{player_id}"),
            );
            crate::overlay::multiplayer_ui::add_player(
                player_id as u32,
                nickname.clone(),
                0,
                true,
            );
            crate::overlay::multiplayer_ui::add_system_message(format!(
                "Подключение принято. Ваш ID: {player_id}"
            ));

            logger::info(&format!(
                "[network] connect accepted: player_id={player_id}, nickname={nickname}"
            ));
        }

        ServerPacket::ConnectRejected { reason } => {
            logger::warn(&format!("[network] connect rejected: {reason}"));

            if let Ok(mut guard) = state().lock() {
                guard.connected = false;
                guard.local_player_id = None;
            }

            crate::overlay::multiplayer_ui::set_connection_status(
                false,
                format!("Отказ: {reason}"),
            );
        }

        ServerPacket::PlayerSpawn { player_id, name } => {
            if Some(player_id) == local_player_id() {
                return;
            }

            crate::overlay::multiplayer_ui::add_player(player_id as u32, name.clone(), 42, false);
            crate::overlay::multiplayer_ui::add_system_message(format!(
                "{name} присоединился к игре"
            ));
            crate::remote_players::ensure_binding(player_id, &name);
        }

        ServerPacket::PlayerDespawn { player_id } => {
            crate::remote_players::remove_binding(player_id);
            crate::overlay::multiplayer_ui::remove_player(player_id as u32);
        }

        ServerPacket::Snapshot(snapshot) => {
            if Some(snapshot.player_id) == local_player_id() {
                return;
            }
            crate::remote_players::apply_snapshot(snapshot);
        }

        ServerPacket::Event { player_id, event } => {
            if Some(player_id) == local_player_id() {
                return;
            }
            crate::remote_players::apply_event(player_id, event);
        }

        ServerPacket::ChatMessage { player_id, text } => {
            let author = format!("Player#{player_id}");
            crate::overlay::multiplayer_ui::add_chat_message(author, text);
        }
    }
}

/// Реальный transport thread.
fn transport_thread_main(mut stream: TcpStream) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());

    logger::info(&format!("[network] transport thread started for {peer}"));

    let mut read_accum = String::new();
    let mut read_buf = [0u8; 4096];

    loop {
        if TRANSPORT_STOP.load(Ordering::Acquire) {
            let _ = stream.shutdown(Shutdown::Both);
            break;
        }

        let outbound_packets = {
            let mut guard = match state().lock() {
                Ok(g) => g,
                Err(_) => {
                    logger::error("[network] mutex poisoned in transport outbound");
                    break;
                }
            };

            guard.outbound.drain(..).collect::<Vec<_>>()
        };

        for packet in outbound_packets {
            crate::net_debug::on_outbound(&packet);

            if let Err(e) = write_packet_line(&mut stream, &packet) {
                logger::error(&format!("[network] write packet failed: {e}"));
                transport_fail_disconnect("Ошибка записи в сокет");
                return;
            }
        }

        match stream.read(&mut read_buf) {
            Ok(0) => {
                logger::warn("[network] server closed connection");
                transport_fail_disconnect("Сервер закрыл соединение");
                return;
            }

            Ok(n) => {
                let chunk = String::from_utf8_lossy(&read_buf[..n]);
                read_accum.push_str(&chunk);

                while let Some(pos) = read_accum.find('\n') {
                    let mut line = read_accum.drain(..=pos).collect::<String>();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }

                    if line.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<ServerPacket>(&line) {
                        Ok(packet) => {
                            crate::net_debug::on_inbound(&packet);

                            if let Ok(mut guard) = state().lock() {
                                guard.inbound.push_back(packet);
                            }
                        }
                        Err(e) => {
                            logger::warn(&format!(
                                "[network] failed to parse server packet: {e}; line={line}"
                            ));
                        }
                    }
                }
            }

            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Ничего не пришло — нормально.
            }

            Err(e) => {
                logger::error(&format!("[network] read failed: {e}"));
                transport_fail_disconnect("Ошибка чтения из сокета");
                return;
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    TRANSPORT_RUNNING.store(false, Ordering::Release);
    logger::info("[network] transport thread stopped");
}

/// Сериализует пакет в JSON line (`...\n`).
fn write_packet_line(stream: &mut TcpStream, packet: &ClientPacket) -> std::io::Result<()> {
    let json = serde_json::to_string(packet)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

/// Переводит network subsystem в disconnected state после ошибки transport thread.
fn transport_fail_disconnect(reason: &str) {
    logger::warn(&format!("[network] transport disconnected: {reason}"));

    if let Ok(mut guard) = state().lock() {
        guard.connected = false;
        guard.local_player_id = None;
        guard.outbound.clear();
    }

    TRANSPORT_RUNNING.store(false, Ordering::Release);
    TRANSPORT_STOP.store(true, Ordering::Release);

    crate::remote_players::clear_all();
    crate::overlay::multiplayer_ui::clear_players();
    crate::overlay::multiplayer_ui::set_connection_status(false, reason.to_string());
    crate::overlay::multiplayer_ui::add_system_message(reason.to_string());
}

/// Автоматически оборвать session, если игра ушла в меню/выгрузку.
pub fn auto_disconnect_if_session_invalid() {
    use crate::state::GameSessionState;

    match crate::state::get() {
        GameSessionState::Boot
        | GameSessionState::FrontendMenu
        | GameSessionState::ShuttingDown => {
            if is_connected() || TRANSPORT_RUNNING.load(Ordering::Acquire) {
                logger::info("[network] auto-disconnect: session no longer in game");
                let _ = disconnect();
            }
        }
        GameSessionState::Loading | GameSessionState::Paused | GameSessionState::InGame => {}
    }
}
