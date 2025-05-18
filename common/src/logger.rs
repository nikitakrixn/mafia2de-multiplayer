use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use chrono::Local;

/// Уровни логирования
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

/// Цели логирования
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTarget {
    Console,
    File,
    Both,
}

/// Структура для управления логированием
pub struct Logger {
    level: LogLevel,
    target: LogTarget,
    file: Option<File>,
}

// Глобальный экземпляр логгера
lazy_static::lazy_static! {
    static ref LOGGER: Arc<Mutex<Option<Logger>>> = Arc::new(Mutex::new(None));
}

impl Logger {
    /// Инициализирует логгер
    pub fn init(level: LogLevel, target: LogTarget, file_path: Option<String>) -> io::Result<()> {
        let mut logger = Self {
            level,
            target,
            file: None,
        };

        if matches!(target, LogTarget::File | LogTarget::Both) {
            if let Some(path) = file_path {
                // Добавляем временную метку к имени файла
                let now = Local::now();
                let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                
                // Создаем директорию для логов, если ее нет
                let path_obj = Path::new(&path);
                if let Some(parent) = path_obj.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                
                // Получаем базовое имя файла и расширение
                let file_stem = path_obj.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("log");
                let extension = path_obj.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("log");
                
                // Конструируем имя файла с временной меткой
                let parent_path = path_obj.parent().unwrap_or_else(|| Path::new(""));
                let filename = format!("{}_{}.{}", file_stem, timestamp, extension);
                let timestamped_path = parent_path.join(filename);
                
                // Создаем новый файл
                logger.file = Some(File::create(&timestamped_path)?);
                
                println!("Логирование в файл: {}", timestamped_path.display());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path to log file is required when target is File or Both",
                ));
            }
        }

        let mut global_logger = LOGGER.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock logger"))?;
        *global_logger = Some(logger);

        Ok(())
    }

    /// Записывает сообщение в лог
    pub fn log(&mut self, level: LogLevel, message: &str) -> io::Result<()> {
        if level < self.level {
            return Ok(());
        }

        let level_str = match level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        };

        // Получаем текущее время в формате ISO 8601
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        
        let log_message = format!("[{}] {} {}\n", timestamp, level_str, message);

        match self.target {
            LogTarget::Console => {
                if level >= LogLevel::Error {
                    eprint!("{}", log_message);
                } else {
                    print!("{}", log_message);
                }
            }
            LogTarget::File => {
                if let Some(file) = &mut self.file {
                    file.write_all(log_message.as_bytes())?;
                    file.flush()?;
                }
            }
            LogTarget::Both => {
                if level >= LogLevel::Error {
                    eprint!("{}", log_message);
                } else {
                    print!("{}", log_message);
                }
                if let Some(file) = &mut self.file {
                    file.write_all(log_message.as_bytes())?;
                    file.flush()?;
                }
            }
        }

        Ok(())
    }
}

/// Получает глобальный экземпляр логгера
fn get_logger() -> io::Result<MutexGuard<'static, Option<Logger>>> {
    LOGGER
        .lock()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock logger"))
}

/// Публичные функции для логирования

pub fn debug(message: &str) -> io::Result<()> {
    let mut guard = get_logger()?;
    if let Some(logger) = guard.as_mut() {
        logger.log(LogLevel::Debug, message)?;
    }
    Ok(())
}

pub fn info(message: &str) -> io::Result<()> {
    let mut guard = get_logger()?;
    if let Some(logger) = guard.as_mut() {
        logger.log(LogLevel::Info, message)?;
    }
    Ok(())
}

pub fn warning(message: &str) -> io::Result<()> {
    let mut guard = get_logger()?;
    if let Some(logger) = guard.as_mut() {
        logger.log(LogLevel::Warning, message)?;
    }
    Ok(())
}

pub fn error(message: &str) -> io::Result<()> {
    let mut guard = get_logger()?;
    if let Some(logger) = guard.as_mut() {
        logger.log(LogLevel::Error, message)?;
    }
    Ok(())
} 