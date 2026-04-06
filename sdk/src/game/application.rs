//! Высокоуровневый API для `C_Application` (GameModule).

use crate::addresses;
use crate::memory::{self, Ptr};
use crate::structures::CApplication;

use super::base;

/// Обёртка над глобальным `C_Application`.
#[derive(Debug, Clone, Copy)]
pub struct Application {
    ptr: Ptr<CApplication>,
}

impl Application {
    /// Получить глобальный `C_Application`.
    ///
    /// Возвращает `None` если модуль не инициализирован.
    #[inline]
    pub fn get() -> Option<Self> {
        let addr = unsafe { memory::read_ptr(base() + addresses::globals::APPLICATION)? };
        if !memory::is_valid_ptr(addr) {
            return None;
        }
        Some(Self {
            ptr: Ptr::new(addr),
        })
    }

    /// Сырой адрес объекта.
    #[inline]
    pub fn as_ptr(&self) -> usize {
        self.ptr.addr()
    }

    /// Ссылка на `CApplication`.
    ///
    /// # Safety
    /// Объект должен быть жив и валиден.
    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&CApplication> {
        unsafe { self.ptr.as_ref() }
    }

    /// Имя текущей игры/карты.
    #[inline]
    pub fn game_name(&self) -> Option<&str> {
        unsafe { self.as_ref()?.game_name() }
    }

    /// Номер текущей миссии (-1 если нет).
    #[inline]
    pub fn mission_number(&self) -> Option<i64> {
        unsafe { self.as_ref().map(|a| a.mission_number) }
    }

    /// Часть текущей миссии.
    #[inline]
    pub fn mission_part(&self) -> Option<u32> {
        unsafe { self.as_ref().map(|a| a.mission_part) }
    }

    /// Последний reload был успешным.
    #[inline]
    pub fn is_reload_success(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|a| a.is_reload_success()) }
    }

    /// Миссия загружена.
    #[inline]
    pub fn has_mission(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|a| a.has_mission()) }
    }
}
