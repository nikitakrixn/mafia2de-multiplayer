//! Автоприменение пользовательского FOV.
//!
//! Логика:
//! - храним желаемый FOV
//! - на lifecycle событиях помечаем, что надо пере-применить
//! - на main thread пробуем записать FOV, когда камера уже готова

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use common::logger;

/// Желаемый FOV в bit-представлении `f32::to_bits()`.
static DESIRED_FOV_BITS: AtomicU32 = AtomicU32::new(75.0f32.to_bits());

/// Нужно ли пере-применить FOV.
static PENDING_APPLY: AtomicBool = AtomicBool::new(true);

fn desired_fov() -> f32 {
    f32::from_bits(DESIRED_FOV_BITS.load(Ordering::Acquire))
}

/// Текущий целевой FOV клиента.
pub fn get_desired_fov() -> f32 {
    desired_fov()
}

/// Изменить целевой FOV и пометить на пере-применение.
pub fn set_desired_fov(fov: f32) {
    DESIRED_FOV_BITS.store(fov.to_bits(), Ordering::Release);
    PENDING_APPLY.store(true, Ordering::Release);
    logger::info(&format!("[camera-state] desired FOV -> {fov:.1}"));
}

/// Запланировать пере-применение FOV.
pub fn request_apply() {
    PENDING_APPLY.store(true, Ordering::Release);
}

/// Вызывается на main thread каждый тик.
pub fn update_main_thread() {
    if !PENDING_APPLY.load(Ordering::Acquire) {
        return;
    }

    let fov = desired_fov();

    if sdk::game::camera::set_all_fov(fov) {
        PENDING_APPLY.store(false, Ordering::Release);
        logger::info(&format!("[camera-state] FOV applied: {fov:.1}"));
    }
}