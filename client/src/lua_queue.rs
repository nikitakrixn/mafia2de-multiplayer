use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use common::logger;

#[derive(Debug, Clone)]
pub struct QueuedLuaCommand {
    pub code: String,
    pub chunk_name: String,
}

static LUA_QUEUE: OnceLock<Mutex<VecDeque<QueuedLuaCommand>>> = OnceLock::new();

fn queue() -> &'static Mutex<VecDeque<QueuedLuaCommand>> {
    LUA_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn init() {
    let _ = queue();
}

#[allow(dead_code)]
pub fn queue_exec(code: impl Into<String>) {
    queue_exec_named(code, "=m2mp_queue");
}

pub fn queue_exec_named(code: impl Into<String>, chunk_name: impl Into<String>) {
    let cmd = QueuedLuaCommand {
        code: code.into(),
        chunk_name: chunk_name.into(),
    };

    match queue().lock() {
        Ok(mut q) => {
            q.push_back(cmd);
            logger::debug(&format!("[lua-queue] queued, len={}", q.len()));
        }
        Err(_) => logger::error("[lua-queue] mutex poisoned in queue_exec_named"),
    }
}

pub fn pop_front() -> Option<QueuedLuaCommand> {
    match queue().lock() {
        Ok(mut q) => q.pop_front(),
        Err(_) => {
            logger::error("[lua-queue] mutex poisoned in pop_front");
            None
        }
    }
}

pub fn push_front(cmd: QueuedLuaCommand) {
    match queue().lock() {
        Ok(mut q) => {
            q.push_front(cmd);
        }
        Err(_) => logger::error("[lua-queue] mutex poisoned in push_front"),
    }
}

#[allow(dead_code)]
pub fn len() -> usize {
    match queue().lock() {
        Ok(q) => q.len(),
        Err(_) => 0,
    }
}
