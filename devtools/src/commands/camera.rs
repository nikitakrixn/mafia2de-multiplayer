//! FOV управление.

use common::logger;
use crate::tools::camera_state;

pub fn show_fov() {
    sdk::game::camera::log_status();
    logger::info(&format!(
        "[camera-state] desired FOV = {:.1}",
        camera_state::get_desired_fov()
    ));
}

pub fn adjust_fov(delta: f32) {
    let current = camera_state::get_desired_fov();
    let new_fov = (current + delta).clamp(30.0, 150.0);
    logger::info(&format!("FOV: {current:.1} → {new_fov:.1}"));
    camera_state::set_desired_fov(new_fov);
}

pub fn set_fov(fov: f32) {
    logger::info(&format!("Setting all cameras FOV: {fov:.1}"));
    camera_state::set_desired_fov(fov);
}