//! DevTools DLL для Mafia II: DE — инструменты разработчика.
//!
//! Инжектится отдельно от мультиплеерного клиента.
//! Содержит: noclip, camera control, entity scanner,
//! debug commands, player probe, Lua console.

mod runtime;
mod state;
mod events;
mod hooks;
mod main_thread;
mod input;
mod lua_queue;
mod commands;
mod tools;

use std::ffi::c_void;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Console::AllocConsole;
use common::logger;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(
    _module: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(initialize);
            1
        }
        DLL_PROCESS_DETACH => {
            logger::info("[devtools] unloading...");
            runtime::shutdown();
            1
        }
        _ => 1,
    }
}

fn initialize() {
    unsafe { let _ = AllocConsole(); }

    if let Err(e) = logger::init(
        logger::Level::Debug,
        logger::Target::Both,
        Some("logs/m2mp_devtools.log"),
    ) {
        eprintln!("[devtools] logger init failed: {e}");
    }

    logger::info("══════════════════════════════════════");
    logger::info("  Mafia II: DE — DevTools");
    logger::info("  Version 0.1.0 | x86_64");
    logger::info("══════════════════════════════════════");

    sdk::game::log_module_info();

    lua_queue::init();
    let _ = state::refresh_from_runtime();

    sdk::game::lua::log_chain();
    sdk::game::callbacks::dump_interesting_events();

    logger::info("[devtools] installing hooks...");
    if let Err(e) = hooks::install() {
        logger::error(&format!("[devtools] hook install failed: {e}"));
        return;
    }

    input::log_keybinds();
    logger::info("══════════════════════════════════════");
    logger::info("  DevTools ready!");
    logger::info("══════════════════════════════════════");

    input::run();
}