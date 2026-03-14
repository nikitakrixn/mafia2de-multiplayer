//! Main-thread tick dispatcher для devtools.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use common::logger;
use crate::lua_queue;

const MAX_LUA_PER_TICK: usize = 8;
const STATE_REFRESH_MS: u64 = 250;

static IN_DRAIN: AtomicBool = AtomicBool::new(false);
static LAST_REFRESH_MS: AtomicU64 = AtomicU64::new(0);
static START: OnceLock<Instant> = OnceLock::new();

fn uptime_ms() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

pub fn on_main_thread_tick() {
    drain_lua_queue();
    refresh_state();
    crate::tools::noclip::tick_main_thread();
    crate::tools::camera_state::update_main_thread();
}

fn refresh_state() {
    let now = uptime_ms();
    let last = LAST_REFRESH_MS.load(Ordering::Acquire);
    if now.saturating_sub(last) < STATE_REFRESH_MS { return; }
    LAST_REFRESH_MS.store(now, Ordering::Release);
    let _ = crate::state::refresh_from_runtime();
}

fn drain_lua_queue() {
    if IN_DRAIN.swap(true, Ordering::AcqRel) { return; }
    let _guard = scopeguard(|| IN_DRAIN.store(false, Ordering::Release));

    for _ in 0..MAX_LUA_PER_TICK {
        let Some(cmd) = lua_queue::pop_front() else { break };
        match sdk::game::lua::exec_named(&cmd.code, &cmd.chunk_name) {
            Ok(()) => logger::info(&format!("[lua] ok: {}", cmd.chunk_name)),
            Err(e) if e.contains("не готова") => {
                lua_queue::push_front(cmd);
                break;
            }
            Err(e) => logger::error(&format!("[lua] {}: {e}", cmd.chunk_name)),
        }
    }
}

fn scopeguard<F: FnOnce()>(f: F) -> impl Drop {
    struct G<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for G<F> {
        fn drop(&mut self) { if let Some(f) = self.0.take() { f(); } }
    }
    G(Some(f))
}