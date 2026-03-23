//! Remote player bindings.
//!
//! MVP-идея:
//! - удалённые игроки не создают новых human entities
//! - вместо этого мы привязываем их к существующим NPC (Joe, Henry)
//!
//! Это надёжнее и дешевле, чем сразу лезть в native spawning.
//!
//! Ограничения v0:
//! - максимум 2 удалённых игрока (Joe, Henry)
//! - только position / forward / health / death
//! - vehicle enter/leave пока только логируются

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use common::logger;
use protocol::{NetPlayerEvent, NetPlayerSnapshot, PlayerId};
use sdk::game::npc::Npc;
use sdk::game::player::Vec3;

#[derive(Debug)]
struct RemoteBinding {
    player_id: PlayerId,
    player_name: String,
    npc_name: String,
    npc: Npc,
}

static BINDINGS: OnceLock<Mutex<HashMap<PlayerId, RemoteBinding>>> = OnceLock::new();

fn bindings() -> &'static Mutex<HashMap<PlayerId, RemoteBinding>> {
    BINDINGS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Доступные NPC для MVP proxy.
const NPC_POOL: &[&str] = &["Joe", "Henry"];

pub fn init() {
    let _ = bindings();
}

/// Очистить все привязки.
pub fn clear_all() {
    if let Ok(mut map) = bindings().lock() {
        map.clear();
    }
}

/// Удалить привязку одного удалённого игрока.
pub fn remove_binding(player_id: PlayerId) {
    if let Ok(mut map) = bindings().lock() {
        if let Some(binding) = map.remove(&player_id) {
            logger::info(&format!(
                "[remote] unbound player {} from NPC '{}'",
                player_id, binding.npc_name
            ));
        }
    }
}

/// Убедиться что у удалённого игрока есть NPC-binding.
pub fn ensure_binding(player_id: PlayerId, player_name: &str) {
    let mut map = match bindings().lock() {
        Ok(m) => m,
        Err(_) => {
            logger::error("[remote] mutex poisoned in ensure_binding");
            return;
        }
    };

    if map.contains_key(&player_id) {
        return;
    }

    // Ищем свободного NPC из пула
    let used_names: Vec<String> = map.values().map(|b| b.npc_name.clone()).collect();

    for &npc_name in NPC_POOL {
        if used_names.iter().any(|s| s == npc_name) {
            continue;
        }

        let Some(npc) = Npc::find(npc_name) else {
            logger::warn(&format!(
                "[remote] NPC '{}' не найден для player_id={}",
                npc_name, player_id
            ));
            continue;
        };

        // Proxy-NPC не должен умирать от локального хаоса.
        npc.set_invulnerable(true);
        npc.set_demigod(true);

        logger::info(&format!(
            "[remote] bound player {} ('{}') -> NPC '{}'",
            player_id, player_name, npc_name
        ));

        map.insert(
            player_id,
            RemoteBinding {
                player_id,
                player_name: player_name.to_string(),
                npc_name: npc_name.to_string(),
                npc,
            },
        );
        return;
    }

    logger::warn(&format!(
        "[remote] no free NPC binding slot for player_id={}",
        player_id
    ));
}

/// Применить удалённый snapshot к связанному NPC.
///
/// Вызывается на game thread.
pub fn apply_snapshot(snapshot: NetPlayerSnapshot) {
    let mut map = match bindings().lock() {
        Ok(m) => m,
        Err(_) => {
            logger::error("[remote] mutex poisoned in apply_snapshot");
            return;
        }
    };

    let Some(binding) = map.get_mut(&snapshot.player_id) else {
        // Если биндинга ещё нет, создадим placeholder name
        drop(map);
        ensure_binding(
            snapshot.player_id,
            &format!("Player#{}", snapshot.player_id),
        );

        map = match bindings().lock() {
            Ok(m) => m,
            Err(_) => return,
        };

        let Some(binding) = map.get_mut(&snapshot.player_id) else {
            return;
        };

        apply_snapshot_to_binding(binding, &snapshot);
        return;
    };

    apply_snapshot_to_binding(binding, &snapshot);
}

fn apply_snapshot_to_binding(binding: &mut RemoteBinding, snapshot: &NetPlayerSnapshot) {
    let pos = Vec3 {
        x: snapshot.position.x,
        y: snapshot.position.y,
        z: snapshot.position.z,
    };

    let dir = Vec3 {
        x: snapshot.forward.x,
        y: snapshot.forward.y,
        z: snapshot.forward.z,
    };

    binding.npc.set_position(&pos);

    let dir_len_sq = dir.x * dir.x + dir.y * dir.y + dir.z * dir.z;
    if dir_len_sq > 0.0001 {
        let _ = binding.npc.set_forward(&dir);
    }

    if snapshot.is_dead {
        binding.npc.set_health(0.0);
        logger::debug(&format!(
            "[remote] {} (NPC '{}') — dead",
            binding.player_name, binding.npc_name
        ));
    } else {
        binding.npc.set_health(snapshot.health.max(1.0));
    }

    // Обновляем запись в UI (пинг пока 0 — нет RTT)
    crate::overlay::multiplayer_ui::update_player_ping(binding.player_id as u32, 0);
}

/// Получить имя удалённого игрока по его ID.
#[allow(dead_code)]
pub fn get_player_name(player_id: PlayerId) -> Option<String> {
    bindings()
        .lock()
        .ok()?
        .get(&player_id)
        .map(|b| b.player_name.clone())
}

/// Применить удалённое высокоуровневое событие.
///
/// Пока v0 делает только безопасные вещи:
/// - Death → hp=0
/// - остальное логируется
pub fn apply_event(player_id: PlayerId, event: NetPlayerEvent) {
    let map = match bindings().lock() {
        Ok(m) => m,
        Err(_) => {
            logger::error("[remote] mutex poisoned in apply_event");
            return;
        }
    };

    let Some(binding) = map.get(&player_id) else {
        return;
    };

    match event {
        NetPlayerEvent::Death => {
            binding.npc.set_health(0.0);
            logger::info(&format!(
                "[remote] {} (NPC '{}') died",
                binding.player_name, binding.npc_name
            ));
        }

        NetPlayerEvent::Shot => {
            logger::debug(&format!(
                "[remote] {} (NPC '{}') shot",
                binding.player_name, binding.npc_name
            ));
        }

        NetPlayerEvent::EnterVehicle
        | NetPlayerEvent::EnterVehicleDone
        | NetPlayerEvent::LeaveVehicle
        | NetPlayerEvent::LeaveVehicleDone
        | NetPlayerEvent::Damage
        | NetPlayerEvent::WeaponSelect
        | NetPlayerEvent::WeaponHide
        | NetPlayerEvent::Fx(_) => {
            logger::debug(&format!(
                "[remote] {} (NPC '{}'): {:?}",
                binding.player_name, binding.npc_name, event
            ));
        }
    }
}
