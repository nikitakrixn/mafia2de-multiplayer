//! Управление — блокировка, стиль, физика.

use std::ffi::{CStr, CString};

use crate::addresses;
use crate::memory;
use common::logger;

use super::{Player, base};

impl Player {
    /// Pointer на компонент управления (`CHuman.property_accessor`).
    pub fn control_component_ptr(&self) -> Option<usize> {
        let ptr = unsafe { self.human()?.property_accessor as usize };
        memory::is_valid_ptr(ptr).then_some(ptr)
    }

    /// Заблокировано ли управление.
    pub fn are_controls_locked(&self) -> Option<bool> {
        let control = self.control_component_ptr()?;
        type Fn = unsafe extern "C" fn(usize) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::IS_LOCKED)
        };
        Some(unsafe { func(control) != 0 })
    }

    /// Заблокировать / разблокировать управление.
    pub fn lock_controls(&self, locked: bool) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls: компонент управления NULL");
            return false;
        };
        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED)
        };
        unsafe { func(control, locked as u8, 0) };
        true
    }

    /// Заблокировать для проигрывания анимации.
    pub fn lock_controls_to_play_anim(&self) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls_to_play_anim: компонент управления NULL");
            return false;
        };
        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED)
        };
        unsafe { func(control, 1, 1) };
        true
    }

    /// Принудительная блокировка (минуя проверку состояния).
    pub fn lock_controls_force(&self, locked: bool) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls_force: компонент управления NULL");
            return false;
        };

        let control_ref = unsafe {
            match memory::read_ptr(control) {
                Some(ptr) => ptr,
                None => return false,
            }
        };

        let internal_ptr = unsafe {
            match memory::read_ptr(control_ref + 248) {
                Some(ptr) => ptr,
                None => return false,
            }
        };

        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED_INTERNAL)
        };

        unsafe { func(internal_ptr + 112, locked as u8, 0) };
        true
    }

    /// Текущий стиль управления ("Normal", "DoNothing", ...).
    pub fn get_control_style_str(&self) -> Option<String> {
        let control = self.control_component_ptr()?;
        type Fn = unsafe extern "C" fn(usize) -> *const i8;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::GET_STYLE_STR)
        };
        let ptr = unsafe { func(control) };
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
    }

    /// Установить стиль управления по имени.
    pub fn set_control_style_str(&self, style: &str) -> bool {
        let Some(control) = self.control_component_ptr() else {
            return false;
        };
        let Ok(c_style) = CString::new(style) else {
            return false;
        };
        type Fn = unsafe extern "C" fn(usize, *const i8) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_STYLE_STR)
        };
        unsafe { func(control, c_style.as_ptr()) != 0 }
    }

    /// Текущее состояние физики.
    pub fn get_phys_state(&self) -> Option<u32> {
        let provider = unsafe { self.human()?.physics_provider as usize };
        if !memory::is_valid_ptr(provider) {
            return None;
        }
        unsafe {
            let vtable = memory::read_ptr(provider)?;
            let func_ptr = memory::read_ptr(vtable + 53 * 8)?;
            type GetStateFn = unsafe extern "C" fn(usize) -> u32;
            let func: GetStateFn = std::mem::transmute(func_ptr);
            Some(func(provider))
        }
    }

    /// Установить состояние физики через движковую функцию.
    pub fn set_phys_state(&self, state: u32) -> bool {
        let Some(control) = self.control_component_ptr() else {
            return false;
        };
        type Fn = unsafe extern "C" fn(usize, u32) -> u64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::physics::SET_PHYS_STATE)
        };
        unsafe { func(control, state) };
        true
    }
}