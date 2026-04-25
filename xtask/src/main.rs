//! xtask — служебные задачи сборки / пакетирования.
//!
//! Не зависит от game-кода и SDK. Запускается как `cargo xtask <subcommand>`
//! либо через alias из `.cargo/config.toml`.
//!
//! ## Команды
//!
//! - `dist` — собрать все компоненты в `release` и сложить в `binary/`
//!   рядом с `steam_api64.dll`, `steam_appid.txt`, .bat-файлами для запуска
//!   одиночного клиента и devtools
//! - `clean-binary` — удалить `binary/`.
//!
//! ## Структура `binary/`
//!
//! После `cargo dist`:
//!
//! ```text
//! binary/
//!   launcher.exe            — наш launcher (release)
//!   m2mp_client.dll         — основной mod-DLL (release)
//!   m2mp_devtools.dll       — devtools mod-DLL (release)
//!   server.exe              — сервер (release)
//!   steam_api64.dll         — копия из assets/ (нужно положить вручную)
//!   steam_appid.txt         — `1030830` (Mafia II Definitive Edition)
//!   client.bat              — обычный запуск клиента
//!   devtools.bat            — запуск с devtools-модом
//!   client-pair.bat         — запустить два клиента параллельно
//!   server.bat              — запуск локального сервера
//! ```

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const STEAM_APP_ID: &str = "1030830";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".into());

    match cmd.as_str() {
        "dist" => match dist() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("[xtask] dist failed: {e}");
                ExitCode::FAILURE
            }
        },
        "clean-binary" => match clean_binary() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("[xtask] clean-binary failed: {e}");
                ExitCode::FAILURE
            }
        },
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("[xtask] unknown command: {other}");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("xtask — служебные задачи сборки\n");
    println!("Использование:");
    println!("  cargo xtask dist          собрать всё в release и упаковать в binary/");
    println!("  cargo xtask clean-binary  удалить папку binary/");
    println!("  cargo xtask help          показать эту справку");
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live inside workspace")
        .to_path_buf()
}

fn binary_dir() -> PathBuf {
    workspace_root().join("binary")
}

fn assets_dir() -> PathBuf {
    workspace_root().join("assets")
}

fn target_release_dir() -> PathBuf {
    workspace_root()
        .join("target")
        .join("x86_64-pc-windows-msvc")
        .join("release")
}

fn dist() -> Result<(), String> {
    println!("[xtask] === build all release artefacts ===");

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(&cargo)
        .args([
            "build",
            "--release",
            "-p", "launcher",
            "-p", "client",
            "-p", "devtools",
            "-p", "server",
        ])
        .current_dir(workspace_root())
        .status()
        .map_err(|e| format!("failed to spawn cargo: {e}"))?;

    if !status.success() {
        return Err(format!("cargo build failed (status={status})"));
    }

    println!("[xtask] === prepare binary/ ===");
    let bin = binary_dir();
    std::fs::create_dir_all(&bin)
        .map_err(|e| format!("create_dir_all({}): {e}", bin.display()))?;

    let release = target_release_dir();

    let copies = [
        ("launcher.exe",      "launcher.exe",      true),
        ("m2mp_client.dll",   "m2mp_client.dll",   true),
        ("m2mp_devtools.dll", "m2mp_devtools.dll", true),
        ("server.exe",        "server.exe",        true),
    ];

    for (src_name, dst_name, required) in copies {
        let src = release.join(src_name);
        let dst = bin.join(dst_name);
        match copy_atomic(&src, &dst) {
            Ok(()) => println!("[xtask]   copied {dst_name}"),
            Err(e) if required => return Err(e),
            Err(e) => println!("[xtask]   skipped {dst_name}: {e}"),
        }
    }

    copy_assets(&bin)?;
    write_steam_appid(&bin)?;
    write_bat_files(&bin)?;

    println!("[xtask] === done. binary/ is ready ===");
    println!();
    println!("Запуск:");
    println!("  binary\\client.bat        — один клиент (mod release)");
    println!("  binary\\devtools.bat      — один клиент с devtools");
    println!("  binary\\server.bat        — локальный сервер");
    println!("  binary\\client-pair.bat   — два клиента (для теста MP)");
    println!();

    let has_steam_api = bin.join("steam_api64.dll").exists();
    if !has_steam_api {
        println!("⚠️  client-pair.bat не сработает без steam_api64.dll!");
        println!();
        println!("    Скопируй его рядом с launcher одной из команд:");
        println!();
        println!("    copy \"C:\\Program Files (x86)\\Steam\\steamapps\\common\\\\\\");
        println!("          Mafia II Definitive Edition\\pc\\steam_api64.dll\" assets\\");
        println!("    cargo dist");
        println!();
        println!("    либо вручную:");
        println!();
        println!("    copy \"C:\\Program Files (x86)\\Steam\\steamapps\\common\\\\\\");
        println!("          Mafia II Definitive Edition\\pc\\steam_api64.dll\" binary\\");
    } else {
        println!("✓ steam_api64.dll на месте — DLL hijacking trick активирован.");
        println!("  Launcher использует binary/ как working_dir, что даёт MP-pair.");
    }

    Ok(())
}

