use windows::core::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::{GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE};
use windows::Win32::{
    Foundation::HINSTANCE,
    System::Console::AllocConsole,
};
use common::logger::{self, LogLevel, LogTarget};

fn dll_init() {
    unsafe {
        if AllocConsole().is_ok() {
            let handle: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE).unwrap();
            let mut mode: CONSOLE_MODE = CONSOLE_MODE(0);

            if GetConsoleMode(handle, &mut mode).is_ok() {
                let new_mode = mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
                let _ = SetConsoleMode(handle, new_mode);
            }
        }
    }

    
    // Инициализируем логгер
    if let Err(err) = logger::Logger::init(
        LogLevel::Debug, 
        LogTarget::Both, 
        Some("logs/mafia2de_client.log".to_string())
    ) {
        eprintln!("Не удалось инициализировать логгер: {}", err);
    }
    
    let _ = logger::info("====================================");
    let _ = logger::info("Mafia II DE Multiplayer Client v0.1");
    let _ = logger::info("====================================");
    let _ = logger::info("Клиент инициализирован");
    
    let _ = logger::trace("trace message");
    let _ = logger::debug("debug message");
    let _ = logger::info("info message");
    let _ = logger::warning("warning message");
    let _ = logger::error("error message");
    let _ = logger::critical("critical message");

}

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, reserved: u32) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => {
            dll_init();
            BOOL::from(true)
        },
        DLL_PROCESS_DETACH => {
            // При выгрузке DLL запишем в лог
            let _ = logger::info("Клиент завершает работу");
            BOOL::from(true)
        },
        _ => BOOL::from(true),
    }
}