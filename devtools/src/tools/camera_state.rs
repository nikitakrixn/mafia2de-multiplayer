//! Auto-apply FOV

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use common::logger;

static DESIRED_FOV_BITS: AtomicU32 = AtomicU32::new(75.0f32.to_bits());
static PENDING_APPLY: AtomicBool = AtomicBool::new(true);

fn desired_fov() -> f32 {
    f32::from_bits(DESIRED_FOV_BITS.load(Ordering::Acquire))
}

pub fn get_desired_fov() -> f32 { desired_fov() }

pub fn set_desired_fov(fov: f32) {
    DESIRED_FOV_BITS.store(fov.to_bits(), Ordering::Release);
    PENDING_APPLY.store(true, Ordering::Release);
    logger::info(&format!("[camera-state] desired FOV -> {fov:.1}"));
}

pub fn request_apply() {
    PENDING_APPLY.store(true, Ordering::Release);
}

pub fn update_main_thread() {
    if !PENDING_APPLY.load(Ordering::Acquire) { return; }
    let fov = desired_fov();
    if sdk::game::camera::set_all_fov(fov) {
        PENDING_APPLY.store(false, Ordering::Release);
        logger::info(&format!("[camera-state] FOV applied: {fov:.1}"));
    }
}