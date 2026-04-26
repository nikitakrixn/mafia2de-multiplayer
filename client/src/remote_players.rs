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
//!
//! **Locomotion:** в движении — `MoveDir` + `apply_from_delta(prev, target, dt_net)` на
//! каждом snapshot, где **`dt_net`** — фактический интервал между приходами snapshot
//! (clamp 45..550 мс), а не константа 150 мс. Между snapshot позицию ведёт движок.
//! В стоянии — только `SetPos` + lerp на тике. Сброс `CleanMoveCommands` — при стопе,
//! телепорте, первом snapshot и входе в движение (не на каждом шаге).
//!
//! Реверс DE / vtable сообщений: `sdk::game::remote_locomotion_de`, env
//! `M2MP_LOG_ENTITY_MSG_VTABLES=1` в `human_messages`.
//! TODO: см. sdk::game::remote_locomotion_de

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use common::logger;
use protocol::{NetPlayerEvent, NetPlayerSnapshot, PlayerId};
use sdk::game::aim_look::HumanAim;
use sdk::game::npc::Npc;
use sdk::game::npc_motion::MoveDirCommand;
use sdk::game::spawn;
use sdk::game::Player;
use sdk::structures::CHumanAIController;
use sdk::types::Vec3;

/// Если рассинхрон позиций > этого значения — форсим SetPos (телепорт).
const TELEPORT_THRESHOLD_M: f32 = 8.0;

/// Если drift между фактической позицией NPC и целью > этого значения,
/// делаем мягкую коррекцию через SetPos (без телепорта).
///
/// Меньше точнее sync, но больше анимационных артефактов.
/// Больше плавнее анимация, но NPC может отставать.
const DRIFT_CORRECTION_M: f32 = 1.5;

/// Сколько секунд между snapshot'ами (приблизительно — зависит
/// от `TRACK_INTERVAL_MS` в player_tracker)
const SNAPSHOT_DT_SECS: f32 = 0.15;

/// Сброс MoveDir handle + нативной очереди move-команд.
fn hard_reset_remote_locomotion(binding: &mut RemoteBinding) {
    if let Some(cmd) = binding.move_cmd {
        unsafe { cmd.stop() };
    }
    binding.move_cmd = None;
    unsafe { binding.npc.clean_move_command_queue() };
}

/// Оценка dt игрового тика для `apply_from_delta` (нет стабильного fixed-tick API).
const EST_GAME_DT_SECS: f32 = 1.0 / 60.0;

static INTERP_LAST_CLOCK: Mutex<Option<Instant>> = Mutex::new(None);

/// Стандартный пистолет (Colt M1911A1) для авто-выдачи remote NPC.
/// Без оружия SetupAimDir не виден визуально (нет ствола = нет подъёма).
/// Нужно добавлять CHuman_WeaponChangeAnim
const DEFAULT_REMOTE_WEAPON_ID: u32 = 4;
const DEFAULT_REMOTE_WEAPON_AMMO: u32 = 100;

#[derive(Debug)]
struct RemoteBinding {
    player_id: PlayerId,
    player_name: String,
    npc_name: String,
    npc: Npc,
    move_cmd: Option<MoveDirCommand>,
    /// Предыдущая позиция из **сетевого** snapshot (для телепорта / длины шага).
    last_net_pos: Option<Vec3>,
    /// Интерполяция: начало сегмента (прошлая сетевая точка).
    interp_from: Vec3,
    /// Интерполяция: конец сегмента (последняя сетевая точка).
    interp_to: Vec3,
    /// Когда пришёл текущий `interp_to`.
    interp_begin: Instant,
    /// Время предыдущего сетевого snapshot (для `dt` в `apply_from_delta`).
    last_net_sample_at: Option<Instant>,
    /// Копия последнего snapshot: идёт ли удалённый игрок.
    remote_is_moving: bool,
    /// Копия `movement_mode` из последнего snapshot.
    remote_movement_mode: u8,
    /// Включён ли сейчас SetupAimDir на этом NPC
    /// (используем чтобы не дёргать движок на каждом snapshot,
    /// а только при смене состояния).
    aim_active: bool,
}

static BINDINGS: OnceLock<Mutex<HashMap<PlayerId, RemoteBinding>>> = OnceLock::new();

fn bindings() -> &'static Mutex<HashMap<PlayerId, RemoteBinding>> {
    BINDINGS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Смещение начального спавна proxy-NPC относительно локального игрока.
/// 3 метра в сторону, чтобы не появиться внутри player'а.
const SPAWN_OFFSET: Vec3 = Vec3 { x: 3.0, y: 0.0, z: 0.0 };

pub fn init() {
    let _ = bindings();
}

/// Плавное сближение с сетевой позицией на каждом игровом тике
///
/// Вызывается из `main_thread::on_main_thread_tick` (game thread).
pub fn tick_interpolation() {
    let dt = measured_game_dt_secs();
    let mut map = match bindings().lock() {
        Ok(m) => m,
        Err(_) => return,
    };
    for b in map.values_mut() {
        b.tick_interpolate(dt);
    }
}

