//! Потокобезопасный логгер с выводом в консоль и/или файл.
//!
//! # Использование
//! ```ignore
//! use common::logger::{self, Level, Target};
//!
//! logger::init(Level::Debug, Target::Both, Some("logs/app.log")).unwrap();
//! logger::info("Запуск приложения");
//! logger::error("Что-то пошло не так");
//! ```

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{LazyLock, Mutex};

use chrono::Local;

// ── Типы ────────────────────────────────────────────────────────────────────

/// Уровень логирования.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            Self::Debug => "DEBUG",
            Self::Info  => "INFO ",
            Self::Warn  => "WARN ",
            Self::Error => "ERROR",
        };
        f.write_str(tag)
    }
}

/// Куда писать логи.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Console,
    File,
    Both,
}

// ── Внутреннее состояние ────────────────────────────────────────────────────

struct Inner {
    level: Level,
    target: Target,
    file: Option<File>,
}

static LOGGER: LazyLock<Mutex<Option<Inner>>> = LazyLock::new(|| Mutex::new(None));

// ── Публичный API ───────────────────────────────────────────────────────────

/// Инициализирует глобальный логгер.
///
/// Если `target` — `File` или `Both`, `file_path` обязателен.
/// К имени файла автоматически добавляется временная метка.
pub fn init(level: Level, target: Target, file_path: Option<&str>) -> io::Result<()> {
    let file = if matches!(target, Target::File | Target::Both) {
        let raw = file_path.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path required when target is File or Both",
            )
        })?;

        let path = Path::new(raw);

        // Создаём директорию
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }

        // Добавляем временную метку к имени файла
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("log");
        let ext  = path.extension().and_then(|s| s.to_str()).unwrap_or("log");
        let ts   = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let name = format!("{stem}_{ts}.{ext}");
        let full = path.parent().unwrap_or(Path::new("")).join(name);

        let f = File::create(&full)?;
        println!("[logger] Writing to {}", full.display());
        Some(f)
    } else {
        None
    };

    let mut guard = LOGGER
        .lock()
        .map_err(|_| io::Error::other("Logger mutex poisoned"))?;

    *guard = Some(Inner { level, target, file });
    Ok(())
}

/// Записывает сообщение с указанным уровнем.
fn log(level: Level, msg: &str) {
    let Ok(mut guard) = LOGGER.lock() else { return };
    let Some(inner) = guard.as_mut() else { return };

    if level < inner.level {
        return;
    }

    let ts   = Local::now().format("%H:%M:%S%.3f");
    let line = format!("[{ts}] {level} | {msg}\n");

    // Консоль
    if matches!(inner.target, Target::Console | Target::Both) {
        if level >= Level::Error {
            eprint!("{line}");
        } else {
            print!("{line}");
        }
    }

    // Файл
    if matches!(inner.target, Target::File | Target::Both)
        && let Some(f) = &mut inner.file {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
}

// Удобные функции
pub fn debug(msg: &str) { log(Level::Debug, msg); }
pub fn info(msg: &str)  { log(Level::Info,  msg); }
pub fn warn(msg: &str)  { log(Level::Warn,  msg); }
pub fn error(msg: &str) { log(Level::Error, msg); }