use std::path::PathBuf;

use windows::{
    core::{PCSTR, PSTR}, Win32::{Foundation::*, System::Registry::{RegOpenKeyA, RegOpenKeyExA, HKEY}, UI::Controls::Dialogs::*}
};


const SUB_KEY: &str = "Software\\EmpireBay-Network";
const SUB_KEY_VALUE: &str = "mafia2exepath";
const CORE_DLL_NAME: &str = "ebn_client.dll";
const GAME_NAME: &str = "Mafia 2";
const GAME_EXE_NAME: &str = "mafia2.exe";

fn main() -> windows::core::Result<()>  {
    let game_path = folder_game_path(GAME_EXE_NAME)?;
    println!("Mafia II Path: {}", game_path.display());

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

fn write_registry_string(hkeylocation: HKEY, subkey: &str, subkeyvalue: &str) -> bool {
    let mut handle = HKEY::default(); 
    RegOpenKeyExA(hkey, lpsubkey, uloptions, samdesired, phkresult);
}