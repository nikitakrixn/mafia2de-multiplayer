use windows::Win32::{
    Foundation::{BOOL, HINSTANCE},
    System::Console::AllocConsole,
  };
  
  
  fn dll_init() {
    unsafe { AllocConsole() };
    println!("My first Rust DLL!");
  }
  
  #[no_mangle]
  #[allow(non_snake_case, unused_variables)]
  extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, reserved: u32) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;
  
    match call_reason {
      DLL_PROCESS_ATTACH => dll_init(),
      DLL_PROCESS_DETACH => (),
      _ => (),
    }
  
    BOOL::from(true)
  }