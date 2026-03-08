use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;

use crate::state;

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

pub fn is_shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::Acquire)
}

pub fn shutdown() {
    if SHUTTING_DOWN.swap(true, Ordering::AcqRel) {
        return;
    }

    state::mark_shutting_down();
    logger::info("[runtime] shutdown started");

    if let Err(e) = crate::hooks::uninstall() {
        logger::error(&format!("[runtime] hook uninstall failed: {e}"));
    }

    logger::info("[runtime] shutdown finished");
}