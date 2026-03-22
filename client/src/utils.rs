// Общие утилиты клиента

use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

// Проверяет, находится ли окно игры в фокусе
pub fn is_window_focused() -> bool {
    let Some(game_hwnd) = sdk::game::render::get_hwnd() else {
        return false;
    };

    unsafe {
        let foreground = GetForegroundWindow();
        foreground.0 as usize == game_hwnd
    }
}
