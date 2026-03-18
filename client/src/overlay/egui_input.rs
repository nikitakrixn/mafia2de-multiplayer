// Конвертация Win32 input → egui::RawInput
//
// Не используем GetAsyncKeyState(0x0001) — он крадёт нажатия у игры
// Вместо этого: edge detection через 0x8000 для горячих клавиш мода,
// а для egui собираем состояние мыши и modifier'ов

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

// Собрать egui::RawInput из текущего состояния Win32 input
// Вызывается из render thread каждый кадр когда overlay активен
pub fn collect_raw_input(
    screen_w: f32,
    screen_h: f32,
    hwnd: Option<usize>,
    wants_mouse: bool,
) -> egui::RawInput {
    let mut raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(screen_w, screen_h),
        )),
        ..Default::default()
    };

    // Мышь — только когда overlay хочет захватить
    if wants_mouse {
        if let Some(pos) = get_cursor_client_pos(hwnd) {
            raw.events.push(egui::Event::PointerMoved(pos));
        }

        // Клик левой кнопки
        let lmb_down = is_key_down(VK_LBUTTON);
        static LMB_WAS: OnceLock<Mutex<bool>> = OnceLock::new();
        let was = {
            let mut g = LMB_WAS.get_or_init(|| Mutex::new(false)).lock().unwrap();
            let was = *g;
            *g = lmb_down;
            was
        };

        if let Some(pos) = get_cursor_client_pos(hwnd) {
            if lmb_down && !was {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: get_modifiers(),
                });
            } else if !lmb_down && was {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: get_modifiers(),
                });
            }
        }

        // Правая кнопка
        let rmb_down = is_key_down(VK_RBUTTON);
        static RMB_WAS: OnceLock<Mutex<bool>> = OnceLock::new();
        let rmb_was = {
            let mut g = RMB_WAS.get_or_init(|| Mutex::new(false)).lock().unwrap();
            let was = *g;
            *g = rmb_down;
            was
        };

        if let Some(pos) = get_cursor_client_pos(hwnd) {
            if rmb_down && !rmb_was {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: true,
                    modifiers: get_modifiers(),
                });
            } else if !rmb_down && rmb_was {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: false,
                    modifiers: get_modifiers(),
                });
            }
        }
    }

    raw.modifiers = get_modifiers();
    raw
}

fn is_key_down(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(vk.0 as i32) } as u16 & 0x8000) != 0
}

fn get_modifiers() -> egui::Modifiers {
    egui::Modifiers {
        alt: is_key_down(VK_MENU),
        ctrl: is_key_down(VK_CONTROL),
        shift: is_key_down(VK_SHIFT),
        mac_cmd: false,
        command: is_key_down(VK_CONTROL),
    }
}

fn get_cursor_client_pos(hwnd: Option<usize>) -> Option<egui::Pos2> {
    let mut pt = POINT::default();
    if unsafe { GetCursorPos(&mut pt) }.is_err() {
        return None;
    }

    if let Some(h) = hwnd {
        let hwnd = windows::Win32::Foundation::HWND(h as *mut _);
        // ScreenToClient может вернуть ошибку — игнорируем
        let _ = unsafe { windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut pt) };
    }

    Some(egui::Pos2::new(pt.x as f32, pt.y as f32))
}

// ═══════════════════════════════════════════════════════════════
//  Edge detection для горячих клавиш (не для egui)
// ═══════════════════════════════════════════════════════════════

static KEY_STATES: OnceLock<Mutex<HashMap<u16, bool>>> = OnceLock::new();

fn key_states() -> &'static Mutex<HashMap<u16, bool>> {
    KEY_STATES.get_or_init(|| Mutex::new(HashMap::with_capacity(64)))
}

// Edge detection: was_up → is_down
// Использует ТОЛЬКО 0x8000 — НЕ крадёт ввод у игры
pub fn just_pressed(vk: VIRTUAL_KEY) -> bool {
    let currently_down = is_key_down(vk);
    let Ok(mut states) = key_states().lock() else {
        return false;
    };
    let was_down = states.get(&vk.0).copied().unwrap_or(false);
    states.insert(vk.0, currently_down);
    currently_down && !was_down
}

pub fn is_held(vk: VIRTUAL_KEY) -> bool {
    is_key_down(vk)
}