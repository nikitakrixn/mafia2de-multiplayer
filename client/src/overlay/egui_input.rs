// Конвертация Win32 input -> egui::RawInput
//
// Архитектура:
// - WndProc хук (hooks.rs) перехватывает WM_CHAR / WM_KEYDOWN / WM_KEYUP
//   и кладёт события в очередь `WNDPROC_QUEUE`
// - collect_raw_input() каждый кадр дренирует очередь в egui::RawInput
// - GetAsyncKeyState используется ТОЛЬКО для мыши и edge detection горячих клавиш

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

// =============================================================================
//  Очередь событий от WndProc
// =============================================================================

#[derive(Debug)]
pub enum WndProcEvent {
    /// Unicode символ (WM_CHAR) — любая раскладка, кириллица, всё
    Char(char),
    /// Служебная клавиша нажата (WM_KEYDOWN)
    KeyDown(egui::Key),
    /// Служебная клавиша отпущена (WM_KEYUP)
    KeyUp(egui::Key),
}

static WNDPROC_QUEUE: OnceLock<Mutex<VecDeque<WndProcEvent>>> = OnceLock::new();

fn wndproc_queue() -> &'static Mutex<VecDeque<WndProcEvent>> {
    WNDPROC_QUEUE.get_or_init(|| Mutex::new(VecDeque::with_capacity(64)))
}

/// Вызывается из WndProc хука — кладёт событие в очередь.
pub fn push_wndproc_event(event: WndProcEvent) {
    if let Ok(mut q) = wndproc_queue().lock() {
        q.push_back(event);
    }
}

/// Сбросить очередь — используется при открытии чата чтобы
/// клавиша-триггер (T) не попала в поле ввода.
pub fn flush_wndproc_queue() {
    if let Ok(mut q) = wndproc_queue().lock() {
        q.clear();
    }
}

/// Маппинг VK -> egui::Key для служебных клавиш.
pub fn vk_to_egui_key(vk: u16) -> Option<egui::Key> {
    match VIRTUAL_KEY(vk) {
        VK_BACK => Some(egui::Key::Backspace),
        VK_DELETE => Some(egui::Key::Delete),
        VK_RETURN => Some(egui::Key::Enter),
        VK_ESCAPE => Some(egui::Key::Escape),
        VK_LEFT => Some(egui::Key::ArrowLeft),
        VK_RIGHT => Some(egui::Key::ArrowRight),
        VK_UP => Some(egui::Key::ArrowUp),
        VK_DOWN => Some(egui::Key::ArrowDown),
        VK_HOME => Some(egui::Key::Home),
        VK_END => Some(egui::Key::End),
        VK_TAB => Some(egui::Key::Tab),
        _ => None,
    }
}

// =============================================================================
//  Сбор RawInput для egui
// =============================================================================

/// Собрать egui::RawInput из текущего состояния Win32 input.
/// Вызывается из render thread каждый кадр.
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

        // Левая кнопка мыши
        let lmb_down = is_key_down(VK_LBUTTON);
        static LMB_WAS: OnceLock<Mutex<bool>> = OnceLock::new();
        let lmb_was = {
            let mut g = LMB_WAS.get_or_init(|| Mutex::new(false)).lock().unwrap();
            let was = *g;
            *g = lmb_down;
            was
        };
        if let Some(pos) = get_cursor_client_pos(hwnd) {
            if lmb_down != lmb_was {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: lmb_down,
                    modifiers: get_modifiers(),
                });
            }
        }

        // Правая кнопка мыши
        let rmb_down = is_key_down(VK_RBUTTON);
        static RMB_WAS: OnceLock<Mutex<bool>> = OnceLock::new();
        let rmb_was = {
            let mut g = RMB_WAS.get_or_init(|| Mutex::new(false)).lock().unwrap();
            let was = *g;
            *g = rmb_down;
            was
        };
        if let Some(pos) = get_cursor_client_pos(hwnd) {
            if rmb_down != rmb_was {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Secondary,
                    pressed: rmb_down,
                    modifiers: get_modifiers(),
                });
            }
        }

        // Клавиатура — дренируем очередь от WndProc хука
        drain_wndproc_events(&mut raw.events);
    }

    raw.modifiers = get_modifiers();
    raw
}

/// Дренирует очередь WndProc событий в egui events.
fn drain_wndproc_events(events: &mut Vec<egui::Event>) {
    let Ok(mut q) = wndproc_queue().lock() else {
        return;
    };

    let modifiers = get_modifiers();

    for ev in q.drain(..) {
        match ev {
            WndProcEvent::Char(ch) => {
                if !ch.is_control() {
                    events.push(egui::Event::Text(ch.to_string()));
                }
            }
            WndProcEvent::KeyDown(key) => {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                });
            }
            WndProcEvent::KeyUp(key) => {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                });
            }
        }
    }
}

// =============================================================================
//  Вспомогательные функции
// =============================================================================

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
        let _ = unsafe { windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut pt) };
    }
    Some(egui::Pos2::new(pt.x as f32, pt.y as f32))
}

// =============================================================================
//  Edge detection для горячих клавиш мода (не для egui)
// =============================================================================

static KEY_STATES: OnceLock<Mutex<HashMap<u16, bool>>> = OnceLock::new();

fn key_states() -> &'static Mutex<HashMap<u16, bool>> {
    KEY_STATES.get_or_init(|| Mutex::new(HashMap::with_capacity(64)))
}

/// Edge detection: true только в момент перехода was_up -> is_down.
pub fn just_pressed(vk: VIRTUAL_KEY) -> bool {
    let currently_down = is_key_down(vk);
    let Ok(mut states) = key_states().lock() else {
        return false;
    };
    let was_down = states.get(&vk.0).copied().unwrap_or(false);
    states.insert(vk.0, currently_down);
    currently_down && !was_down
}

#[allow(dead_code)]
pub fn is_held(vk: VIRTUAL_KEY) -> bool {
    is_key_down(vk)
}
