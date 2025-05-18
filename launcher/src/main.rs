use std::{ffi::CString, path::PathBuf};
use common::logger::{self, LogLevel, LogTarget};

use windows::{
    core::{PCSTR, PSTR}, Win32::{Foundation::*, System::{Diagnostics::Debug::WriteProcessMemory, LibraryLoader::{GetModuleHandleA, GetProcAddress}, Memory::*, Registry::*, Threading::*}, UI::Controls::Dialogs::*}
};


const SUB_KEY: &str = "Software\\Mafia2-Multiplayer";
const SUB_KEY_VALUE: &str = "mafia2depath";
const CLIENT_DLL_NAME: &str = "ebn_client.dll";
const GAME_NAME: &str = "Mafia II Definitive Edition";
const GAME_EXE_NAME: &str = "Mafia II Definitive Edition.exe";

fn main() -> windows::core::Result<()>  {
    // Инициализируем логгер
    if let Err(err) = logger::Logger::init(
        LogLevel::Debug, 
        LogTarget::Both, 
        Some("logs/launcher.log".to_string())
    ) {
        eprintln!("Не удалось инициализировать логгер: {}", err);
    }

    logger::info("Mafia II: Multiplayer Launcher starting...");
    
    // Получаем путь к игре
    let game_path = match get_game_path() {
        Ok(path) => {
            let msg = format!("Game path: {}", path.display());
            logger::info(&msg);
            path
        },
        Err(e) => {
            let msg = format!("Error finding game path: {:?}", e);
            logger::error(&msg);
            return Err(e);
        }
    };
    
    // Проверяем существование исполняемого файла игры
    if !game_path.exists() {
        let msg = format!("Game executable not found: {}", game_path.display());
        logger::error(&msg);
        return Err(windows::core::Error::from_win32());
    }
    
    // Получаем путь к рабочей директории игры
    let _game_dir = match game_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => {
            let msg = "Could not determine game directory";
            logger::error(msg);
            return Err(windows::core::Error::from_win32());
        }
    };
    
    // Получаем путь к DLL нашего клиента
    let dll_path = match std::env::current_exe() {
        Ok(exe_path) => {
            let dll_path = exe_path.parent().unwrap().join(CLIENT_DLL_NAME);
            let msg = format!("DLL Path: {}", dll_path.display());
            logger::info(&msg);
            dll_path
        },
        Err(e) => {
            let msg = format!("Failed to get current executable path: {:?}", e);
            logger::error(&msg);
            return Err(windows::core::Error::from_win32());
        }
    };
    
    // Проверяем существование DLL
    if !dll_path.exists() {
        let msg = format!("Client DLL not found: {}", dll_path.display());
        logger::error(&msg);
        return Err(windows::core::Error::from_win32());
    }
    
    // Запускаем игру и инжектим DLL
    match start_game_process(&game_path, &dll_path) {
        Ok(process_id) => {
            let msg = format!("Game started successfully with PID: {}", process_id);
            logger::info(&msg);
        },
        Err(e) => {
            let msg = format!("Failed to start game process: {:?}", e);
            logger::error(&msg);
            return Err(e);
        }
    }

    logger::info("Launcher completed successfully!");
    
    Ok(())
}

fn get_game_path() -> windows::core::Result<PathBuf> {
    match read_registry_string(HKEY_CURRENT_USER, SUB_KEY, SUB_KEY_VALUE) {
        Ok(path) if !path.is_empty() && PathBuf::from(&path).exists() => {
            println!("Mafia II DE path (from registry): {}", path);
            Ok(PathBuf::from(path))
        }
        _ => {
            // Попытка найти через Steam
            let steam_path = get_steam_path();
            match steam_path {
                Some(steam_dir) => {
                    let game_path = steam_dir.join("steamapps").join("common").join("Mafia II Definitive Edition").join("pc").join(GAME_EXE_NAME);
                    if game_path.exists() {
                        println!("Mafia II DE path (found in Steam): {}", game_path.display());
                        // Сохраняем путь в реестре
                        let _ = write_registry_string(
                            HKEY_CURRENT_USER,
                            SUB_KEY,
                            SUB_KEY_VALUE,
                            game_path.to_str().unwrap(),
                        );
                        return Ok(game_path);
                    }
                }
                None => println!("Steam path not found"),
            }

            // Если не найден в Steam - запрашиваем у пользователя
            let selected_path = folder_game_path(GAME_EXE_NAME)?;
            println!("Mafia II DE path (selected): {}", selected_path.display());
            // Сохраняем путь в реестре
            let _ = write_registry_string(
                HKEY_CURRENT_USER,
                SUB_KEY,
                SUB_KEY_VALUE,
                selected_path.to_str().unwrap(),
            );
            Ok(selected_path)
        }
    }
}

