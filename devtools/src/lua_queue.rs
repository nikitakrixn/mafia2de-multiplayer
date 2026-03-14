//! Очередь Lua команд для main thread

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use common::logger;

#[derive(Debug, Clone)]
pub struct QueuedLuaCommand {
    pub code: String,
    pub chunk_name: String,
}

static QUEUE: OnceLock<Mutex<VecDeque<QueuedLuaCommand>>> = OnceLock::new();

fn q() -> &'static Mutex<VecDeque<QueuedLuaCommand>> {
    QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn init() { let _ = q(); }

pub fn queue_exec_named(code: impl Into<String>, name: impl Into<String>) {
    let cmd = QueuedLuaCommand { code: code.into(), chunk_name: name.into() };
    match q().lock() {
        Ok(mut q) => q.push_back(cmd),
        Err(_) => logger::error("[lua-queue] mutex poisoned"),
    }
}

pub fn pop_front() -> Option<QueuedLuaCommand> {
    q().lock().ok()?.pop_front()
}

pub fn push_front(cmd: QueuedLuaCommand) {
    if let Ok(mut q) = q().lock() { q.push_front(cmd); }
}