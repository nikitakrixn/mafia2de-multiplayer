use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use common::logger;
use protocol::{
    ClientPacket, PlayerId, ServerPacket, DEFAULT_PORT, MAX_PLAYERS, PROTOCOL_VERSION,
};

/// Следующий выдаваемый PlayerId.
static NEXT_PLAYER_ID: AtomicU16 = AtomicU16::new(1);

#[derive(Clone)]
struct ClientHandle {
    player_id: PlayerId,
    sender: mpsc::Sender<ServerPacket>,
}

struct SharedServer {
    clients: Mutex<HashMap<PlayerId, ClientHandle>>,
    names: Mutex<HashMap<PlayerId, String>>,
}

impl SharedServer {
    fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            names: Mutex::new(HashMap::new()),
        }
    }

    fn insert_client(&self, player_id: PlayerId, sender: mpsc::Sender<ServerPacket>) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.insert(player_id, ClientHandle { player_id, sender });
        }
    }

    fn remove_client(&self, player_id: PlayerId) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.remove(&player_id);
        }
        if let Ok(mut names) = self.names.lock() {
            names.remove(&player_id);
        }
    }

    fn set_name(&self, player_id: PlayerId, name: String) {
        if let Ok(mut names) = self.names.lock() {
            names.insert(player_id, name);
        }
    }

    fn get_name(&self, player_id: PlayerId) -> Option<String> {
        self.names.lock().ok()?.get(&player_id).cloned()
    }

    fn list_named_players(&self) -> Vec<(PlayerId, String)> {
        self.names
            .lock()
            .map(|m| m.iter().map(|(id, name)| (*id, name.clone())).collect())
            .unwrap_or_default()
    }

    fn send_to(&self, player_id: PlayerId, packet: ServerPacket) {
        let sender = {
            let clients = match self.clients.lock() {
                Ok(c) => c,
                Err(_) => return,
            };
            clients.get(&player_id).map(|c| c.sender.clone())
        };

        if let Some(tx) = sender {
            let _ = tx.send(packet);
        }
    }

    fn broadcast_except(&self, except_player: Option<PlayerId>, packet: ServerPacket) {
        let senders = {
            let clients = match self.clients.lock() {
                Ok(c) => c,
                Err(_) => return,
            };

            clients
                .iter()
                .filter_map(|(id, handle)| {
                    if Some(*id) == except_player {
                        None
                    } else {
                        Some(handle.sender.clone())
                    }
                })
                .collect::<Vec<_>>()
        };

        for tx in senders {
            let _ = tx.send(packet.clone());
        }
    }
}

fn main() {
    if let Err(e) = logger::init(
        logger::Level::Debug,
        logger::Target::Both,
        Some("logs/server.log"),
    ) {
        eprintln!("Logger init failed: {e}");
    }

    logger::info("=============================================================================");
    logger::info("  Mafia II: DE Multiplayer Server");
    logger::info(&format!("  Protocol v{}", PROTOCOL_VERSION));
    logger::info(&format!("  Max players: {}", MAX_PLAYERS));
    logger::info(&format!("  Port: {}", DEFAULT_PORT));
    logger::info("=============================================================================");

    let listener = match TcpListener::bind(("0.0.0.0", DEFAULT_PORT)) {
        Ok(l) => l,
        Err(e) => {
            logger::error(&format!("Bind failed: {e}"));
            return;
        }
    };

    logger::info(&format!("Listening on 0.0.0.0:{DEFAULT_PORT}"));

    let shared = Arc::new(SharedServer::new());

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let shared = Arc::clone(&shared);
                thread::spawn(move || handle_client(stream, shared));
            }
            Err(e) => {
                logger::warn(&format!("Accept failed: {e}"));
            }
        }
    }
}

