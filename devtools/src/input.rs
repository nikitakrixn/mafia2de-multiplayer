//! Горячие клавиши devtools — F1-F12, Num+/-.

use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_DELETE,
    VK_F1, VK_F2, VK_F3, VK_F5, VK_F6,
    VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
    VK_ADD, VK_SUBTRACT, VK_MULTIPLY, VK_DIVIDE,
    VK_INSERT,
    VIRTUAL_KEY,
};
use common::logger;
use sdk::game::Player;
use crate::{commands, runtime, lua_queue};

const POLL_MS: u64 = 100;

fn just_pressed(vk: VIRTUAL_KEY) -> bool {
    let state = unsafe { GetAsyncKeyState(vk.0 as i32) };
    (state & 0x0001) != 0
}

pub fn log_keybinds() {
    logger::info("  DevTools keybinds:");
    logger::info("    DELETE  — Unload devtools");
    logger::info("    INSERT  — Send test Lua command");
    logger::info("    F1      — Entity factory registry");
    logger::info("    F2      — Scan cached entities");
    logger::info("    F3      — Player status");
    logger::info("    F4      — Toggle noclip");
    logger::info("    F5      — Teleport to test point");
    logger::info("    F6      — Toggle god mode");
    logger::info("    F7      — SDS line dump");
    logger::info("    F8      — Player memory dump");
    logger::info("    F9      — Show balance");
    logger::info("    F10     — Show FOV");
    logger::info("    F11     — Dump frame directions");
    logger::info("    F12     — Give Thompson + 200");
    logger::info("    Num+/-  — FOV ±5");
    logger::info("    Num*    — Reset FOV to 65");
    logger::info("    Num/    — Set FOV to 75");
}

pub fn run() {
    logger::debug("[input] devtools input loop started");

    loop {
        if runtime::is_shutting_down() { break; }
        std::thread::sleep(Duration::from_millis(POLL_MS));

        if just_pressed(VK_DELETE) {
            logger::info("[devtools] manual shutdown");
            runtime::shutdown();
            break;
        }

        if just_pressed(VK_INSERT) {
            lua_queue::queue_exec_named(
                r#"print("[devtools] Lua test OK")"#,
                "=devtools_test",
            );
        }

        let player = Player::get_active();

        if just_pressed(VK_F1) { commands::entity::dump_factory_registry(); }
        if just_pressed(VK_F2) { commands::entity::scan_all_cached(); }
        if just_pressed(VK_F3) {
            if let Some(ref p) = player { commands::player::show_status(p); }
        }
        if just_pressed(VK_F5) {
            if let Some(ref p) = player { commands::player::teleport(p); }
        }
        if just_pressed(VK_F6) { commands::health::toggle_god_mode(); }
        if just_pressed(VK_F7) { commands::world::dump_sds_lines(); }
        if just_pressed(VK_F8) {
            crate::tools::player_probe::dump_player_range(0x00, 0x80);
        }
        if just_pressed(VK_F9) {
            if let Some(ref p) = player { commands::player::show_balance(p); }
        }
        if just_pressed(VK_F10) { commands::camera::show_fov(); }
        if just_pressed(VK_F11) {
            crate::tools::player_probe::dump_frame_directions();
        }
        if just_pressed(VK_F12) {
            if let Some(ref p) = player {
                commands::player::give_weapon(
                    p,
                    sdk::addresses::constants::weapons::THOMPSON_1928,
                    200,
                    "Thompson",
                );
            }
        }

        if just_pressed(VK_ADD) { commands::camera::adjust_fov(5.0); }
        if just_pressed(VK_SUBTRACT) { commands::camera::adjust_fov(-5.0); }
        if just_pressed(VK_MULTIPLY) { commands::camera::show_fov(); }
        if just_pressed(VK_DIVIDE) { commands::camera::set_fov(75.0); }
    }
}