// Клиентская DLL для Mafia II: DE Multiplayer

mod events;
mod hooks;
mod human_messages;
mod input;
mod lua_queue;
mod main_thread;
mod multiplayer_test;
mod network;
mod overlay;
mod player_events;
mod player_tracker;
mod remote_players;
mod state;
mod utils;
mod vehicle_tracker;

use common::logger;
use std::ffi::c_void;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Console::AllocConsole;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(_module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(initialize);
            1
        }
        DLL_PROCESS_DETACH => {
            logger::info("Клиент завершает работу...");
            state::shutdown();
            1
        }
        _ => 1,
    }
}

fn initialize() {
    unsafe {
        let _ = AllocConsole();
    }

    if let Err(e) = logger::init(
        logger::Level::Debug,
        logger::Target::Both,
        Some("logs/m2mp_client.log"),
    ) {
        eprintln!("[m2mp] Ошибка инициализации логгера: {e}");
    }

    logger::info("======================================");
    logger::info("  Mafia II: DE Multiplayer Client");
    logger::info("  v0.1.0 | x86_64 | egui D3D11 UI");
    logger::info("======================================");

    sdk::game::log_module_info();

    // Подсистемы
    lua_queue::init();
    player_tracker::init();
    player_events::init();
    vehicle_tracker::init();
    network::init();
    remote_players::init();
    let _ = state::refresh_from_runtime();
    sdk::game::lua::log_chain();

    // Хуки
    logger::info("Установка хуков...");
    if let Err(e) = hooks::install() {
        logger::error(&format!("Ошибка установки хуков: {e}"));
        return;
    }

    logger::info("[init] Инициализация оверлея...");
    match overlay::init() {
        Ok(()) => logger::info("[init] Оверлей готов (или отложен)"),
        Err(e) => logger::error(&format!("[init] Ошибка оверлея: {e}")),
    }

    input::log_keybinds();
    logger::info("======================================");
    logger::info("  Клиент инициализирован!");
    logger::info("======================================");

    // Input loop — блокирует поток до shutdown
    input::run();
}
