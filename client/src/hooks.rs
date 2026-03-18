//! Хуки для перехвата игровых функций
//!
//! Перехватываем:
//! - Game Tick Always — главный тик игры, запускаем наш main thread
//! - FireEventById — события жизненного цикла (миссии, пауза и т.д.)
//! - EntityMessageRegistry_Broadcast — сообщения между сущностями
//! - IDXGISwapChain1::Present1 — рендер, встраиваем egui overlay

use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use minhook::{MH_STATUS, MinHook};
use sdk::{addresses, memory};

// ═══════════════════════════════════════════════════════════════
//  Типы функций
// ═══════════════════════════════════════════════════════════════

type GameTickAlwaysCallback = unsafe extern "C" fn(usize, usize);
type FireEventByIdFn        = unsafe extern "C" fn(usize, i32, usize) -> usize;
type EntityBroadcastFn      = unsafe extern "C" fn(usize, usize) -> u8;
type Present1Fn             = unsafe extern "system" fn(*mut c_void, u32, u32, *const c_void) -> i32;

// ═══════════════════════════════════════════════════════════════
//  Оригиналы функций
// ═══════════════════════════════════════════════════════════════

static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

static ORIGINAL_GAME_TICK_ALWAYS: OnceLock<GameTickAlwaysCallback> = OnceLock::new();
static ORIGINAL_FIRE_EVENT_BY_ID: OnceLock<FireEventByIdFn>       = OnceLock::new();
static ORIGINAL_ENTITY_BROADCAST: OnceLock<EntityBroadcastFn>     = OnceLock::new();
static ORIGINAL_PRESENT1:         OnceLock<Present1Fn>            = OnceLock::new();

// ═══════════════════════════════════════════════════════════════
//  Detour функции
// ═══════════════════════════════════════════════════════════════

unsafe extern "C" fn game_tick_always_detour(callback_object: usize, dispatch_ctx: usize) {
    if let Some(original) = ORIGINAL_GAME_TICK_ALWAYS.get() {
        unsafe { original(callback_object, dispatch_ctx) };
    }
    crate::main_thread::on_main_thread_tick();
}

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

unsafe extern "C" fn entity_broadcast_detour(entity_ptr: usize, msg_ptr: usize) -> u8 {
    crate::human_messages::process_broadcast(entity_ptr, msg_ptr);

    if let Some(original) = ORIGINAL_ENTITY_BROADCAST.get() {
        unsafe { original(entity_ptr, msg_ptr) }
    } else {
        0
    }
}

unsafe extern "system" fn present1_detour(
    swapchain: *mut c_void,
    sync_interval: u32,
    present_flags: u32,
    present_params: *const c_void,
) -> i32 {
    crate::overlay::render_frame();

    if let Some(original) = ORIGINAL_PRESENT1.get() {
        unsafe { original(swapchain, sync_interval, present_flags, present_params) }
    } else {
        0
    }
}

// ═══════════════════════════════════════════════════════════════
//  Install / uninstall
// ═══════════════════════════════════════════════════════════════

fn map_status(prefix: &str, status: MH_STATUS) -> String {
    format!("{prefix}: {status:?}")
}

unsafe fn create_hook<F: Copy>(
    target: usize,
    detour: *const (),
    storage: &OnceLock<F>,
    name: &str,
) -> Result<(), String> {
    let trampoline = unsafe {
        MinHook::create_hook(target as _, detour as _)
            .map_err(|e| map_status(&format!("create_hook({name})"), e))?
    };
    let original: F = unsafe { std::mem::transmute_copy(&trampoline) };
    let _ = storage.set(original);
    logger::info(&format!("[hooks] created {name} hook at 0x{target:X}"));
    Ok(())
}

pub fn install() -> Result<(), String> {
    if HOOK_INSTALLED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    let base = memory::get_module_base(addresses::GAME_MODULE)
        .ok_or_else(|| "failed to get game module base".to_string())?;

    let tick_target = base + addresses::functions::callbacks::GAME_TICK_ALWAYS_CB_CANDIDATE;
    let fire_target = base + addresses::functions::callbacks::FIRE_EVENT_BY_ID;
    let broadcast_target = base + addresses::functions::entity_messages::BROADCAST;

    unsafe {
        create_hook(
            tick_target,
            game_tick_always_detour as *const (),
            &ORIGINAL_GAME_TICK_ALWAYS,
            "GameTickAlways",
        )?;

        create_hook(
            fire_target,
            fire_event_by_id_detour as *const (),
            &ORIGINAL_FIRE_EVENT_BY_ID,
            "FireEventById",
        )?;

        create_hook(
            broadcast_target,
            entity_broadcast_detour as *const (),
            &ORIGINAL_ENTITY_BROADCAST,
            "EntityBroadcast",
        )?;
    }

    // Present1 может быть не готов сразу
    match install_present1_hook() {
        Ok(()) => {}
        Err(e) => {
            logger::warn(&format!("[hooks] Present1 hook deferred: {e}"));
        }
    }

    unsafe {
        MinHook::enable_all_hooks()
            .map_err(|e| map_status("enable_all_hooks", e))?;
    }

    logger::info("[hooks] all hooks installed and enabled");
    Ok(())
}

/// IDXGISwapChain1 vtable layout: [22] Present1
const PRESENT1_VTABLE_INDEX: usize = 22;

fn install_present1_hook() -> Result<(), String> {
    if ORIGINAL_PRESENT1.get().is_some() {
        return Ok(());
    }

    let sc_ptr = sdk::game::render::get_swapchain_ptr()
        .ok_or("swapchain not ready")?;

    let vtable = unsafe { *(sc_ptr as *const *const usize) };
    if vtable.is_null() {
        return Err("swapchain vtable is null".to_string());
    }

    let present1_addr = unsafe { *vtable.add(PRESENT1_VTABLE_INDEX) };
    if present1_addr == 0 || !memory::is_valid_ptr(present1_addr) {
        return Err("Present1 vtable entry is null".to_string());
    }

    logger::info(&format!(
        "[hooks] hooking ONLY Present1[{PRESENT1_VTABLE_INDEX}]=0x{present1_addr:X}"
    ));

    unsafe {
        create_hook(
            present1_addr,
            present1_detour as *const (),
            &ORIGINAL_PRESENT1,
            "Present1",
        )?;
    }

    Ok(())
}

pub fn try_deferred_present_hook() {
    if ORIGINAL_PRESENT1.get().is_some() {
        return;
    }

    if let Ok(()) = install_present1_hook() {
        unsafe {
            let _ = MinHook::enable_all_hooks();
        }
        logger::info("[hooks] deferred Present1 hook installed");
    }
}

pub fn uninstall() -> Result<(), String> {
    if !HOOK_INSTALLED.swap(false, Ordering::AcqRel) {
        return Ok(());
    }

    unsafe {
        MinHook::disable_all_hooks()
            .map_err(|e| map_status("disable_all_hooks", e))?;
    }

    logger::info("[hooks] all hooks disabled");
    Ok(())
}