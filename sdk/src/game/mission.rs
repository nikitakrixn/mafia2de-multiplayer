//! Высокоуровневый API для `C_Mission`.

use crate::addresses;
use crate::memory::{self, Ptr};
use crate::structures::CMission;
use super::base;

/// Обёртка над глобальным `C_Mission`.
#[derive(Debug, Clone, Copy)]
pub struct Mission {
    ptr: Ptr<CMission>,
}

impl Mission {
    /// Получить глобальный `C_Mission`.
    #[inline]
    pub fn get() -> Option<Self> {
        let addr = unsafe { memory::read_ptr(base() + addresses::globals::MISSION)? };
        if !memory::is_valid_ptr(addr) { return None; }
        Some(Self { ptr: Ptr::new(addr) })
    }

    #[inline]
    pub fn as_ptr(&self) -> usize { self.ptr.addr() }

    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&CMission> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn is_game_inited(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|m| m.is_game_inited()) }
    }

    #[inline]
    pub fn is_opened(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|m| m.is_opened()) }
    }

    #[inline]
    pub fn mission_name(&self) -> Option<&str> {
        unsafe { self.as_ref()?.mission_name() }
    }

    #[inline]
    pub fn state_flags(&self) -> Option<u32> {
        unsafe { self.as_ref().map(|m| m.state_flags) }
    }

    #[inline]
    pub fn scene_ptr(&self) -> Option<usize> {
        unsafe { self.as_ref().map(|m| m.scene_ptr as usize) }
    }

    #[inline]
    pub fn unk_11c(&self) -> Option<u16> {
        unsafe { self.as_ref().map(|m| m.unk_11c) }
    }
}
