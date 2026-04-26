//! Обход single-instance check

use std::ffi::CStr;
use std::sync::{Mutex, OnceLock};

use common::logger;
use windows::Win32::Foundation::{HANDLE, HMODULE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::core::{BOOL, PCSTR, PCWSTR};

const SINGLE_INSTANCE_MUTEX: &[u8] = b"Mafia2 Launcher Super Mutex";
const INSTALLER_MUTEX: &[u8] = b"Mafia2 Installer Super Mutex";

type FnCreateMutexA = unsafe extern "system" fn(
    sa: *const SECURITY_ATTRIBUTES,
    initial_owner: BOOL,
    name: PCSTR,
) -> HANDLE;

type FnOpenMutexA =
    unsafe extern "system" fn(desired_access: u32, inherit: BOOL, name: PCSTR) -> HANDLE;

static ORIG_CREATE_MUTEX_A: OnceLock<FnCreateMutexA> = OnceLock::new();
static ORIG_OPEN_MUTEX_A: OnceLock<FnOpenMutexA> = OnceLock::new();
static PENDING: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn defer_log(msg: String) {
    if let Ok(mut g) = PENDING.get_or_init(|| Mutex::new(Vec::new())).lock() {
        g.push(msg);
    }
}

/// Дописать накопленные при `install()` сообщения в основной лог.
/// Вызвать после `logger::init` из `client::initialize`.
pub fn flush_pending_logs() {
    let Some(lock) = PENDING.get() else { return };
    let Ok(mut g) = lock.lock() else { return };
    for msg in g.drain(..) {
        logger::info(&msg);
    }
}

const KERNEL32_W: [u16; 13] = [
    b'k' as u16, b'e' as u16, b'r' as u16, b'n' as u16, b'e' as u16, b'l' as u16, b'3' as u16,
    b'2' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16, 0,
];

/// Установить хуки. Вызывать **синхронно** из `DllMain`.
pub fn install() -> bool {
    let kernel32 = match unsafe { GetModuleHandleW(PCWSTR(KERNEL32_W.as_ptr())) } {
        Ok(h) => h,
        Err(e) => {
            defer_log(format!("[mutex-bypass] GetModuleHandle(kernel32): {e}"));
            return false;
        }
    };

    let create_ok = hook_one(
        kernel32,
        b"CreateMutexA\0",
        hook_create_mutex_a as *mut _,
        &ORIG_CREATE_MUTEX_A,
    );

    let open_ok = hook_one(
        kernel32,
        b"OpenMutexA\0",
        hook_open_mutex_a as *mut _,
        &ORIG_OPEN_MUTEX_A,
    );

    defer_log(format!(
        "[mutex-bypass] CreateMutexA={} OpenMutexA={} (target: \"Mafia2 *Super Mutex\")",
        if create_ok { "✓" } else { "✗" },
        if open_ok { "✓" } else { "✗" },
    ));

    create_ok || open_ok
}

fn hook_one<F: Copy + 'static>(
    module: HMODULE,
    proc_name: &[u8],
    detour: *mut std::ffi::c_void,
    slot: &OnceLock<F>,
) -> bool {
    let proc_name_c = match CStr::from_bytes_with_nul(proc_name) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let target_fn = match unsafe { GetProcAddress(module, PCSTR(proc_name_c.as_ptr().cast())) } {
        Some(f) => f as *mut std::ffi::c_void,
        None => {
            defer_log(format!(
                "[mutex-bypass] GetProcAddress({}) failed",
                proc_name_c.to_string_lossy()
            ));
            return false;
        }
    };

    match unsafe { minhook::MinHook::create_hook(target_fn, detour) } {
        Ok(trampoline) => {
            let typed: F = unsafe { std::mem::transmute_copy(&trampoline) };
            let _ = slot.set(typed);
            unsafe { minhook::MinHook::enable_hook(target_fn) }.is_ok()
        }
        Err(e) => {
            defer_log(format!(
                "[mutex-bypass] create_hook({}): {e:?}",
                proc_name_c.to_string_lossy()
            ));
            false
        }
    }
}

/// Strict-equal сравнение имени с одной из known single-instance сигнатур.
fn name_bytes_match(name: PCSTR, target: &[u8]) -> bool {
    if name.is_null() {
        return false;
    }
    let cstr = unsafe { CStr::from_ptr(name.0 as *const i8) };
    cstr.to_bytes() == target
}

unsafe extern "system" fn hook_create_mutex_a(
    sa: *const SECURITY_ATTRIBUTES,
    initial: BOOL,
    name: PCSTR,
) -> HANDLE {
    let orig = ORIG_CREATE_MUTEX_A.get().copied().expect("not installed");

    if name_bytes_match(name, SINGLE_INSTANCE_MUTEX) {
        log_once("[mutex-bypass] CreateMutexA(\"Mafia2 Launcher Super Mutex\") -> anonymous");
        return unsafe { orig(sa, initial, PCSTR::null()) };
    }
    unsafe { orig(sa, initial, name) }
}

unsafe extern "system" fn hook_open_mutex_a(
    desired: u32,
    inherit: BOOL,
    name: PCSTR,
) -> HANDLE {
    let orig = ORIG_OPEN_MUTEX_A.get().copied().expect("not installed");

    if name_bytes_match(name, INSTALLER_MUTEX) {
        log_once("[mutex-bypass] OpenMutexA(\"Mafia2 Installer Super Mutex\") -> NULL");
        return HANDLE::default();
    }
    unsafe { orig(desired, inherit, name) }
}

/// Лог только первого попадания
fn log_once(msg: &'static str) {
    static SEEN: OnceLock<Mutex<std::collections::HashSet<&'static str>>> = OnceLock::new();
    let set = SEEN.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    if let Ok(mut g) = set.lock()
        && g.insert(msg)
    {
        defer_log(msg.to_string());
    }
}
