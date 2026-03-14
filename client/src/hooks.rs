//! Hook installation layer for the client runtime.
//!
//! Hooks:
//! - Game Tick Always callback    → main-thread task execution
//! - FireEventById                → lifecycle/session events
//! - EntityMessageRegistry_Broadcast → active player human messages
//! - IDXGISwapChain1::Present1   → overlay rendering

use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use minhook::{MH_STATUS, MinHook};
use sdk::{addresses, memory};

// ═══════════════════════════════════════════════════════════════
//  Type definitions
// ═══════════════════════════════════════════════════════════════

type GameTickAlwaysCallback = unsafe extern "C" fn(usize, usize);
type FireEventByIdFn        = unsafe extern "C" fn(usize, i32, usize) -> usize;
type EntityBroadcastFn      = unsafe extern "C" fn(usize, usize) -> u8;

/// IDXGISwapChain::Present — vtable[8]
/// HRESULT (this, UINT SyncInterval, UINT Flags)
type PresentFn = unsafe extern "system" fn(*mut c_void, u32, u32) -> i32;

/// IDXGISwapChain1::Present1 — vtable[22]
/// HRESULT (this, UINT SyncInterval, UINT PresentFlags, const DXGI_PRESENT_PARAMETERS*)
type Present1Fn = unsafe extern "system" fn(*mut c_void, u32, u32, *const c_void) -> i32;

// ═══════════════════════════════════════════════════════════════
//  Statics
// ═══════════════════════════════════════════════════════════════

static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

static ORIGINAL_GAME_TICK_ALWAYS: OnceLock<GameTickAlwaysCallback> = OnceLock::new();
static ORIGINAL_FIRE_EVENT_BY_ID: OnceLock<FireEventByIdFn>       = OnceLock::new();
static ORIGINAL_ENTITY_BROADCAST: OnceLock<EntityBroadcastFn>     = OnceLock::new();
static ORIGINAL_PRESENT:          OnceLock<PresentFn>             = OnceLock::new();
static ORIGINAL_PRESENT1:         OnceLock<Present1Fn>            = OnceLock::new();

// ═══════════════════════════════════════════════════════════════
//  Detours
// ═══════════════════════════════════════════════════════════════

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

/// IDXGISwapChain::Present detour (vtable[8]).
/// Некоторые игры вызывают Present, другие Present1.
/// Хукаем оба на всякий случай.
unsafe extern "system" fn present_detour(
    swapchain: *mut c_void,
    sync_interval: u32,
    flags: u32,
) -> i32 {
    crate::overlay::render_frame();

    if let Some(original) = ORIGINAL_PRESENT.get() {
        unsafe { original(swapchain, sync_interval, flags) }
    } else {
        0
    }
}

/// IDXGISwapChain1::Present1 detour (vtable[22]).
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

    // ── Engine hooks ───────────────────────────────────────────

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

    match install_present_hooks() {
        Ok(()) => {}
        Err(e) => {
            logger::warn(&format!("[hooks] Present hooks deferred: {e}"));
        }
    }

    unsafe {
        MinHook::enable_all_hooks()
            .map_err(|e| map_status("enable_all_hooks", e))?;
    }

    logger::info("[hooks] all hooks installed and enabled");
    Ok(())
}

/// IDXGISwapChain / IDXGISwapChain1 vtable layout:
///
/// IUnknown:
///   [0] QueryInterface
///   [1] AddRef
///   [2] Release
/// IDXGIObject:
///   [3] SetPrivateData
///   [4] SetPrivateDataInterface
///   [5] GetPrivateData
///   [6] GetParent
/// IDXGIDeviceSubObject:
///   [7] GetDevice
/// IDXGISwapChain:
///   [8]  Present              ← хукаем
///   [9]  GetBuffer
///   [10] SetFullscreenState
///   [11] GetFullscreenState
///   [12] GetDesc
///   [13] ResizeBuffers
///   [14] ResizeTarget
///   [15] GetContainingOutput
///   [16] GetFrameStatistics
///   [17] GetLastPresentCount
/// IDXGISwapChain1:
///   [18] GetDesc1
///   [19] GetFullscreenDesc
///   [20] GetHwnd
///   [21] GetCoreWindow
///   [22] Present1             ← хукаем (M2DE использует это!)

const PRESENT_VTABLE_INDEX: usize = 8;
const PRESENT1_VTABLE_INDEX: usize = 22;

fn install_present_hooks() -> Result<(), String> {
    // Уже установлены?
    if ORIGINAL_PRESENT1.get().is_some() {
        return Ok(());
    }

    let sc_ptr = sdk::game::render::get_swapchain_ptr()
        .ok_or("swapchain not ready")?;

    let vtable = unsafe { *(sc_ptr as *const *const usize) };
    if vtable.is_null() {
        return Err("swapchain vtable is null".to_string());
    }

    let present_addr = unsafe { *vtable.add(PRESENT_VTABLE_INDEX) };
    if present_addr == 0 || !memory::is_valid_ptr(present_addr) {
        return Err("Present vtable entry is null".to_string());
    }

    logger::info(&format!(
        "[hooks] swapchain vtable: Present[{PRESENT_VTABLE_INDEX}]=0x{present_addr:X}"
    ));

    unsafe {
        create_hook(
            present_addr,
            present_detour as *const (),
            &ORIGINAL_PRESENT,
            "Present",
        )?;
    }

    let present1_addr = unsafe { *vtable.add(PRESENT1_VTABLE_INDEX) };
    if present1_addr == 0 || !memory::is_valid_ptr(present1_addr) {
        logger::warn("[hooks] Present1 vtable entry is null — game may only use Present");
    } else {
        logger::info(&format!(
            "[hooks] swapchain vtable: Present1[{PRESENT1_VTABLE_INDEX}]=0x{present1_addr:X}"
        ));

        unsafe {
            create_hook(
                present1_addr,
                present1_detour as *const (),
                &ORIGINAL_PRESENT1,
                "Present1",
            )?;
        }
    }

    Ok(())
}

/// Попробовать установить Present hooks если ещё не установлены.
pub fn try_deferred_present_hook() {
    if ORIGINAL_PRESENT1.get().is_some() {
        return; // уже установлены
    }

    if let Ok(()) = install_present_hooks() {
        unsafe {
            let _ = MinHook::enable_all_hooks();
        }
        logger::info("[hooks] deferred Present hooks installed");
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

/// Установлены ли хуки?
pub fn is_installed() -> bool {
    HOOK_INSTALLED.load(Ordering::Acquire)
}