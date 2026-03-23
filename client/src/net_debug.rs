//! Телеметрия transport слоя.
//!
//! Цель:
//! - не спамить лог каждым snapshot
//! - иметь понятную статистику: сколько packet'ов отправлено/получено
//! - логировать важные события (connect, disconnect, events, chat)
//!
//! Вызывается из network.rs.

use std::sync::atomic::{AtomicU64, Ordering};

use common::logger;
use protocol::{ClientPacket, ServerPacket};

static OUT_SNAPSHOT_COUNT: AtomicU64 = AtomicU64::new(0);
static OUT_EVENT_COUNT: AtomicU64 = AtomicU64::new(0);
static OUT_CHAT_COUNT: AtomicU64 = AtomicU64::new(0);

static IN_SNAPSHOT_COUNT: AtomicU64 = AtomicU64::new(0);
static IN_EVENT_COUNT: AtomicU64 = AtomicU64::new(0);
static IN_CHAT_COUNT: AtomicU64 = AtomicU64::new(0);

/// Логировать отправку пакета с throttling.
pub fn on_outbound(packet: &ClientPacket) {
    match packet {
        ClientPacket::Connect { name, version } => {
            logger::info(&format!(
                "[net/out] Connect name='{}' version={}",
                name, version
            ));
        }
        ClientPacket::Disconnect => {
            logger::info("[net/out] Disconnect");
        }
        ClientPacket::Snapshot(_) => {
            let n = OUT_SNAPSHOT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            // Каждый 20-й snapshot
            if n % 20 == 0 {
                logger::debug(&format!("[net/out] Snapshot count={}", n));
            }
        }
        ClientPacket::Event(ev) => {
            let n = OUT_EVENT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            logger::debug(&format!("[net/out] Event #{n}: {:?}", ev));
        }
        ClientPacket::ChatMessage { text } => {
            let n = OUT_CHAT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            logger::info(&format!("[net/out] Chat #{n}: {}", text));
        }
    }
}

/// Логировать получение пакета с throttling.
pub fn on_inbound(packet: &ServerPacket) {
    match packet {
        ServerPacket::ConnectAccepted { player_id } => {
            logger::info(&format!("[net/in] ConnectAccepted id={}", player_id));
        }
        ServerPacket::ConnectRejected { reason } => {
            logger::warn(&format!("[net/in] ConnectRejected: {}", reason));
        }
        ServerPacket::PlayerSpawn { player_id, name } => {
            logger::info(&format!(
                "[net/in] PlayerSpawn id={} name='{}'",
                player_id, name
            ));
        }
        ServerPacket::PlayerDespawn { player_id } => {
            logger::info(&format!("[net/in] PlayerDespawn id={}", player_id));
        }
        ServerPacket::Snapshot(snapshot) => {
            let n = IN_SNAPSHOT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            if n % 20 == 0 {
                logger::debug(&format!(
                    "[net/in] Snapshot count={} last_from={}",
                    n, snapshot.player_id
                ));
            }
        }
        ServerPacket::Event { player_id, event } => {
            let n = IN_EVENT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            logger::debug(&format!(
                "[net/in] Event #{n} from {}: {:?}",
                player_id, event
            ));
        }
        ServerPacket::ChatMessage { player_id, text } => {
            let n = IN_CHAT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            logger::info(&format!(
                "[net/in] Chat #{n} from {}: {}",
                player_id, text
            ));
        }
    }
}
