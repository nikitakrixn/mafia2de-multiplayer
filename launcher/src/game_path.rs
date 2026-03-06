//! Поиск пути к игре: реестр -> Steam -> диалог выбора файла.

use std::ffi::CString;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::*;
use windows::Win32::System::Registry::*;
use windows::Win32::UI::Controls::Dialogs::*;
use windows::core::{PCSTR, PSTR};

use common::logger;
use crate::error::Error;

const SUB_KEY: &str = "Software\\Mafia2-Multiplayer";
const SUB_KEY_VALUE: &str = "mafia2depath";
const GAME_NAME: &str = "Mafia II Definitive Edition";
const GAME_EXE: &str = "Mafia II Definitive Edition.exe";
const REGISTRY_BUF: usize = 1024;

/// Находит путь к исполняемому файлу игры.
pub fn find() -> Result<PathBuf, Error> {
    // 1. Реестр
    if let Ok(path_str) = registry_read(HKEY_CURRENT_USER, SUB_KEY, SUB_KEY_VALUE) {
        let path = PathBuf::from(&path_str);
        if !path_str.is_empty() && path.exists() {
            logger::info("Game path loaded from registry");
            return Ok(path);
        }
    }

    // 2. Steam
    if let Some(steam_dir) = find_steam_dir() {
        let game_path = steam_dir
            .join("steamapps")
            .join("common")
            .join(GAME_NAME)
            .join("pc")
            .join(GAME_EXE);

        if game_path.exists() {
            logger::info("Game found in Steam directory");
            save_to_registry(&game_path);
            return Ok(game_path);
        }
    }

    // 3. Диалог
    logger::info("Requesting game path from user...");
    let selected = show_file_dialog()?;
    save_to_registry(&selected);
    Ok(selected)
}

/// Проверяет доступность Steam. Логирует предупреждение если не найден.
///
/// Не возвращает ошибку — игра может быть установлена без Steam.
pub fn ensure_steam() -> Result<(), Error> {
    match find_steam_dir() {
        Some(dir) => {
            logger::info(&format!("Steam found: {}", dir.display()));
            Ok(())
        }
        None => {
            logger::warn("Steam directory not found in registry");
            logger::warn("Game path will be requested manually if needed");
            Ok(())
        }
    }
}

fn find_steam_dir() -> Option<PathBuf> {
    let path_str = registry_read(
        HKEY_CURRENT_USER,
        "Software\\Valve\\Steam",
        "SteamPath",
    )
    .ok()?;

    if path_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(path_str))
    }
}

fn registry_read(hkey: HKEY, subkey: &str, value: &str) -> Result<String, Error> {
    let subkey_c = to_cstring(subkey)?;
    let value_c = to_cstring(value)?;

    unsafe {
        let mut handle = HKEY::default();

        let status = RegOpenKeyExA(
            hkey,
            PCSTR(subkey_c.as_ptr() as *const u8),
            Some(0),
            KEY_READ,
            &mut handle,
        );

        if status != ERROR_SUCCESS {
            return Err(Error::Registry(format!("Cannot open key: {subkey}")));
        }

        let mut buf = vec![0u8; REGISTRY_BUF];
        let mut buf_size = buf.len() as u32;
        let mut typ = REG_SZ;

        let read_status = RegQueryValueExA(
            handle,
            PCSTR(value_c.as_ptr() as *const u8),
            None,
            Some(&mut typ),
            Some(buf.as_mut_ptr()),
            Some(&mut buf_size),
        );

        let _ = RegCloseKey(handle);

        if read_status != ERROR_SUCCESS {
            return Err(Error::Registry(format!("Cannot read value: {value}")));
        }

        let len = buf_size.saturating_sub(1) as usize;
        Ok(String::from_utf8_lossy(&buf[..len]).into_owned())
    }
}

fn registry_write(hkey: HKEY, subkey: &str, name: &str, value: &str) -> Result<(), Error> {
    let subkey_c = to_cstring(subkey)?;
    let name_c = to_cstring(name)?;
    let value_c = to_cstring(value)?;

    unsafe {
        let mut handle = HKEY::default();

        let status = RegOpenKeyExA(
            hkey,
            PCSTR(subkey_c.as_ptr() as *const u8),
            Some(0),
            KEY_WRITE,
            &mut handle,
        );

        if status != ERROR_SUCCESS {
            let create = RegCreateKeyA(
                hkey,
                PCSTR(subkey_c.as_ptr() as *const u8),
                &mut handle,
            );
            if create != ERROR_SUCCESS {
                return Err(Error::Registry(format!("Cannot create key: {subkey}")));
            }
        }

        let write = RegSetValueExA(
            handle,
            PCSTR(name_c.as_ptr() as *const u8),
            Some(0),
            REG_SZ,
            Some(value_c.as_bytes_with_nul()),
        );

        let _ = RegCloseKey(handle);

        if write != ERROR_SUCCESS {
            return Err(Error::Registry("Failed to write value".into()));
        }
    }

    Ok(())
}

fn save_to_registry(path: &Path) {
    if let Some(s) = path.to_str()
        && let Err(e) = registry_write(HKEY_CURRENT_USER, SUB_KEY, SUB_KEY_VALUE, s) {
            logger::warn(&format!("Failed to save path to registry: {e}"));
        }
}

fn show_file_dialog() -> Result<PathBuf, Error> {
    unsafe {
        let mut buf = vec![0u8; 260];
        let title = format!("Select {GAME_EXE}\0");

        #[allow(clippy::manual_c_str_literals)]
        let mut ofn = OPENFILENAMEA {
            lStructSize: std::mem::size_of::<OPENFILENAMEA>() as u32,
            lpstrFilter: PCSTR(b"Executable (*.exe)\0*.exe\0All Files\0*.*\0\0".as_ptr()),
            nFilterIndex: 1,
            lpstrFile: PSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            lpstrTitle: PCSTR(title.as_ptr()),
            Flags: OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR,
            ..Default::default()
        };

        if GetOpenFileNameA(&mut ofn).as_bool() {
            let end = buf.iter().position(|&b| b == 0).unwrap_or(0);
            if end > 0 {
                let s = String::from_utf8_lossy(&buf[..end]).into_owned();
                return Ok(PathBuf::from(s));
            }
        }

        Err(Error::GamePathNotFound)
    }
}

fn to_cstring(s: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|_| Error::Registry(format!("Invalid string: {s}")))
}