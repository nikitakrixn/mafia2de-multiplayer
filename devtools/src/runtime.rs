use std::sync::atomic::{AtomicBool, Ordering};
use common::logger;

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

pub fn is_shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::Acquire)
}

pub fn shutdown() {
    if SHUTTING_DOWN.swap(true, Ordering::AcqRel) {
        return;
    }

    crate::state::mark_shutting_down();
    logger::info("[devtools] shutdown started");

    crate::tools::noclip::shutdown();

    if let Err(e) = crate::hooks::uninstall() {
        logger::error(&format!("[devtools] hook uninstall failed: {e}"));
    }

    logger::info("[devtools] shutdown finished");
}