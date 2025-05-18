use mafia2de_sdk::{
    globals, CGame, CResourceManager, CPhysicalProcessor, CFreeRaid, CTranslocator,
    Vector3, Vector2, Quaternion, Matrix, Transform,
    object_list::object_list,
};
use common::logger::{self, LogLevel, LogTarget};
use std::path::Path;

fn main() {
    // Инициализируем логгер
    if let Err(err) = logger::Logger::init(
        LogLevel::Debug,
        LogTarget::Both,
        Some("logs/sdk_example.log".to_string()),
    ) {
        eprintln!("Не удалось инициализировать логгер: {}", err);
        return;
    }
    
    logger::info("SDK для Mafia II DE - Базовый пример");
    
    // Проверяем, инициализирована ли игра
    if !globals::is_game_initialized() {
        logger::warning("Игра не инициализирована");
        return;
    }
    
    // Получаем основной экземпляр игры
    let game = match globals::get_game() {
        Ok(game_instance) => game_instance,
        Err(err) => {
            logger::error(&format!("Не удалось получить экземпляр игры: {}", err));
            return;
        }
    };
    
    // Получаем и выводим информацию об игре
    logger::info(&format!("Игра запущена: {}", game.is_running()));
    
    if let Some(map_name) = game.get_map_name() {
        logger::info(&format!("Текущая карта: {}", map_name));
    }
    
    logger::info(&format!("Дельта времени: {:.6}", game.get_time_delta()));
    
    // Получаем менеджер ресурсов
    if let Some(resources) = game.get_resource_manager() {
        logger::info("Менеджер ресурсов получен");
        
        if let Some(game_path) = resources.get_game_path() {
            logger::info(&format!("Путь к игре: {}", game_path));
        }
        
        if let Some(steam_path) = resources.get_steam_path() {
            logger::info(&format!("Путь к Steam: {}", steam_path));
        }
    }
    
    // Получаем физический процессор
    if let Some(physics) = game.get_physical_processor() {
        logger::info("Физический процессор получен");
        
        // Обновляем физику
        physics.update_physics();
        
        if let Some(timer_name) = physics.get_timer_name() {
            logger::info(&format!("Имя таймера: {}", timer_name));
        }
        
        // Пробуем найти транслокатор по имени
        if let Some(translocator) = physics.find_translocator_by_name("player_start") {
            logger::info("Транслокатор игрока найден");
            
            // Получаем позицию игрока
            let pos = &translocator.position;
            logger::info(&format!("Позиция игрока: x={:.2}, y={:.2}, z={:.2}", pos.x, pos.y, pos.z));
            
            // Изменяем позицию транслокатора (для демонстрации)
            translocator.set_position(pos.x + 1.0, pos.y, pos.z);
            logger::info("Позиция транслокатора изменена");
        } else {
            logger::warning("Транслокатор игрока не найден");
        }
        
        // Попробуем преобразовать физический процессор в FreeRaid режим
        unsafe {
            if let Some(free_raid) = CFreeRaid::from_physical_processor(physics) {
                logger::info("Переключение в режим FreeRaid");
                free_raid.set_game_mode(1); // Установим режим свободной игры
            }
        }
    }
    
    // Получаем список объектов
    let obj_count = object_list::get_object_count();
    logger::info(&format!("Количество объектов: {}", obj_count));
    
    // Перебираем первые 10 объектов
    let max_display = 10.min(obj_count as usize);
    for i in 0..max_display {
        if let Some(obj_ptr) = object_list::get_object(i) {
            logger::info(&format!("Объект #{}: {:p}", i, obj_ptr));
        }
    }
    
    // Демонстрация использования типов данных SDK
    let position = Vector3::new(100.0, 200.0, 300.0);
    let rotation = Quaternion::from_euler(0.0, 1.57, 0.0); // поворот на 90 градусов вокруг Y
    let transform = Transform::new();
    
    logger::info(&format!("Пример преобразования: позиция={:?}, поворот={:?}", 
                         transform.position, transform.rotation));
    
    logger::info("Пример завершен");
} 