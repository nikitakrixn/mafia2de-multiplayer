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

    if let Err(e) = run() {
        logger::error(&format!("Fatal: {e}"));
        show_error(&format!("{e}"));
        std::process::exit(1);
    }

    logger::info("Launcher finished. Have fun!");
}

fn run() -> Result<(), Error> {
    // 1. Проверяем, не запущена ли уже игра
    if inject::is_process_running(GAME_EXE_NAME)? {
        return Err(Error::GameAlreadyRunning);
    }
    logger::info("[ok] Game is not running");

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

    // 4. Находим и проверяем DLL
    let dll_path = find_client_dll()?;
    logger::info(&format!("[ok] DLL: {}", dll_path.display()));
    validate_dll(&dll_path)?;
    logger::info("[ok] DLL validated");

    // 5. Запускаем игру и инжектим
    let pid = inject::launch_and_inject(&game_path, &dll_path)?;
    logger::info(&format!("[ok] Game launched with mod (PID: {pid})"));

    Ok(())
}

fn find_client_dll() -> Result<std::path::PathBuf, Error> {
    let exe_dir = std::env::current_exe()
        .map_err(|_| Error::DllNotFound(CLIENT_DLL.into()))?;
    let dll = exe_dir.parent().unwrap().join(CLIENT_DLL);

    if !dll.exists() {
        return Err(Error::DllNotFound(dll.display().to_string()));
    }

    Ok(dll)
}

fn validate_dll(path: &Path) -> Result<(), Error> {
    let meta = std::fs::metadata(path)
        .map_err(|_| Error::DllNotFound(path.display().to_string()))?;

    if meta.len() < 1024 {
        return Err(Error::DllInvalid(format!(
            "File too small ({} bytes)",
            meta.len()
        )));
    }

    let mut f = std::fs::File::open(path)
        .map_err(|_| Error::DllNotFound(path.display().to_string()))?;
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