fn copy_assets(bin: &Path) -> Result<(), String> {
    let assets = assets_dir();
    if !assets.exists() {
        println!(
            "[xtask]   note: assets/ не найдена; пропускаю копирование steam_api64.dll и др."
        );
        println!(
            "[xtask]         Создай assets/ и положи туда steam_api64.dll (из папки игры),"
        );
        println!("[xtask]         чтобы он автоматически копировался в binary/.");
        return Ok(());
    }

    let known = ["steam_api64.dll", "pair-launcher.ahk", "pair-launcher.exe"];
    for name in known {
        let src = assets.join(name);
        if !src.exists() {
            continue;
        }
        let dst = bin.join(name);
        copy_atomic(&src, &dst)?;
        println!("[xtask]   copied asset {name}");
    }
    Ok(())
}

fn write_steam_appid(bin: &Path) -> Result<(), String> {
    let path = bin.join("steam_appid.txt");
    std::fs::write(&path, STEAM_APP_ID)
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    println!("[xtask]   wrote steam_appid.txt = {STEAM_APP_ID}");
    Ok(())
}

fn write_bat_files(bin: &Path) -> Result<(), String> {
    let client_bat = "@echo off\r\n\
        setlocal\r\n\
        cd /d \"%~dp0\"\r\n\
        launcher.exe --client\r\n";
    let devtools_bat = "@echo off\r\n\
        setlocal\r\n\
        cd /d \"%~dp0\"\r\n\
        launcher.exe --devtools\r\n";
    let pair_bat = "@echo off\r\n\
        rem Запускает 2 клиента для локального теста MP.\r\n\
        rem Если рядом есть pair-launcher.exe (скомпилированный AHK скрипт)\r\n\
        rem — используем его для расстановки окон по половинам экрана.\r\n\
        rem Иначе — простой последовательный запуск без раскладки.\r\n\
        setlocal\r\n\
        cd /d \"%~dp0\"\r\n\
        if exist pair-launcher.exe (\r\n\
            start \"\" pair-launcher.exe\r\n\
        ) else (\r\n\
            echo [pair] pair-launcher.exe не найден, fallback на простой запуск\r\n\
            echo [pair] чтобы получить окна по половинам экрана:\r\n\
            echo [pair]   1. установи AutoHotkey v1\r\n\
            echo [pair]   2. скомпилируй assets\\pair-launcher.ahk через Ahk2Exe\r\n\
            echo [pair]   3. положи pair-launcher.exe рядом с launcher.exe\r\n\
            start \"client-1\" launcher.exe --client --allow-second\r\n\
            timeout /t 3 /nobreak ^>nul\r\n\
            start \"client-2\" launcher.exe --client --allow-second\r\n\
        )\r\n";
    let server_bat = "@echo off\r\n\
        setlocal\r\n\
        cd /d \"%~dp0\"\r\n\
        server.exe\r\n\
        pause\r\n";

    write_text(bin.join("client.bat"), client_bat)?;
    write_text(bin.join("devtools.bat"), devtools_bat)?;
    write_text(bin.join("client-pair.bat"), pair_bat)?;
    write_text(bin.join("server.bat"), server_bat)?;
    println!("[xtask]   wrote client.bat / devtools.bat / client-pair.bat / server.bat");
    Ok(())
}

fn write_text(path: PathBuf, content: &str) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("write {}: {e}", path.display()))
}

fn copy_atomic(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Err(format!("source missing: {}", src.display()));
    }
    let tmp = dst.with_extension("tmp");
    std::fs::copy(src, &tmp).map_err(|e| format!("copy {} -> {}: {e}", src.display(), tmp.display()))?;
    if dst.exists() {
        std::fs::remove_file(dst).ok();
    }
    std::fs::rename(&tmp, dst)
        .map_err(|e| format!("rename {} -> {}: {e}", tmp.display(), dst.display()))?;
    Ok(())
}

fn clean_binary() -> Result<(), String> {
    let bin = binary_dir();
    if !bin.exists() {
        println!("[xtask] binary/ already absent");
        return Ok(());
    }
    std::fs::remove_dir_all(&bin)
        .map_err(|e| format!("remove_dir_all({}): {e}", bin.display()))?;
    println!("[xtask] removed {}", bin.display());
    Ok(())
}
