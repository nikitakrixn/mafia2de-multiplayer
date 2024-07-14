use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK};
use windows::core::PCSTR;

fn initialize_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

static mut CONSOLE_ALLOCATED: bool = false;

fn show_error_message(error: &str) {
    unsafe {
        let caption = "Mafia II Multiplayer Mod Error";
        MessageBoxA(
            None,
            PCSTR::from_raw(error.as_ptr()),
            PCSTR::from_raw(caption.as_ptr()),
            MB_OK,
        );
    }
}

#[no_mangle]
pub extern "system" fn DllMain(
    dll_module: windows::Win32::Foundation::HINSTANCE,
    call_reason: u32,
    _: *mut ::std::ffi::c_void,
) -> windows::Win32::Foundation::BOOL {
    match call_reason {
        windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH => {
            unsafe {
                if AllocConsole().is_ok() {
                    CONSOLE_ALLOCATED = true;
                    println!("Mafia II Multiplayer Mod Console");
                    println!("Type 'help' for a list of commands.");
                } else {
                    show_error_message("Failed to allocate console");
                    return windows::Win32::Foundation::BOOL::from(false);
                }
            }

            initialize_tracing();
            info!("DLL attached successfully");
        }
        windows::Win32::System::SystemServices::DLL_PROCESS_DETACH => {
            unsafe {
                if CONSOLE_ALLOCATED {
                    FreeConsole();
                }
            }
            info!("DLL detached successfully");
        }
        _ => {}
    }
    windows::Win32::Foundation::BOOL::from(true)
}
