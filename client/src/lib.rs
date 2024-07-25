use windows::Win32::{
    Foundation::{BOOL, HINSTANCE},
    System::Console::AllocConsole,
  };
  
  fn init_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};
  
    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
      .or_else(|_| EnvFilter::try_new("info"))
      .unwrap();
  
    tracing_subscriber::registry()
      .with(filter_layer)
      .with(fmt_layer)
      .with(ErrorLayer::default())
      .init();
  
    eyre_span::install().unwrap();
  }
  
  fn dll_init() {
    unsafe { AllocConsole() };
    println!("Hello, world!");
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