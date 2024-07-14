use std::ffi::CString;
use std::thread;
use std::time::Duration;
use windows::Win32::System::Threading::{
    CreateProcessA, CreateRemoteThread, OpenProcess, ResumeThread, WaitForSingleObject, CREATE_SUSPENDED, PROCESS_ACCESS_RIGHTS, PROCESS_ALL_ACCESS, PROCESS_CREATE_THREAD, PROCESS_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE, STARTUPINFOA
};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RESERVE, MEM_RELEASE,
    PAGE_READWRITE,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, GetModuleHandleA};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::core::{PCSTR, PSTR};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExA;

fn main() -> windows::core::Result<()> {
    let game_path = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Mafia II\\pc\\mafia2.exe";
    let dll_path = "C:\\Users\\admin\\Documents\\Repos\\Mafia2-Multiplayer\\target\\debug\\client.dll";

    unsafe {
        const ALL_RIGHTS: u32 = 0xFFFF;
        let mut startup_info = STARTUPINFOA::default();
        let mut process_info = PROCESS_INFORMATION::default();
        println!("process {:?}", process_info.dwProcessId);

        println!("Starting the game process...");
        // Создание игрового процесса в приостановленном состоянии
        let game_path_cstring = CString::new(game_path).map_err(|_| windows::core::Error::from_win32())?;
        CreateProcessA(
            PCSTR::null(),
            PSTR(game_path_cstring.as_ptr() as *mut u8),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            PCSTR::null(),
            &mut startup_info,
            &mut process_info,
        )?;

        println!("Game process created. Waiting for it to initialize...");
        // Возобновление основного потока игры
        ResumeThread(process_info.hThread);

        // Ожидание инициализации игры
        wait_for_game_initialization(process_info.dwProcessId)?;

        println!("Game initialized. Injecting DLL...");

        println!("Loading DLL from {}", dll_path);

        println!("process {:?}", process_info.dwProcessId);
        
        // Открытие процесса с нужными привилегиями
        let process_handle = OpenProcess(
            PROCESS_ACCESS_RIGHTS(ALL_RIGHTS),
            false,
            process_info.dwProcessId,
        )?;

        // Выделение памяти в процессе игры для пути DLL
        let dll_path_cstring = CString::new(dll_path).map_err(|_| windows::core::Error::from_win32())?;
        let dll_path_bytes = dll_path_cstring.as_bytes_with_nul();
        let remote_memory = VirtualAllocEx(
            process_handle,
            None,
            dll_path_bytes.len(),
            MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_memory.is_null() {
            return Err(windows::core::Error::from_win32());
        }

        // Запись пути DLL в выделенную память
        let mut bytes_written = 0;
        WriteProcessMemory(
            process_handle,
            remote_memory,
            dll_path_bytes.as_ptr() as _,
            dll_path_bytes.len(),
            Some(&mut bytes_written),
        )?;

        // Получение адреса LoadLibraryA
        let kernel32 = GetModuleHandleA(PCSTR("kernel32.dll\0".as_ptr()))?;
        let load_library_addr = GetProcAddress(kernel32, PCSTR("LoadLibraryA\0".as_ptr()));

        if load_library_addr.is_none() {
            return Err(windows::core::Error::from_win32());
        }

        // Создание удаленного потока для загрузки нашей DLL
        let thread = CreateRemoteThread(
            process_handle,
            None,
            0,
            Some(std::mem::transmute(load_library_addr)),
            Some(remote_memory),
            0,
            None,
        )?;

        println!("Waiting for DLL to load...");
        // Ожидание завершения потока
        WaitForSingleObject(thread, 10000); // Ожидание до 10 секунд

        // Очистка
        CloseHandle(thread)?;
        VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE)?;

        // Очистка дескрипторов процесса
        CloseHandle(process_handle)?;
        CloseHandle(process_info.hThread)?;

        println!("DLL injected successfully!");
        Ok(())
    }
}

unsafe fn wait_for_game_initialization(process_id: u32) -> windows::core::Result<()> {
    let process_handle = OpenProcess(
        PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ, 
        false, 
        process_id
    )?;
    
    let mut module_name_buf = [0u8; 1024];
    let mut retry_count = 0;
    loop {
        let result = K32GetModuleFileNameExA(
            process_handle,
            None,
            &mut module_name_buf,
        );
        if result != 0 {
            println!("Game executable loaded.");
            break;
        }
        retry_count += 1;
        if retry_count > 50 {  // Ожидание до 5 секунд
            return Err(windows::core::Error::from_win32());
        }
        thread::sleep(Duration::from_millis(100));
    }
    // Дополнительное ожидание для полной инициализации игры
    thread::sleep(Duration::from_secs(5));
    
    CloseHandle(process_handle)?;
    Ok(())
}