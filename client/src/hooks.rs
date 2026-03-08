//! Установка detour-hook'ов клиента.
//!
//! Сейчас здесь два hook'а:
//! 1. Game Tick Always callback
//!    Используется как безопасная main-thread точка для выполнения очереди задач.
//! 2. GameCallbackManager::FireEventById
//!    Используется для lifecycle-событий игры.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use minhook::{MH_STATUS, MinHook};
use sdk::{addresses, memory};

/// Один из существующих callback'ов на событии "Game Tick Always".
///
/// Prototype подтверждён практикой:
/// RCX = callback_object
/// RDX = dispatch_context
type GameTickAlwaysCallback = unsafe extern "C" fn(usize, usize);

/// Public fire path callback manager'а.
///
/// RCX = GameCallbackManager*
/// EDX = event_id
/// R8  = user_data/context
type FireEventByIdFn = unsafe extern "C" fn(usize, i32, usize) -> usize;

/// true если hook'и уже установлены.
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Оригинал detour target для Game Tick Always callback.
static ORIGINAL_GAME_TICK_ALWAYS: OnceLock<GameTickAlwaysCallback> = OnceLock::new();

/// Оригинал detour target для FireEventById.
static ORIGINAL_FIRE_EVENT_BY_ID: OnceLock<FireEventByIdFn> = OnceLock::new();

/// Detour для main-thread tick callback.
///
/// Сначала даём игре выполнить оригинальный callback,
/// потом обслуживаем очередь задач клиента на главном игровом потоке.
unsafe extern "C" fn game_tick_always_detour(callback_object: usize, dispatch_ctx: usize) {
    if let Some(original) = ORIGINAL_GAME_TICK_ALWAYS.get() {
        unsafe { original(callback_object, dispatch_ctx) };
    }

    crate::main_thread::on_main_thread_tick();
}

/// Detour для lifecycle event fire path.
///
/// Здесь мы не ломаем логику игры:
/// - сначала регистрируем событие в клиентском state/event слое
/// - затем передаём выполнение оригиналу.
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

fn map_status(prefix: &str, status: MH_STATUS) -> String {
    format!("{prefix}: {status:?}")
}

/// Устанавливает все hook'и клиента.
///
/// Повторный вызов безопасен.
pub fn install() -> Result<(), String> {
    if HOOK_INSTALLED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    let base = memory::get_module_base(addresses::GAME_MODULE)
        .ok_or_else(|| "failed to get game module base".to_string())?;

    let tick_target = base + addresses::functions::callbacks::GAME_TICK_ALWAYS_CB_CANDIDATE;
    let fire_target = base + addresses::functions::callbacks::FIRE_EVENT_BY_ID;

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

    Ok(())
}

/// Отключает hook'и клиента.
///
/// Важно:
/// для dev-цикла и hot-reload полезно иметь явный shutdown path.
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