fn get_steam_path() -> Option<PathBuf> {
    unsafe {
        let mut handle = HKEY::default();
        let path = CString::new("Software\\Valve\\Steam").expect("CString::new failed");
        
        // Открываем ключ реестра
        let status = RegOpenKeyExA(
            HKEY_CURRENT_USER,
            PCSTR(path.to_bytes().as_ptr()),
            Some(0),
            KEY_READ,
            &mut handle,
        );
        
        if status != ERROR_SUCCESS {
            return None;
        }
        
        // Читаем значение SteamPath
        let mut buffer: [u8; 1024] = [0; 1024];
        let mut buffer_size: u32 = buffer.len() as u32;
        let mut typ = REG_SZ;
        let value_name = CString::new("SteamPath").expect("CString::new failed");
        
        let read_status = RegQueryValueExA(
            handle, 
            PCSTR(value_name.to_bytes().as_ptr()), 
            None, 
            Some(&mut typ), 
            Some(buffer.as_mut_ptr()),
            Some(&mut buffer_size),
        );
        
        // Закрываем ключ реестра
        let _ = RegCloseKey(handle);
        
        if read_status != ERROR_SUCCESS {
            return None;
        }
        
        // Преобразуем буфер в строку и возвращаем путь
        let steam_path = String::from_utf8_lossy(&buffer[..buffer_size as usize - 1]).to_string();
        Some(PathBuf::from(steam_path))
    }
}

