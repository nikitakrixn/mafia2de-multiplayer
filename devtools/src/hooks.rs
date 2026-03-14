//! Минимальные хуки для devtools: GameTick + FireEvent.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use minhook::{MH_STATUS, MinHook};
use sdk::{addresses, memory};

type GameTickAlwaysCallback = unsafe extern "C" fn(usize, usize);
type FireEventByIdFn = unsafe extern "C" fn(usize, i32, usize) -> usize;

static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_TICK: OnceLock<GameTickAlwaysCallback> = OnceLock::new();
static ORIGINAL_FIRE: OnceLock<FireEventByIdFn> = OnceLock::new();

unsafe extern "C" fn tick_detour(obj: usize, ctx: usize) {
    if let Some(orig) = ORIGINAL_TICK.get() {
        unsafe { orig(obj, ctx) };
    }
    crate::main_thread::on_main_thread_tick();
}

unsafe extern "C" fn fire_detour(mgr: usize, event_id: i32, data: usize) -> usize {
    crate::events::process_fired_event(event_id);
    if let Some(orig) = ORIGINAL_FIRE.get() {
        unsafe { orig(mgr, event_id, data) }
    } else {
        0
    }
}

fn map_status(prefix: &str, status: MH_STATUS) -> String {
    format!("{prefix}: {status:?}")
}

unsafe fn create_hook<F: Copy>(
    target: usize, detour: *const (), storage: &OnceLock<F>, name: &str,
) -> Result<(), String> {
    let trampoline = unsafe {
        MinHook::create_hook(target as _, detour as _)
            .map_err(|e| map_status(&format!("create_hook({name})"), e))?
    };
    let original: F = unsafe { std::mem::transmute_copy(&trampoline) };
    let _ = storage.set(original);
    logger::info(&format!("[hooks] created {name} at 0x{target:X}"));
    Ok(())
}

pub fn install() -> Result<(), String> {
    if HOOK_INSTALLED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    let base = memory::get_module_base(addresses::GAME_MODULE)
        .ok_or("game module not found")?;

    let tick_addr = base + addresses::functions::callbacks::GAME_TICK_ALWAYS_CB_CANDIDATE;
    let fire_addr = base + addresses::functions::callbacks::FIRE_EVENT_BY_ID;

    unsafe {
        create_hook(tick_addr, tick_detour as *const (), &ORIGINAL_TICK, "GameTick")?;
        create_hook(fire_addr, fire_detour as *const (), &ORIGINAL_FIRE, "FireEvent")?;
        MinHook::enable_all_hooks()
            .map_err(|e| map_status("enable_all", e))?;
    }

    logger::info("[hooks] devtools hooks installed");
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    if !HOOK_INSTALLED.swap(false, Ordering::AcqRel) {
        return Ok(());
    }
    unsafe {
        MinHook::disable_all_hooks()
            .map_err(|e| map_status("disable_all", e))?;
    }
    logger::info("[hooks] devtools hooks disabled");
    Ok(())
}