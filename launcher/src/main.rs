use std::{ffi::CString, path::PathBuf};

use windows::{
    core::{PCSTR, PSTR}, Win32::{Foundation::*, System::Registry::*, UI::Controls::Dialogs::*}
};


const SUB_KEY: &str = "Software\\EmpireBay-Network";
const SUB_KEY_VALUE: &str = "mafia2exepath";
const CORE_DLL_NAME: &str = "ebn_client.dll";
const GAME_NAME: &str = "Mafia 2";
const GAME_EXE_NAME: &str = "mafia2.exe";

fn main() -> windows::core::Result<()>  {
    let game_path_result = read_registry_string(HKEY_CURRENT_USER, SUB_KEY, SUB_KEY_VALUE);

    let game_path = match game_path_result {
        Ok(path) => {
            if path.is_empty() {
                let selected_path = folder_game_path(GAME_EXE_NAME)?;
                println!("Путь Mafia II (выбранный): {}", selected_path.display());
                write_registry_string(HKEY_CURRENT_USER, SUB_KEY, SUB_KEY_VALUE, selected_path.to_str().unwrap());
                selected_path
            } else {
                println!("Путь Mafia II (из реестра): {}", path);
                PathBuf::from(path)
            }
        }
        Err(e) => {
            println!("Не удалось прочитать путь из реестра: {:?}", e);
            let selected_path = folder_game_path(GAME_EXE_NAME)?;
            println!("Путь Mafia II (выбранный): {}", selected_path.display());
            write_registry_string(HKEY_CURRENT_USER, SUB_KEY, SUB_KEY_VALUE, selected_path.to_str().unwrap());
            selected_path
        },
    };

    //let dll_path = game_path.join(CORE_DLL_NAME);

    let dll_path = std::env::current_exe().unwrap().parent().unwrap().join(CORE_DLL_NAME);
    println!("DLL Path: {}", dll_path.display());

    Ok(())
}

fn folder_game_path (game_exe_name: &str) -> Result<PathBuf, windows::core::Error> {
    unsafe
    {
        let mut open_file_dialog = OPENFILENAMEA::default();
        let mut file_name_buffer = String::from_utf8(vec![0; 260]).unwrap();
        open_file_dialog.lStructSize  = std::mem::size_of::<OPENFILENAMEA>() as u32;
        open_file_dialog.hwndOwner    = HWND::default();
        open_file_dialog.lpstrFilter  = PCSTR("mafia2.exe file\0Mafia2.exe\0".as_ptr());
        open_file_dialog.nFilterIndex = 1;
        open_file_dialog.lpstrFile    = PSTR(file_name_buffer.as_bytes_mut().as_mut_ptr());
        open_file_dialog.nMaxFile     = file_name_buffer.len() as u32;
        open_file_dialog.lpstrTitle   = PCSTR(format!("Pick a {} executable", game_exe_name).as_ptr());
        open_file_dialog.Flags        = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

        if GetOpenFileNameA(&mut open_file_dialog).into() {
            // Check if the user actually selected a file
            if !file_name_buffer.is_empty() {
                Ok(PathBuf::from(String::from(file_name_buffer.trim_matches('\0'))))
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
                0, 
                KEY_WRITE, 
                &mut handle
            );

        if status != ERROR_SUCCESS {
            // If the key does not exist, create it
            if status == ERROR_FILE_NOT_FOUND {
                let create_status = RegCreateKeyA(
                    hkeylocation,
                    PCSTR(path.as_bytes().as_ptr()),
                    &mut handle,
                );

                // If creation was unsuccessful, return false
                if create_status != ERROR_SUCCESS {
                    println!("Failed to create registry key: {:?}", create_status);
                    return false;
                }
            } else {
                println!("Failed to open registry key: {:?}", status);
                return false;
            }
        }

        // Write the string value to the registry
        let write_status = RegSetValueExA(
            handle, 
            PCSTR(sub_key_value.to_bytes().as_ptr()), 
            0, 
            REG_SZ, 
            Some(value.to_bytes())
        );

        // Check if writing the value was successful
        if write_status != ERROR_SUCCESS {
            println!("Failed to write to registry: {:?}", write_status);
            return false;
        }

        // Close the handle to the registry key
        let _ = RegCloseKey(handle);

        println!("{:?}", write_status);

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
        
        // Open the registry key for reading
        let status = RegOpenKeyExA(
            hkeylocation,
            PCSTR(path.to_bytes().as_ptr()),
            0,
            KEY_READ,
            &mut handle,
        );

        // Check if the key was opened successfully
        if status != ERROR_SUCCESS {
            println!("Failed to open registry key: {:?}", status);
            return Err(windows::core::Error::from_win32());
        }

        // Prepare to read the value
        let mut buffer: [u8; 1024] = [0; 1024];
        let mut buffer_size: u32 = buffer.len() as u32;
        let mut typ = REG_SZ;

        // Read the value from the registry
        let read_status = RegQueryValueExA(
            handle, 
            PCSTR(value_name_cstr.to_bytes().as_ptr()), 
            None, 
            Some(&mut typ), 
            Some(buffer.as_mut_ptr()),
            Some(&mut buffer_size),
        );

        // Close the handle to the registry key
        let _ =RegCloseKey(handle);

        // Check if reading the value was successful
        if read_status != ERROR_SUCCESS {
            println!("Failed to read registry value: {:?}", read_status);
            return Err(windows::core::Error::from_win32());
        }

        // Convert the buffer to a string and return
        let result = String::from_utf8_lossy(&buffer[..buffer_size as usize]);
        Ok(result.to_string())
    }
}