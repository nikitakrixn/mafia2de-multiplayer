use windows::core::BOOL;
use windows::Win32::{
    Foundation::HINSTANCE,
    System::Console::AllocConsole,
};
use common::logger::{self, LogLevel, LogTarget};
use mafia2de_sdk::globals;

fn dll_init() {
    unsafe { 
        // Создаем консоль для отображения сообщений логгера
        match AllocConsole() {
            Ok(_) => (),
            Err(e) => eprintln!("Ошибка при создании консоли: {:?}", e),
        }
    };
    
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
    
    // Проверяем доступ к глобальным объектам игры
    if globals::is_game_initialized() {
        let _ = logger::info("Игра обнаружена, получаем экземпляр игры");
        
        match globals::get_game() {
            Ok(game) => {
                let _ = logger::info("Экземпляр игры получен успешно");
                
                if let Some(map_name) = game.get_map_name() {
                    let _ = logger::info(&format!("Текущая карта: {}", map_name));
                } else {
                    let _ = logger::warning("Не удалось получить имя карты");
                }
                
                if let Some(_resource_manager) = game.get_resource_manager() {
                    let _ = logger::info("Получен менеджер ресурсов");
                    
                    // TODO: Добавить путь к игре
                    // if let Some(game_path) = resource_manager.get_game_path() {
                    //     let _ = logger::info(&format!("Путь к игре: {}", game_path));
                    // } else {
                    //     let _ = logger::warning("Не удалось получить путь к игре");
                    // }
                    
                    // if let Some(steam_path) = resource_manager.get_steam_path() {
                    //     let _ = logger::info(&format!("Путь к Steam: {}", steam_path));
                    // }
                } else {
                    let _ = logger::warning("Не удалось получить менеджер ресурсов");
                }
                
                if let Some(physical_processor) = game.get_physical_processor() {
                    let _ = logger::info("Физический процессор получен");
                    
                    if let Some(timer_name) = physical_processor.get_timer_name() {
                        let _ = logger::info(&format!("Имя таймера: {}", timer_name));
                    }
                    
                    if let Some(locale) = physical_processor.get_locale() {
                        let _ = logger::info(&format!("Локаль игры: {}", locale));
                    }
                } else {
                    let _ = logger::warning("Не удалось получить физический процессор");
                }
            },
            Err(err) => {
                let _ = logger::error(&format!("Ошибка доступа к игре: {}", err));
            }
        }
    } else {
        let _ = logger::warning("Игра не инициализирована или не обнаружена");
    }
    
    let _ = logger::info("Клиент готов к работе");


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