use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use chrono::Local;

/// Уровни логирования
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
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

        let mut global_logger = LOGGER.lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock logger"))?;
        *global_logger = Some(logger);

        Ok(())
    }

    /// Записывает сообщение в лог
    pub fn log(&mut self, level: LogLevel, message: &str) -> io::Result<()> {
        if level < self.level {
            return Ok(());
        }

        // Строковое представление уровня логирования
        let level_str = match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        };

        // Получаем текущее время в формате ISO 8601
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        
        let log_message = format!("[{}] {} {}\n", timestamp, level_str, message);

        match self.target {
            LogTarget::Console => {
                let colored_msg = Self::colorize(level, &log_message);
                if level >= LogLevel::Error {
                    eprint!("{}", colored_msg);
                } else {
                    print!("{}", colored_msg);
                }
            }
            LogTarget::File => {
                if let Some(file) = &mut self.file {
                    file.write_all(log_message.as_bytes())?;
                    file.flush()?;
                }
            }
            LogTarget::Both => {
                let colored_msg = Self::colorize(level, &log_message);
                if level >= LogLevel::Error {
                    eprint!("{}", colored_msg);
                } else {
                    print!("{}", colored_msg);
                }
                if let Some(file) = &mut self.file {
                    file.write_all(log_message.as_bytes())?;
                    file.flush()?;
                }
            }
        }

        Ok(())
    }

    /// Функция для раскрашивания сообщения, добавляя ANSI escape-последовательности
    fn colorize(level: LogLevel, message: &str) -> String {
        const RESET: &str = "\x1b[0m";
        match level {
            LogLevel::Trace => format!("{}", message),
            LogLevel::Debug => format!("\x1b[34m{}{}", message, RESET),
            LogLevel::Info => format!("\x1b[32m{}{}", message, RESET),
            LogLevel::Warning => format!("\x1b[33m{}{}", message, RESET),
            LogLevel::Error => format!("\x1b[31m{}{}", message, RESET),
            LogLevel::Critical => format!("\x1b[1;31m{}{}", message, RESET),
        }
    }
}

/// Получает глобальный экземпляр логгера
fn get_logger() -> io::Result<MutexGuard<'static, Option<Logger>>> {
    LOGGER.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock logger"))
}

/// Публичные функции для логирования

pub fn trace(message: &str) -> io::Result<()> {
    let mut guard = get_logger()?;
    if let Some(logger) = guard.as_mut() {
        logger.log(LogLevel::Trace, message)?;
    }
    Ok(())
}

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

pub fn critical(message: &str) -> io::Result<()> {
    let mut guard = get_logger()?;
    if let Some(logger) = guard.as_mut() {
        logger.log(LogLevel::Critical, message)?;
    }
    Ok(())
}