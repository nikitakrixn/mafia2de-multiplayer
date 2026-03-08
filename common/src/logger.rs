//! Потокобезопасный логгер с цветным выводом в консоль и записью в файл.
//!
//! ```ignore
//! use common::logger::{self, Level, Target};
//!
//! logger::init(Level::Debug, Target::Both, Some("logs/app.log")).unwrap();
//! logger::info("Запуск");
//! logger::error("Ошибка");
//! ```

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{LazyLock, Mutex};

use chrono::Local;

mod ansi {
    pub const RESET:  &str = "\x1b[0m";
    pub const DIM:    &str = "\x1b[2m";
    pub const GREEN:  &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED:    &str = "\x1b[1;31m";
    pub const CYAN:   &str = "\x1b[36m";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    /// Метка фиксированной ширины чтобы столбцы не прыгали
    fn tag(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info  => "INFO ",
            Self::Warn  => "WARN ",
            Self::Error => "ERROR",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Self::Debug => ansi::DIM,
            Self::Info  => ansi::GREEN,
            Self::Warn  => ansi::YELLOW,
            Self::Error => ansi::RED,
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.tag())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Console,
    File,
    Both,
}

struct Inner {
    level: Level,
    target: Target,
    file: Option<File>,
}

static LOGGER: LazyLock<Mutex<Option<Inner>>> = LazyLock::new(|| Mutex::new(None));

#[cfg(windows)]
fn enable_ansi_colors() {
    const STDOUT: u32 = -11_i32 as u32;
    const STDERR: u32 = -12_i32 as u32;
    const ENABLE_VTP: u32 = 0x0004;

    unsafe extern "system" {
        fn GetStdHandle(id: u32) -> isize;
        fn GetConsoleMode(h: isize, mode: *mut u32) -> i32;
        fn SetConsoleMode(h: isize, mode: u32) -> i32;
    }

    for id in [STDOUT, STDERR] {
        unsafe {
            let h = GetStdHandle(id);
            if h <= 0 { continue; }
            let mut mode: u32 = 0;
            if GetConsoleMode(h, &mut mode) != 0 {
                let _ = SetConsoleMode(h, mode | ENABLE_VTP);
            }
        }
    }
}

#[cfg(not(windows))]
fn enable_ansi_colors() {}

/// Инициализация глобального логгера. Вызывать один раз при старте.
/// Если target включает файл — file_path обязателен.
/// К имени файла добавляется timestamp чтобы логи не перезатирались.
pub fn init(level: Level, target: Target, file_path: Option<&str>) -> io::Result<()> {
    enable_ansi_colors();

    let file = if matches!(target, Target::File | Target::Both) {
        let raw = file_path.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path required when target is File or Both",
            )
        })?;

        let path = Path::new(raw);

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("log");
        let ext  = path.extension().and_then(|s| s.to_str()).unwrap_or("log");
        let ts   = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let name = format!("{stem}_{ts}.{ext}");
        let full = path.parent().unwrap_or(Path::new("")).join(name);

        let f = File::create(&full)?;
        println!("[logger] log file: {}", full.display());
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

fn log(level: Level, msg: &str) {
    let Ok(mut guard) = LOGGER.lock() else { return };
    let Some(inner) = guard.as_mut() else { return };

    if level < inner.level {
        return;
    }

    let ts = Local::now().format("%H:%M:%S%.3f").to_string();

    // В файл пишем чистый текст без ANSI-кодов
    if matches!(inner.target, Target::File | Target::Both)
        && let Some(f) = &mut inner.file
    {
        let plain = format!("[{ts}] {level} | {msg}\n");
        let _ = f.write_all(plain.as_bytes());
        let _ = f.flush();
    }

    if matches!(inner.target, Target::Console | Target::Both) {
        let line = colorize(level, &ts, msg);
        if level >= Level::Error {
            eprint!("{line}");
        } else {
            print!("{line}");
        }
    }
}

/// DEBUG — целиком dim, не отвлекает.
/// INFO  — зелёная метка, обычный текст.
/// WARN  — жёлтая метка + жёлтый текст.
/// ERROR — красный bold целиком.
fn colorize(level: Level, ts: &str, msg: &str) -> String {
    let c = level.color();
    let r = ansi::RESET;
    let d = ansi::DIM;
    let p = ansi::CYAN;

    match level {
        Level::Debug => format!("{d}[{ts}] {level} | {msg}{r}\n"),
        Level::Info  => format!("{d}[{ts}]{r} {c}{level}{r} {p}|{r} {msg}\n"),
        _            => format!("{d}[{ts}]{r} {c}{level}{r} {p}|{r} {c}{msg}{r}\n"),
    }
}

pub fn debug(msg: &str) { log(Level::Debug, msg); }
pub fn info(msg: &str)  { log(Level::Info,  msg); }
pub fn warn(msg: &str)  { log(Level::Warn,  msg); }
pub fn error(msg: &str) { log(Level::Error, msg); }