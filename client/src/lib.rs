//! Клиентская DLL для Mafia II: DE Multiplayer.
//!
//! Инжектится лаунчером в процесс игры.
//! Точка входа — DllMain → initialize() в отдельном потоке.

mod debug_commands;
mod events;
mod hooks;
mod human_messages;
mod input;
mod lua_queue;
mod main_thread;
mod monitor;
mod player_events;
mod player_tracker;
mod runtime;
mod state;

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
            logger::info("Клиент завершает работу...");
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
        Some("logs/m2mp_client.log"),
    ) {
        eprintln!("[m2mp] Не удалось инициализировать логгер: {e}");
    }

    logger::info("======================================");
    logger::info("  Mafia II: DE Multiplayer Client");
    logger::info("  Версия 0.1.0 | x86_64");
    logger::info("======================================");

    sdk::game::log_module_info();

    // Инициализация подсистем
    lua_queue::init();
    player_tracker::init();
    player_events::init();
    let _ = state::refresh_from_runtime();

    sdk::game::lua::log_chain();
    sdk::game::callbacks::dump_interesting_events();

    // Установка хуков
    logger::info("Устанавливаю хуки...");
    if let Err(e) = hooks::install() {
        logger::error(&format!("Не удалось установить хуки: {e}"));
        return;
    }

    input::log_keybinds();
    logger::info("======================================");
    logger::info("  Клиент инициализирован!");
    logger::info("======================================");

    // Запускаем фоновый мониторинг и ввод
    // std::thread::spawn(monitor::run);
    input::run();
}