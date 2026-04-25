mod error;
mod game_path;
mod inject;

use std::io::Read;
use std::path::Path;

use common::logger;
use error::Error;

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCSTR;

const GAME_EXE_NAME: &str = "Mafia II Definitive Edition.exe";
const CLIENT_DLL: &str = "m2mp_client.dll";
const DEVTOOLS_DLL: &str = "m2mp_devtools.dll";

#[derive(Debug, Clone, Copy)]
enum LaunchMode {
    Client,
    Devtools,
}

#[derive(Debug, Default, Clone, Copy)]
struct LaunchOptions {
    allow_second: bool,
}

fn main() {
    if let Err(e) = logger::init(
        logger::Level::Debug,
        logger::Target::Both,
        Some("logs/launcher.log"),
    ) {
        eprintln!("Logger init failed: {e}");
    }

    logger::info("========================================");
    logger::info("  Mafia II: DE Multiplayer Launcher");
    logger::info("  Version 0.1.0 | x86_64");
    logger::info("========================================");

    let (mode, opts) = parse_args();
    logger::info(&format!(
        "Launch mode: {mode:?}  allow_second: {}",
        opts.allow_second
    ));

    if let Err(e) = run(mode, opts) {
        logger::error(&format!("Fatal: {e}"));
        show_error(&format!("{e}"));
        std::process::exit(1);
    }

    logger::info("Launcher finished. Have fun!");
}

fn parse_args() -> (LaunchMode, LaunchOptions) {
    let mut mode = LaunchMode::Client;
    let mut opts = LaunchOptions::default();

    for arg in std::env::args().skip(1) {
        match arg.to_lowercase().as_str() {
            "--devtools" | "-d" | "devtools" => mode = LaunchMode::Devtools,
            "--client" | "-c" | "client" => mode = LaunchMode::Client,
            "--allow-second" | "--second" => opts.allow_second = true,
            _ => {}
        }
    }

    (mode, opts)
}

fn run(mode: LaunchMode, opts: LaunchOptions) -> Result<(), Error> {
    // 1. Проверяем, не запущена ли уже игра.
    if opts.allow_second {
        logger::info("[skip] is_process_running check (--allow-second)");
    } else if inject::is_process_running(GAME_EXE_NAME)? {
        return Err(Error::GameAlreadyRunning);
    } else {
        logger::info("[ok] Game is not running");
    }

    // 2. Проверяем Steam (процесс + директория)
    if !inject::is_process_running("steam.exe")? {
        return Err(Error::SteamNotRunning);
    }
    logger::info("[ok] Steam process is running");

    game_path::ensure_steam()?;
    logger::info("[ok] Steam check passed");

    // 3. Находим путь к игре
    let game_path = game_path::find()?;
    logger::info(&format!("[ok] Game: {}", game_path.display()));

    if !game_path.exists() {
        return Err(Error::GamePathNotFound);
    }

    // 4. Находим и проверяем DLL в зависимости от режима
    let dll_path = find_dll(mode)?;
    logger::info(&format!("[ok] DLL: {}", dll_path.display()));
    validate_dll(&dll_path)?;
    logger::info("[ok] DLL validated");

    // 5. Решаем working_directory для CreateProcess.
    //
    // Если рядом с launcher лежит `steam_api64.dll` (положен xtask::dist
    // или вручную в binary/) — используем launcher folder как working_dir.
    //
    // Иначе fallback на стандартное поведение — working_dir = папка игры.
    let working_dir = pick_working_dir(&game_path);
    logger::info(&format!("[ok] working_dir = {}", working_dir.display()));

    // 6. Запускаем игру и инжектим
    let pid = inject::launch_and_inject(&game_path, &dll_path, &working_dir)?;
    logger::info(&format!("[ok] Game launched with mod (PID: {pid})"));

    Ok(())
}

/// Выбирает working directory для CreateProcess.
///
/// Если рядом с launcher лежит `steam_api64.dll` — используем launcher folder.
/// Иначе — папка игры как и раньше.
fn pick_working_dir(game_path: &Path) -> std::path::PathBuf {
    let launcher_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));

    if let Some(dir) = launcher_dir
        && dir.join("steam_api64.dll").exists()
    {
        logger::info("[hijack] local steam_api64.dll detected — using launcher dir as CWD");
        return dir;
    }

    game_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn find_dll(mode: LaunchMode) -> Result<std::path::PathBuf, Error> {
    let dll_name = match mode {
        LaunchMode::Client => CLIENT_DLL,
        LaunchMode::Devtools => DEVTOOLS_DLL,
    };

    let exe_dir = std::env::current_exe().map_err(|_| Error::DllNotFound(dll_name.into()))?;
    let dll = exe_dir.parent().unwrap().join(dll_name);

    if !dll.exists() {
        return Err(Error::DllNotFound(dll.display().to_string()));
    }

    Ok(dll)
}

fn validate_dll(path: &Path) -> Result<(), Error> {
    let meta =
        std::fs::metadata(path).map_err(|_| Error::DllNotFound(path.display().to_string()))?;

    if meta.len() < 1024 {
        return Err(Error::DllInvalid(format!(
            "File too small ({} bytes)",
            meta.len()
        )));
    }

    let mut f =
        std::fs::File::open(path).map_err(|_| Error::DllNotFound(path.display().to_string()))?;
    let mut header = [0u8; 2];
    f.read_exact(&mut header)
        .map_err(|_| Error::DllInvalid("Cannot read header".into()))?;

    if &header != b"MZ" {
        return Err(Error::DllInvalid("Not a valid PE file".into()));
    }

    Ok(())
}

fn show_error(message: &str) {
    let title = b"Mafia II: Multiplayer - Error\0";
    let mut msg_buf: Vec<u8> = message.as_bytes().to_vec();
    msg_buf.push(0);

    unsafe {
        let _ = MessageBoxA(
            None,
            PCSTR(msg_buf.as_ptr()),
            PCSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}
