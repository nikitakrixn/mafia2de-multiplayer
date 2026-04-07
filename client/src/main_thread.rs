use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use common::logger;
use sdk::game::lua;

use crate::lua_queue::{self, QueuedLuaCommand};

const MAX_LUA_PER_TICK: usize = 8;
const STATE_REFRESH_INTERVAL_MS: u64 = 250;
const PING_UPDATE_INTERVAL_MS: u64 = 2000;

static IN_DRAIN: AtomicBool = AtomicBool::new(false);
static LAST_STATE_REFRESH_MS: AtomicU64 = AtomicU64::new(0);
static LAST_PING_UPDATE_MS: AtomicU64 = AtomicU64::new(0);
static START_TIME: OnceLock<Instant> = OnceLock::new();

fn uptime_ms() -> u64 {
    START_TIME.get_or_init(Instant::now).elapsed().as_millis() as u64
}

struct ReentrancyGuard;

impl Drop for ReentrancyGuard {
    fn drop(&mut self) {
        IN_DRAIN.store(false, Ordering::Release);
    }
}

enum ExecResult {
    Ok,
    NotReady(QueuedLuaCommand),
    Failed,
}

pub fn on_main_thread_tick() {
    drain_lua_queue(MAX_LUA_PER_TICK);
    refresh_state_if_needed();
    update_ping_if_needed();

    crate::multiplayer::on_main_thread_tick();
    crate::hooks::try_deferred_present_hook();
    crate::overlay::state::sync_from_game();
}

fn refresh_state_if_needed() {
    let now = uptime_ms();
    let last = LAST_STATE_REFRESH_MS.load(Ordering::Acquire);
    if now.saturating_sub(last) < STATE_REFRESH_INTERVAL_MS { return; }
    LAST_STATE_REFRESH_MS.store(now, Ordering::Release);
    let _ = crate::state::refresh_from_runtime();
}

fn update_ping_if_needed() {
    let now = uptime_ms();
    let last = LAST_PING_UPDATE_MS.load(Ordering::Acquire);
    if now.saturating_sub(last) < PING_UPDATE_INTERVAL_MS { return; }
    LAST_PING_UPDATE_MS.store(now, Ordering::Release);
    crate::overlay::demo::simulate_pings();
}

pub fn drain_lua_queue(max_per_tick: usize) -> usize {
    if max_per_tick == 0 { return 0; }
    if IN_DRAIN.swap(true, Ordering::AcqRel) { return 0; }

    let _guard = ReentrancyGuard;
    let mut processed = 0usize;

    for _ in 0..max_per_tick {
        let Some(cmd) = lua_queue::pop_front() else { break; };

        match exec_one(cmd) {
            ExecResult::Ok => processed += 1,
            ExecResult::NotReady(cmd) => {
                lua_queue::push_front(cmd);
                break;
            }
            ExecResult::Failed => processed += 1,
        }
    }

    if processed != 0 {
        logger::debug(&format!("[main-thread] обработано {processed} Lua команд(ы)"));
    }

    processed
}

fn exec_one(cmd: QueuedLuaCommand) -> ExecResult {
    // Проверяем, это команда из консоли?
    let console_id = if cmd.chunk_name.starts_with("=console:") {
        cmd.chunk_name[9..].parse::<u32>().ok()
    } else {
        None
    };

    match lua::exec_named(&cmd.code, &cmd.chunk_name) {
        Ok(()) => {
            logger::info(&format!("[lua-main] ok: {}", cmd.chunk_name));
            if let Some(id) = console_id {
                crate::overlay::state::update_console_result(
                    id,
                    crate::overlay::state::ConsoleResult::Ok,
                );
            }
            ExecResult::Ok
        }
        Err(err) => {
            if err.contains("Lua VM not ready") {
                ExecResult::NotReady(cmd)
            } else {
                logger::error(&format!("[lua-main] ошибка: {} -> {}", cmd.chunk_name, err));
                if let Some(id) = console_id {
                    crate::overlay::state::update_console_result(
                        id,
                        crate::overlay::state::ConsoleResult::Error(err),
                    );
                }
                ExecResult::Failed
            }
        }
    }
}