fn folder_game_path (_game_exe_name: &str) -> Result<PathBuf, windows::core::Error> {
    unsafe
    {
        let mut open_file_dialog = OPENFILENAMEA::default();
        let mut file_name_buffer = vec![0u8; 260];
        open_file_dialog.lStructSize  = std::mem::size_of::<OPENFILENAMEA>() as u32;
        open_file_dialog.hwndOwner    = HWND::default();
        open_file_dialog.lpstrFilter  = PCSTR(b"Executable files\0*.exe\0All files\0*.*\0\0".as_ptr());
        open_file_dialog.nFilterIndex = 1;
        open_file_dialog.lpstrFile    = PSTR(file_name_buffer.as_mut_ptr());
        open_file_dialog.nMaxFile     = file_name_buffer.len() as u32;
        open_file_dialog.lpstrTitle   = PCSTR(format!("Select {}.exe", GAME_NAME).as_bytes().as_ptr());
        open_file_dialog.Flags        = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

        if GetOpenFileNameA(&mut open_file_dialog).into() {
            // Находим конец строки (нулевой байт)
            let mut end_pos = 0;
            while end_pos < file_name_buffer.len() && file_name_buffer[end_pos] != 0 {
                end_pos += 1;
            }
            
            if end_pos > 0 {
                let path_str = String::from_utf8_lossy(&file_name_buffer[0..end_pos]).to_string();
                Ok(PathBuf::from(path_str))
            } else {
                Err(windows::core::Error::from_win32())
            }
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

fn write_registry_string(
    hkeylocation: HKEY, 
    subkey: &str, 
    subkeyvalue: &str, 
    value: &str
) -> bool {
    unsafe {
        let mut handle = HKEY::default();
        let path = CString::new(subkey).expect("Failed to create CString");
        let sub_key_value = CString::new(subkeyvalue).expect("Failed to create CString");
        let value = CString::new(value).expect("Failed to create CString");
        let status = 
            RegOpenKeyExA(
                hkeylocation,
                PCSTR(path.to_bytes().as_ptr()), 
                Some(0), 
                KEY_WRITE, 
                &mut handle
            );

        if status != ERROR_SUCCESS {
            // Если ключ не существует, создаем его
            if status == ERROR_FILE_NOT_FOUND {
                let create_status = RegCreateKeyA(
                    hkeylocation,
                    PCSTR(path.as_bytes().as_ptr()),
                    &mut handle,
                );

                // Если создание не удалось, возвращаем false
                if create_status != ERROR_SUCCESS {
                    println!("Failed to create registry key: {:?}", create_status);
                    return false;
                }
            } else {
                println!("Failed to open registry key: {:?}", status);
                return false;
            }
        }

        // Записываем значение в реестр
        let write_status = RegSetValueExA(
            handle, 
            PCSTR(sub_key_value.to_bytes().as_ptr()), 
            Some(0), 
            REG_SZ, 
            Some(value.to_bytes_with_nul())
        );

        // Проверяем успешность записи
        if write_status != ERROR_SUCCESS {
            println!("Failed to write to registry: {:?}", write_status);
            return false;
        }

        // Закрываем дескриптор ключа реестра
        let _ = RegCloseKey(handle);
        true
    }
}

fn read_registry_string(
    hkeylocation: HKEY, 
    subkey: &str, 
    value_name: &str
) -> Result<String, windows::core::Error> {
    unsafe {
        let mut handle = HKEY::default();
        let path = CString::new(subkey).expect("CString::new failed");
        let value_name_cstr = CString::new(value_name).expect("CString::new failed");
        
        // Открываем ключ реестра для чтения
        let status = RegOpenKeyExA(
            hkeylocation,
            PCSTR(path.to_bytes().as_ptr()),
            Some(0),
            KEY_READ,
            &mut handle,
        );

        // Проверяем, был ли ключ успешно открыт
        if status != ERROR_SUCCESS {
            println!("Failed to open registry key: {:?}", status);
            return Err(windows::core::Error::from_win32());
        }

        // Подготавливаем чтение значения
        let mut buffer: [u8; 1024] = [0; 1024];
        let mut buffer_size: u32 = buffer.len() as u32;
        let mut typ = REG_SZ;

        // Читаем значение из реестра
        let read_status = RegQueryValueExA(
            handle, 
            PCSTR(value_name_cstr.to_bytes().as_ptr()), 
            None, 
            Some(&mut typ), 
            Some(buffer.as_mut_ptr()),
            Some(&mut buffer_size),
        );

        // Закрываем дескриптор ключа реестра
        let _ = RegCloseKey(handle);

        // Проверяем успешность чтения
        if read_status != ERROR_SUCCESS {
            println!("Failed to read registry value: {:?}", read_status);
            return Err(windows::core::Error::from_win32());
        }

        // Преобразуем буфер в строку и возвращаем
        let result = String::from_utf8_lossy(&buffer[..buffer_size as usize - 1]);
        Ok(result.to_string())
    }
}

fn start_game_process(game_path: &PathBuf, dll_path: &PathBuf) -> windows::core::Result<u32> {
    unsafe {
        let mut startup_info = STARTUPINFOA::default();
        startup_info.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
        let mut process_info = PROCESS_INFORMATION::default();

        let game_path_str = match game_path.to_str() {
            Some(path) => path,
            None => return Err(windows::core::Error::from_win32()),
        };

        // Форматируем путь в виде CString для Win32 API
        let game_path_cstr = CString::new(game_path_str).expect("CString::new failed");
        
        // Получаем директорию для использования в качестве рабочей директории
        let game_dir = match game_path.parent() {
            Some(dir) => dir.to_str().unwrap_or(""),
            None => "",
        };
        
        let game_dir_cstr = if !game_dir.is_empty() {
            Some(CString::new(game_dir).expect("CString::new failed"))
        } else {
            None
        };

        let dir_ptr = match &game_dir_cstr {
            Some(dir) => PCSTR(dir.as_ptr() as *const u8),
            None => PCSTR::null(),
        };
            
        // Создаём процесс в приостановленном состоянии
        println!("Starting game process...");
        logger::info("Запуск процесса игры...");
        let create_result = CreateProcessA(
            PCSTR(game_path_cstr.as_ptr() as *const u8),
            Some(PSTR::null()),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            dir_ptr,
            &mut startup_info,
            &mut process_info,
        );

        if let Err(e) = create_result {
            println!("Failed to create process: {:?}", e);
            logger::error(&format!("Не удалось создать процесс: {:?}", e));
            return Err(e);
        }

        // Открываем процесс с полным доступом
        println!("Opening process with full access...");
        logger::info("Открытие процесса с полным доступом...");
        let process_handle = OpenProcess(PROCESS_ALL_ACCESS, false, process_info.dwProcessId)?;

        // Конвертируем путь к DLL в CString
        let dll_path_cstring = CString::new(dll_path.to_str().unwrap_or(""))
            .map_err(|_| windows::core::Error::from_win32())?;
        let dll_path_bytes = dll_path_cstring.as_bytes_with_nul();

        // Выделяем память в удалённом процессе для пути к DLL
        println!("Allocating memory in target process...");
        logger::info("Выделение памяти в целевом процессе...");
        let remote_memory = VirtualAllocEx(
            process_handle,
            None,
            dll_path_bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_memory.is_null() {
            println!("Failed to allocate memory in target process");
            logger::error("Не удалось выделить память в целевом процессе");
            // Завершаем процесс, так как не смогли выделить память
            let _ = TerminateProcess(process_handle, 1);
            return Err(windows::core::Error::from_win32());
        }

        // Записываем путь к DLL в выделенную память
        println!("Writing DLL path to target process memory...");
        logger::info("Запись пути DLL в память целевого процесса...");
        let mut bytes_written = 0;
        let write_result = WriteProcessMemory(
            process_handle,
            remote_memory,
            dll_path_bytes.as_ptr() as *const std::ffi::c_void,
            dll_path_bytes.len(),
            Some(&mut bytes_written),
        );

        if let Err(e) = write_result {
            println!("Failed to write process memory: {:?}", e);
            logger::error(&format!("Не удалось записать память процесса: {:?}", e));
            // Освобождаем память и завершаем процесс
            let _ = VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            let _ = TerminateProcess(process_handle, 1);
            return Err(e);
        }

        // Получаем адрес функции LoadLibraryA
        println!("Getting LoadLibraryA address...");
        logger::info("Получение адреса функции LoadLibraryA...");
        let kernel32_handle = GetModuleHandleA(PCSTR(b"kernel32.dll\0".as_ptr())).unwrap();
        let load_library_addr = GetProcAddress(kernel32_handle, PCSTR(b"LoadLibraryA\0".as_ptr()));

        if load_library_addr.is_none() {
            println!("Failed to get LoadLibraryA address");
            logger::error("Не удалось получить адрес LoadLibraryA");
            // Освобождаем память и завершаем процесс
            let _ = VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            let _ = TerminateProcess(process_handle, 1);
            return Err(windows::core::Error::from_win32());
        }

        // Создаём удалённый поток для загрузки нашей DLL
        println!("Creating remote thread to load DLL...");
        logger::info("Создание удаленного потока для загрузки DLL...");
        let thread_handle = CreateRemoteThread(
            process_handle,
            None,
            0,
            Some(std::mem::transmute(load_library_addr.unwrap())),
            Some(remote_memory),
            0,
            None,
        )?;

        // Ждём, пока поток загрузки DLL завершится
        println!("Waiting for DLL to load...");
        logger::info("Ожидание загрузки DLL...");
        WaitForSingleObject(thread_handle, 10000); // Ждём до 10 секунд

        // Получаем код возврата из удалённого потока
        let mut exit_code = 0;
        GetExitCodeThread(thread_handle, &mut exit_code)?;

        if exit_code == 0 {
            println!("WARNING: DLL load may have failed");
            logger::warning("ВНИМАНИЕ: Загрузка DLL могла завершиться неудачей");
        } else {
            println!("DLL injected successfully (handle: 0x{:X})", exit_code);
            logger::info(&format!("DLL успешно внедрена (handle: 0x{:X})", exit_code));
        }

        // Освобождаем память и закрываем дескрипторы
        let _ = VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
        let _ = CloseHandle(thread_handle);
        
        // Возобновляем выполнение основного потока процесса
        println!("Resuming main thread...");
        logger::info("Возобновление основного потока...");
        let resume_result = ResumeThread(process_info.hThread);
        if resume_result == u32::MAX {
            logger::error("Не удалось возобновить основной поток");
            return Err(windows::core::Error::from_win32());
        }
        
        // Закрываем дескрипторы процесса и основного потока
        let _ = CloseHandle(process_info.hThread);
        let _ = CloseHandle(process_info.hProcess);
        let _ = CloseHandle(process_handle);
        
        // Возвращаем PID запущенного процесса
        Ok(process_info.dwProcessId)
    }
}