fn handle_client(stream: TcpStream, shared: Arc<SharedServer>) {
    let peer = match stream.peer_addr() {
        Ok(a) => a.to_string(),
        Err(_) => "<unknown>".to_string(),
    };

    let player_id = NEXT_PLAYER_ID.fetch_add(1, Ordering::Relaxed);

    logger::info(&format!(
        "[server] accepted connection from {peer}, provisional player_id={player_id}"
    ));

    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            logger::error(&format!("[server] try_clone failed: {e}"));
            return;
        }
    };

    let writer_stream = stream;

    let (tx, rx) = mpsc::channel::<ServerPacket>();
    shared.insert_client(player_id, tx.clone());

    // Writer thread
    let writer_handle = thread::spawn(move || {
        writer_thread(writer_stream, rx, player_id);
    });

    // Reader loop
    let result = reader_loop(reader_stream, player_id, &shared, tx.clone());

    // Cleanup
    shared.remove_client(player_id);
    shared.broadcast_except(
        Some(player_id),
        ServerPacket::PlayerDespawn { player_id },
    );

    if let Some(name) = shared.get_name(player_id) {
        logger::info(&format!(
            "[server] player {} ('{}') disconnected",
            player_id, name
        ));
    } else {
        logger::info(&format!("[server] player {} disconnected", player_id));
    }

    drop(tx);
    let _ = writer_handle.join();

    if let Err(e) = result {
        logger::warn(&format!(
            "[server] player {} connection ended with error: {}",
            player_id, e
        ));
    }
}

fn reader_loop(
    stream: TcpStream,
    player_id: PlayerId,
    shared: &Arc<SharedServer>,
    tx: mpsc::Sender<ServerPacket>,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let mut welcomed = false;

    loop {
        line.clear();

        let read = reader
            .read_line(&mut line)
            .map_err(|e| format!("read_line failed: {e}"))?;

        if read == 0 {
            return Ok(());
        }

        while line.ends_with('\n') || line.ends_with('\r') {
            line.pop();
        }

        if line.is_empty() {
            continue;
        }

        let packet = serde_json::from_str::<ClientPacket>(&line)
            .map_err(|e| format!("invalid client packet json: {e}; line={line}"))?;

        match packet {
            ClientPacket::Connect { name, version } => {
                if welcomed {
                    logger::warn(&format!(
                        "[server] player {} sent duplicate Connect",
                        player_id
                    ));
                    continue;
                }

                if version != PROTOCOL_VERSION {
                    let _ = tx.send(ServerPacket::ConnectRejected {
                        reason: format!(
                            "Protocol mismatch: client={} server={}",
                            version, PROTOCOL_VERSION
                        ),
                    });
                    return Ok(());
                }

                shared.set_name(player_id, name.clone());

                // Welcome
                let _ = tx.send(ServerPacket::ConnectAccepted { player_id });

                // Existing players -> newcomer
                for (other_id, other_name) in shared.list_named_players() {
                    if other_id == player_id {
                        continue;
                    }
                    let _ = tx.send(ServerPacket::PlayerSpawn {
                        player_id: other_id,
                        name: other_name,
                    });
                }

                // Newcomer -> others
                shared.broadcast_except(
                    Some(player_id),
                    ServerPacket::PlayerSpawn {
                        player_id,
                        name: name.clone(),
                    },
                );

                welcomed = true;

                logger::info(&format!(
                    "[server] player {} authenticated as '{}'",
                    player_id, name
                ));
            }

            ClientPacket::Disconnect => {
                return Ok(());
            }

            ClientPacket::Snapshot(mut snapshot) => {
                if !welcomed {
                    continue;
                }

                // Никогда не доверяем player_id клиента.
                snapshot.player_id = player_id;

                shared.broadcast_except(Some(player_id), ServerPacket::Snapshot(snapshot));
            }

            ClientPacket::Event(event) => {
                if !welcomed {
                    continue;
                }

                shared.broadcast_except(
                    Some(player_id),
                    ServerPacket::Event { player_id, event },
                );
            }

            ClientPacket::ChatMessage { text } => {
                if !welcomed {
                    continue;
                }

                shared.broadcast_except(
                    None,
                    ServerPacket::ChatMessage { player_id, text },
                );
            }
        }
    }
}

fn writer_thread(mut stream: TcpStream, rx: mpsc::Receiver<ServerPacket>, player_id: PlayerId) {
    for packet in rx {
        let json = match serde_json::to_string(&packet) {
            Ok(s) => s,
            Err(e) => {
                logger::warn(&format!(
                    "[server] failed to serialize packet for player {}: {}",
                    player_id, e
                ));
                continue;
            }
        };

        if let Err(e) = stream.write_all(json.as_bytes()) {
            logger::warn(&format!(
                "[server] write failed for player {}: {}",
                player_id, e
            ));
            break;
        }
        if let Err(e) = stream.write_all(b"\n") {
            logger::warn(&format!(
                "[server] write newline failed for player {}: {}",
                player_id, e
            ));
            break;
        }
    }

    let _ = stream.shutdown(Shutdown::Both);
}
