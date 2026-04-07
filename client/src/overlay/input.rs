//! Ввод: WndProc -> очередь -> egui::RawInput.

use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

#[derive(Debug)]
pub enum WndProcEvent {
    Char(char),
    KeyDown(egui::Key),
    KeyUp(egui::Key),
}

static QUEUE: LazyLock<Mutex<VecDeque<WndProcEvent>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(64)));

pub fn push_event(event: WndProcEvent) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push_back(event);
    }
}

pub fn flush() {
    if let Ok(mut q) = QUEUE.lock() {
        q.clear();
    }
}

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
        VK_A => Some(egui::Key::A),
        VK_C => Some(egui::Key::C),
        VK_V => Some(egui::Key::V),
        VK_X => Some(egui::Key::X),
        VK_Z => Some(egui::Key::Z),
        _ => None,
    }
}

struct MouseState {
    lmb: bool,
    rmb: bool,
}

static MOUSE: LazyLock<Mutex<MouseState>> =
    LazyLock::new(|| Mutex::new(MouseState { lmb: false, rmb: false }));

pub fn collect(
    screen_w: f32,
    screen_h: f32,
    hwnd: Option<usize>,
    wants_input: bool,
) -> egui::RawInput {
    let mut raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(screen_w, screen_h),
        )),
        modifiers: modifiers(),
        ..Default::default()
    };

    if !wants_input {
        return raw;
    }

    if let Some(pos) = cursor_pos(hwnd) {
        raw.events.push(egui::Event::PointerMoved(pos));
        collect_mouse_buttons(&mut raw.events, pos);
    }

    drain_queue(&mut raw.events);
    raw
}

fn collect_mouse_buttons(events: &mut Vec<egui::Event>, pos: egui::Pos2) {
    let lmb_now = is_down(VK_LBUTTON);
    let rmb_now = is_down(VK_RBUTTON);
    let mods = modifiers();

    let Ok(mut mouse) = MOUSE.lock() else { return };

    if lmb_now != mouse.lmb {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: lmb_now,
            modifiers: mods,
        });
        mouse.lmb = lmb_now;
    }

    if rmb_now != mouse.rmb {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed: rmb_now,
            modifiers: mods,
        });
        mouse.rmb = rmb_now;
    }
}

fn drain_queue(events: &mut Vec<egui::Event>) {
    let Ok(mut q) = QUEUE.lock() else { return };
    let mods = modifiers();

    for ev in q.drain(..) {
        match ev {
            WndProcEvent::Char(ch) if !ch.is_control() => {
                events.push(egui::Event::Text(ch.to_string()));
            }
            WndProcEvent::KeyDown(key) => {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: mods,
                });
            }
            WndProcEvent::KeyUp(key) => {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: mods,
                });
            }
            _ => {}
        }
    }
}

static KEY_PREV: LazyLock<Mutex<[bool; 256]>> =
    LazyLock::new(|| Mutex::new([false; 256]));

pub fn just_pressed(vk: VIRTUAL_KEY) -> bool {
    let down = is_down(vk);
    let Ok(mut prev) = KEY_PREV.lock() else { return false };
    let idx = vk.0 as usize;
    if idx >= 256 { return false; }
    let was = prev[idx];
    prev[idx] = down;
    down && !was
}

/// true пока клавиша удерживается.
pub fn is_held(vk: VIRTUAL_KEY) -> bool {
    is_down(vk)
}

fn is_down(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(vk.0 as i32) } as u16 & 0x8000) != 0
}

fn modifiers() -> egui::Modifiers {
    egui::Modifiers {
        alt: is_down(VK_MENU),
        ctrl: is_down(VK_CONTROL),
        shift: is_down(VK_SHIFT),
        mac_cmd: false,
        command: is_down(VK_CONTROL),
    }
}

fn cursor_pos(hwnd: Option<usize>) -> Option<egui::Pos2> {
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