fn measured_game_dt_secs() -> f32 {
    let now = Instant::now();
    let mut slot = match INTERP_LAST_CLOCK.lock() {
        Ok(s) => s,
        Err(_) => return EST_GAME_DT_SECS,
    };
    let dt = slot
        .replace(now)
        .map(|prev| (now - prev).as_secs_f32())
        .unwrap_or(EST_GAME_DT_SECS)
        .clamp(0.001, 0.08);
    dt
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
        if let Some(mut binding) = map.remove(&player_id) {
            hard_reset_remote_locomotion(&mut binding);
            if binding.aim_active {
                unsafe {
                    HumanAim::new(binding.npc.ptr()).clear();
                }
            }
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

    // Спавним свежий CHumanNPC рядом с локальным игроком.
    let spawn_pos = match Player::get_active().and_then(|p| p.get_position()) {
        Some(p) => Vec3 {
            x: p.x + SPAWN_OFFSET.x,
            y: p.y + SPAWN_OFFSET.y,
            z: p.z + SPAWN_OFFSET.z,
        },
        None => {
            logger::warn(&format!(
                "[remote] player not ready yet, defer binding for player_id={}",
                player_id
            ));
            return;
        }
    };

    let entity_ptr = match unsafe { spawn::spawn_human_npc(spawn_pos) } {
        Some(p) => p,
        None => {
            logger::error(&format!(
                "[remote] spawn_human_npc failed для player_id={}",
                player_id
            ));
            return;
        }
    };

    let npc = match unsafe { Npc::from_raw(entity_ptr, format!("Proxy#{}", player_id)) } {
        Some(n) => n,
        None => {
            logger::error(&format!(
                "[remote] Npc::from_raw failed для entity 0x{:X}",
                entity_ptr
            ));
            return;
        }
    };

    // Proxy-NPC не должен умирать от локального хаоса.
    npc.set_invulnerable(true);
    npc.set_demigod(true);

    // Отключаем AI controller — на свежем спавне behaviour-tasks ещё пустые,
    // но пусть будет на всякий случай (как в echo pilot).
    let ai_inactivated = unsafe {
        match CHumanAIController::from_human(npc.ptr()) {
            Some(aic) => aic.inactivate().is_ok(),
            None => false,
        }
    };

    // Авто-выдача оружия — для visible aim sync.
    let weapon_ok = npc.add_weapon(DEFAULT_REMOTE_WEAPON_ID, DEFAULT_REMOTE_WEAPON_AMMO);

    logger::info(&format!(
        "[remote] bound player {} ('{}') -> fresh NPC 0x{:X} \
         (ai_off={}, weapon_given={})",
        player_id, player_name, entity_ptr, ai_inactivated, weapon_ok
    ));

    map.insert(
        player_id,
        RemoteBinding {
            player_id,
            player_name: player_name.to_string(),
            npc_name: format!("Proxy#{}", player_id),
            npc,
            move_cmd: None,
            last_net_pos: None,
            interp_from: Vec3::ZERO,
            interp_to: Vec3::ZERO,
            interp_begin: Instant::now(),
            last_net_sample_at: None,
            remote_is_moving: false,
            remote_movement_mode: 0,
            aim_active: false,
        },
    );
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

    let dir_len_sq = dir.x * dir.x + dir.y * dir.y + dir.z * dir.z;
    let dir_valid = dir_len_sq > 0.0001;

    apply_network_motion_sample(
        binding,
        pos,
        snapshot.is_moving,
        snapshot.movement_mode,
    );
    // В стоянии — forward из сети; в движении корпус по шагу задаёт `apply_network_motion_sample`.
    if dir_valid && !snapshot.is_moving {
        let _ = binding.npc.set_forward(&dir);
    }
    apply_health(binding, snapshot);
    apply_aim(binding, snapshot);

    crate::overlay::state::update_player_ping(binding.player_id as u32, 0);
}

/// Применить состояние прицела к remote NPC.
///
/// - `is_aiming = true`  → SetupAimDir(npc, true, &dir)
/// - `is_aiming = false` → SetupAimDir(npc, false, &Zero) (только при смене)
///
/// Чтобы не нагружать движок повторными вызовами, флаг `aim_active`
/// в `RemoteBinding` отслеживает текущее состояние.
fn apply_aim(binding: &mut RemoteBinding, snapshot: &NetPlayerSnapshot) {
    let aim = HumanAim::new(binding.npc.ptr());

    match (snapshot.is_aiming, snapshot.aim_dir) {
        (true, Some(dir)) => {
            let dir_vec = Vec3 { x: dir.x, y: dir.y, z: dir.z };
            let ok = unsafe { aim.set_aim_dir(true, &dir_vec) };
            if !binding.aim_active {
                logger::info(&format!(
                    "[remote] {} (NPC '{}') aim ON dir=({:.2},{:.2},{:.2}) ok={}",
                    binding.player_name, binding.npc_name, dir.x, dir.y, dir.z, ok
                ));
            }
            binding.aim_active = true;
        }
        _ => {
            if binding.aim_active {
                let ok = unsafe { aim.set_aim_dir(false, &Vec3::ZERO) };
                logger::info(&format!(
                    "[remote] {} (NPC '{}') aim OFF ok={}",
                    binding.player_name, binding.npc_name, ok
                ));
                binding.aim_active = false;
            }
        }
    }
}

/// Записать новую сетевую позицию в состояние биндинга.
///
/// См. модульный комментарий вверху файла.
fn apply_network_motion_sample(
    binding: &mut RemoteBinding,
    target_pos: Vec3,
    is_moving: bool,
    movement_mode: u8,
) {
    let was_moving = binding.remote_is_moving;
    binding.remote_is_moving = is_moving;
    binding.remote_movement_mode = movement_mode;

    let prev_net = binding.last_net_pos;

    // (1) Первый snapshot — телепорт в стартовую точку.
    let Some(prev) = prev_net else {
        binding.npc.set_position(&target_pos);
        binding.last_net_pos = Some(target_pos);
        binding.interp_from = target_pos;
        binding.interp_to = target_pos;
        binding.interp_begin = Instant::now();
        binding.last_net_sample_at = Some(Instant::now());
        hard_reset_remote_locomotion(binding);
        return;
    };

    let delta = target_pos - prev;
    let dist = (delta.x * delta.x + delta.y * delta.y).sqrt();

    // (2) Реальный телепорт (loading screen / большой скачок).
    if dist >= TELEPORT_THRESHOLD_M {
        binding.npc.set_position(&target_pos);
        hard_reset_remote_locomotion(binding);
        binding.last_net_pos = Some(target_pos);
        binding.interp_from = target_pos;
        binding.interp_to = target_pos;
        binding.interp_begin = Instant::now();
        binding.last_net_sample_at = Some(Instant::now());
        return;
    }

    let now = Instant::now();
    let net_dt = binding
        .last_net_sample_at
        .as_ref()
        .map(|t| (now - *t).as_secs_f32())
        .unwrap_or(SNAPSHOT_DT_SECS)
        .clamp(0.045, 0.55);
    binding.last_net_sample_at = Some(now);

    // Новый сегмент интерполяции: от прошлой сетевой цели к новой.
    binding.interp_from = binding.interp_to;
    binding.interp_to = target_pos;
    binding.interp_begin = Instant::now();
    binding.last_net_pos = Some(target_pos);

    if is_moving {
        if !was_moving {
            hard_reset_remote_locomotion(binding);
        }
        if binding.move_cmd.is_none() {
            binding.move_cmd = unsafe { binding.npc.create_move_dir_command() };
        }
        if let Some(cmd) = binding.move_cmd {
            let applied = unsafe { cmd.apply_from_delta(prev, target_pos, net_dt) };
            if applied {
                unsafe { cmd.set_movement_mode_low_byte(movement_mode) };
                let dx = target_pos.x - prev.x;
                let dy = target_pos.y - prev.y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.025 {
                    let _ = binding.npc.set_forward(&Vec3 {
                        x: dx / len,
                        y: dy / len,
                        z: 0.0,
                    });
                }
            } else {
                binding.npc.set_position(&target_pos);
            }
        } else {
            binding.npc.set_position(&target_pos);
        }
    } else {
        hard_reset_remote_locomotion(binding);
    }
}

impl RemoteBinding {
    fn tick_interpolate(&mut self, _dt_secs: f32) {
        if self.last_net_pos.is_none() {
            return;
        }
        if self.remote_is_moving {
            return;
        }

        let elapsed = self.interp_begin.elapsed().as_secs_f32();
        let alpha = (elapsed / SNAPSHOT_DT_SECS).min(1.0);
        let goal = self.interp_from.lerp(&self.interp_to, alpha);

        self.npc.set_position(&goal);

        if let Some(npc_pos) = self.npc.get_position() {
            let drift_x = npc_pos.x - self.interp_to.x;
            let drift_y = npc_pos.y - self.interp_to.y;
            let drift = (drift_x * drift_x + drift_y * drift_y).sqrt();
            if drift > DRIFT_CORRECTION_M {
                self.npc.set_position(&self.interp_to);
            }
        }
    }
}

/// Применить health snapshot (death / restore).
fn apply_health(binding: &RemoteBinding, snapshot: &NetPlayerSnapshot) {
    if snapshot.is_dead {
        binding.npc.set_health(0.0);
        logger::debug(&format!(
            "[remote] {} (NPC '{}') — dead",
            binding.player_name, binding.npc_name
        ));
    } else {
        binding.npc.set_health(snapshot.health.max(1.0));
    }
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
/// - Death -> hp=0
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
