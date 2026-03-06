//! Создание процесса игры и инжект DLL через CreateRemoteThread + LoadLibraryA.

use std::ffi::CString;
use std::path::Path;

use windows::Win32::Foundation::*;
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;
use windows::core::PCSTR;

use common::logger;
use crate::error::Error;

struct SafeHandle(HANDLE);

impl SafeHandle {
    fn raw(&self) -> HANDLE { self.0 }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { let _ = CloseHandle(self.0); }
        }
    }
}

/// Запускает игру в приостановленном состоянии, инжектит DLL, возобновляет.
///
/// Возвращает PID запущенного процесса.
pub fn launch_and_inject(game_exe: &Path, dll_path: &Path) -> Result<u32, Error> {
    let game_cstr = path_to_cstring(game_exe)?;
    let dir = game_exe
        .parent()
        .ok_or_else(|| Error::ProcessFailed("No parent directory".into()))?;
    let dir_cstr = path_to_cstring(dir)?;

    // 1. Создаём процесс (suspended)
    logger::info("Creating suspended game process...");
    let (process, thread, pid) = create_suspended_process(&game_cstr, &dir_cstr)?;
    logger::info(&format!("Process created (PID: {pid})"));

    // 2. Инжектим DLL
    match inject_dll(process.raw(), dll_path) {
        Ok(()) => {
            logger::info("DLL injected successfully");
        }
        Err(e) => {
            logger::error(&format!("Injection failed: {e}"));
            unsafe { let _ = TerminateProcess(process.raw(), 1); }
            return Err(e);
        }
    }

    // 3. Возобновляем основной поток
    logger::info("Resuming main thread...");
    let result = unsafe { ResumeThread(thread.raw()) };
    if result == u32::MAX {
        logger::error("Failed to resume main thread");
        unsafe { let _ = TerminateProcess(process.raw(), 1); }
        return Err(Error::ProcessFailed("ResumeThread failed".into()));
    }

    logger::info("Game is running!");
    Ok(pid)
}

/// Проверяет, запущен ли процесс с указанным именем.
pub fn is_process_running(exe_name: &str) -> Result<bool, Error> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let snap = SafeHandle(snapshot);

        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };

        if Process32First(snap.raw(), &mut entry).is_ok() {
            loop {
                let name_bytes: Vec<u8> = entry
                    .szExeFile
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8)
                    .collect();

                if let Ok(name) = std::str::from_utf8(&name_bytes)
                    && name.eq_ignore_ascii_case(exe_name) {
                        return Ok(true);
                    }

                if Process32Next(snap.raw(), &mut entry).is_err() {
                    break;
                }
            }
        }

        Ok(false)
    }
}

fn create_suspended_process(
    exe: &CString,
    working_dir: &CString,
) -> Result<(SafeHandle, SafeHandle, u32), Error> {
    unsafe {
        let si = STARTUPINFOA {
            cb: std::mem::size_of::<STARTUPINFOA>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();

        CreateProcessA(
            PCSTR(exe.as_ptr() as *const u8),
            None, // command line
            None, // process security
            None, // thread security
            false,
            CREATE_SUSPENDED,
            None, // environment
            PCSTR(working_dir.as_ptr() as *const u8),
            &si,
            &mut pi,
        )?;

        let process = SafeHandle(pi.hProcess);
        let thread  = SafeHandle(pi.hThread);
        Ok((process, thread, pi.dwProcessId))
    }
}

fn inject_dll(process: HANDLE, dll_path: &Path) -> Result<(), Error> {
    let dll_cstr = path_to_cstring(dll_path)?;
    let dll_bytes = dll_cstr.as_bytes_with_nul();

    unsafe {
        // 1. Выделяем память в целевом процессе
        let remote_buf = VirtualAllocEx(
            process,
            None,
            dll_bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_buf.is_null() {
            return Err(Error::InjectionFailed("VirtualAllocEx failed".into()));
        }

        // Гарантируем освобождение при ошибке
        let _guard = scopeguard(|| {
            let _ = VirtualFreeEx(process, remote_buf, 0, MEM_RELEASE);
        });

        // 2. Записываем путь к DLL
        WriteProcessMemory(
            process,
            remote_buf,
            dll_bytes.as_ptr().cast(),
            dll_bytes.len(),
            None,
        )
        .map_err(|e| Error::InjectionFailed(format!("WriteProcessMemory: {e}")))?;

        // 3. Находим LoadLibraryA
        let kernel32 = GetModuleHandleA(PCSTR(b"kernel32.dll\0".as_ptr()))
            .map_err(|e| Error::InjectionFailed(format!("GetModuleHandle: {e}")))?;

        let load_library = GetProcAddress(kernel32, PCSTR(b"LoadLibraryA\0".as_ptr()))
            .ok_or_else(|| Error::InjectionFailed("GetProcAddress(LoadLibraryA) failed".into()))?;

        logger::info(&format!(
            "LoadLibraryA @ 0x{:X}, remote buffer @ 0x{:X}",
            load_library as usize,
            remote_buf as usize,
        ));

        // 4. Создаём удалённый поток
        let thread = CreateRemoteThread(
            process,
            None,
            0,
            Some(std::mem::transmute(load_library)),
            Some(remote_buf),
            0,
            None,
        )
        .map_err(|e| Error::InjectionFailed(format!("CreateRemoteThread: {e}")))?;

        let thread = SafeHandle(thread);

        // 5. Ждём завершения загрузки
        let wait = WaitForSingleObject(thread.raw(), 15_000);
        match wait {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT  => logger::warn("DLL load timed out (15s)"),
            other         => logger::warn(&format!("WaitForSingleObject: {other:?}")),
        }

        // 6. Проверяем результат
        let mut exit_code: u32 = 0;
        let _ = GetExitCodeThread(thread.raw(), &mut exit_code);

        if exit_code == 0 {
            return Err(Error::InjectionFailed(
                "LoadLibraryA returned NULL — check DLL dependencies".into(),
            ));
        }

        logger::info(&format!("DLL loaded (module handle: 0x{exit_code:X})"));

        // Отменяем guard — DLL загружена, не освобождаем remote_buf
        // (процесс сам его освободит при завершении)
        std::mem::forget(_guard);
    }

    Ok(())
}


fn path_to_cstring(path: &Path) -> Result<CString, Error> {
    let s = path
        .to_str()
        .ok_or_else(|| Error::ProcessFailed("Path contains invalid UTF-8".into()))?;
    CString::new(s)
        .map_err(|_| Error::ProcessFailed("Path contains null byte".into()))
}

fn scopeguard<F: FnOnce()>(f: F) -> impl Drop {
    struct Guard<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Guard<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f();
            }
        }
    }
    Guard(Some(f))
}