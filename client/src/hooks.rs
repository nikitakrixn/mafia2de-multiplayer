//! Hook installation layer for the client runtime.
//!
//! Current hooks:
//! - Game Tick Always callback    -> main-thread task execution
//! - FireEventById                -> lifecycle/session events
//! - EntityMessageRegistry_Broadcast -> active player human messages

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use minhook::{MH_STATUS, MinHook};
use sdk::{addresses, memory};

type GameTickAlwaysCallback = unsafe extern "C" fn(usize, usize);
type FireEventByIdFn = unsafe extern "C" fn(usize, i32, usize) -> usize;
type EntityBroadcastFn = unsafe extern "C" fn(usize, usize) -> u8;

static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

static ORIGINAL_GAME_TICK_ALWAYS: OnceLock<GameTickAlwaysCallback> = OnceLock::new();
static ORIGINAL_FIRE_EVENT_BY_ID: OnceLock<FireEventByIdFn> = OnceLock::new();
static ORIGINAL_ENTITY_BROADCAST: OnceLock<EntityBroadcastFn> = OnceLock::new();

/// Late callback from Game Tick Always.
/// We let the original callback run first, then execute our own main-thread services.
unsafe extern "C" fn game_tick_always_detour(callback_object: usize, dispatch_ctx: usize) {
    if let Some(original) = ORIGINAL_GAME_TICK_ALWAYS.get() {
        unsafe { original(callback_object, dispatch_ctx) };
    }

    crate::main_thread::on_main_thread_tick();
}

/// Public lifecycle event fire path.
/// This is where menu/loading/pause/session events are observed.
unsafe extern "C" fn fire_event_by_id_detour(
    manager: usize,
    event_id: i32,
    user_data: usize,
) -> usize {
    crate::events::process_fired_event(event_id);

    if let Some(original) = ORIGINAL_FIRE_EVENT_BY_ID.get() {
        unsafe { original(manager, event_id, user_data) }
    } else {
        0
    }
}

/// Central entity/human message broadcast path.
/// We use this as a practical high-level event source for the active player.
unsafe extern "C" fn entity_broadcast_detour(entity_ptr: usize, msg_ptr: usize) -> u8 {
    crate::human_messages::process_broadcast(entity_ptr, msg_ptr);

    if let Some(original) = ORIGINAL_ENTITY_BROADCAST.get() {
        unsafe { original(entity_ptr, msg_ptr) }
    } else {
        0
    }
}

fn map_status(prefix: &str, status: MH_STATUS) -> String {
    format!("{prefix}: {status:?}")
}

/// Installs all client hooks.
///
/// Safe to call more than once.
pub fn install() -> Result<(), String> {
    if HOOK_INSTALLED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    let base = memory::get_module_base(addresses::GAME_MODULE)
        .ok_or_else(|| "failed to get game module base".to_string())?;

    let tick_target = base + addresses::functions::callbacks::GAME_TICK_ALWAYS_CB_CANDIDATE;
    let fire_target = base + addresses::functions::callbacks::FIRE_EVENT_BY_ID;
    let broadcast_target = base + addresses::functions::entity_messages::BROADCAST;

    let tick_trampoline = unsafe {
        MinHook::create_hook(tick_target as _, game_tick_always_detour as _)
            .map_err(|e| map_status("MinHook::create_hook(GameTickAlways) failed", e))?
    };
    let tick_original: GameTickAlwaysCallback = unsafe { std::mem::transmute(tick_trampoline) };
    let _ = ORIGINAL_GAME_TICK_ALWAYS.set(tick_original);

    let fire_trampoline = unsafe {
        MinHook::create_hook(fire_target as _, fire_event_by_id_detour as _)
            .map_err(|e| map_status("MinHook::create_hook(FireEventById) failed", e))?
    };
    let fire_original: FireEventByIdFn = unsafe { std::mem::transmute(fire_trampoline) };
    let _ = ORIGINAL_FIRE_EVENT_BY_ID.set(fire_original);

    let broadcast_trampoline = unsafe {
        MinHook::create_hook(broadcast_target as _, entity_broadcast_detour as _)
            .map_err(|e| map_status("MinHook::create_hook(EntityBroadcast) failed", e))?
    };
    let broadcast_original: EntityBroadcastFn =
        unsafe { std::mem::transmute(broadcast_trampoline) };
    let _ = ORIGINAL_ENTITY_BROADCAST.set(broadcast_original);

    unsafe {
        MinHook::enable_all_hooks()
            .map_err(|e| map_status("MinHook::enable_all_hooks failed", e))?
    };

    logger::info(&format!(
        "[hooks] installed Game Tick Always hook at 0x{tick_target:X}"
    ));
    logger::info(&format!(
        "[hooks] installed FireEventById hook at 0x{fire_target:X}"
    ));
    logger::info(&format!(
        "[hooks] installed EntityMessageRegistry_Broadcast hook at 0x{broadcast_target:X}"
    ));

    Ok(())
}

/// Disables all installed hooks.
pub fn uninstall() -> Result<(), String> {
    if !HOOK_INSTALLED.swap(false, Ordering::AcqRel) {
        return Ok(());
    }

    unsafe {
        MinHook::disable_all_hooks()
            .map_err(|e| map_status("MinHook::disable_all_hooks failed", e))?;
    }

    logger::info("[hooks] all hooks disabled");
    Ok(())
}

pub fn is_installed() -> bool {
    HOOK_INSTALLED.load(Ordering::Acquire)
}