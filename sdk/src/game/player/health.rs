//! Здоровье, неуязвимость, полубог, god mode.

use crate::addresses;
use crate::memory;
use common::logger;

use super::{Player, base};

impl Player {
    /// Текущее здоровье. 720.0 = полное на нормальной сложности.
    pub fn get_health(&self) -> Option<f32> {
        unsafe { self.human().map(|h| h.current_health) }
    }

    /// Установить здоровье напрямую.
    ///
    /// Если поставить ≤ 0.0 — персонаж НЕ умрёт автоматически,
    /// смерть тригерится только из кода урона.
    pub fn set_health(&self, value: f32) -> bool {
        if !self.ptr.is_valid() {
            return false;
        }
        unsafe {
            (&raw mut (*self.ptr.raw()).base.current_health).write(value);
        }
        true
    }

    /// Максимум здоровья. Хранится в глобальной структуре, НЕ в entity.
    pub fn get_health_max(&self) -> Option<f32> {
        unsafe { memory::read_value::<f32>(base() + addresses::globals::PLAYER_DATA) }
    }

    /// Установить максимум здоровья.
    pub fn set_health_max(&self, value: f32) -> bool {
        unsafe { memory::write(base() + addresses::globals::PLAYER_DATA, value) }
    }

    /// Полностью восстановить здоровье.
    pub fn heal_full(&self) -> bool {
        match self.get_health_max() {
            Some(max_hp) => self.set_health(max_hp),
            None => {
                logger::warn("heal_full: не удалось прочитать healthmax, ставлю 720.0");
                self.set_health(720.0)
            }
        }
    }

    /// Добавить здоровье (не выше максимума).
    pub fn add_health(&self, amount: f32) -> Option<f32> {
        let current = self.get_health()?;
        let max_hp = self.get_health_max().unwrap_or(720.0);
        let new_hp = (current + amount).min(max_hp).max(0.0);
        self.set_health(new_hp);
        Some(new_hp)
    }

    /// Здоровье в процентах (0.0 — 100.0).
    pub fn get_health_percent(&self) -> Option<f32> {
        let current = self.get_health()?;
        let max_hp = self.get_health_max().unwrap_or(720.0);
        Some(if max_hp > 0.0 {
            (current / max_hp) * 100.0
        } else {
            0.0
        })
    }

    /// Жив ли игрок.
    pub fn is_alive(&self) -> Option<bool> {
        unsafe { self.human().map(|h| h.is_alive()) }
    }

    /// Флаг неуязвимости.
    pub fn is_invulnerable(&self) -> Option<bool> {
        unsafe { self.human().map(|h| h.is_invulnerable()) }
    }

    /// Установить/снять неуязвимость.
    pub fn set_invulnerable(&self, enabled: bool) -> bool {
        if !self.ptr.is_valid() {
            return false;
        }
        unsafe {
            (&raw mut (*self.ptr.raw()).base.invulnerability).write(enabled as u8);
        }
        true
    }

    /// Режим полубога.
    pub fn is_demigod(&self) -> Option<bool> {
        unsafe { self.human().map(|h| h.is_demigod()) }
    }

    /// Установить/снять полубога.
    pub fn set_demigod(&self, enabled: bool) -> bool {
        if !self.ptr.is_valid() {
            return false;
        }
        unsafe {
            (&raw mut (*self.ptr.raw()).base.demigod).write(enabled as u8);
        }
        true
    }

    /// God mode: неуязвимость + полубог + полное здоровье.
    pub fn set_god_mode(&self, enabled: bool) -> bool {
        let ok1 = self.set_invulnerable(enabled);
        let ok2 = self.set_demigod(enabled);
        if enabled {
            self.heal_full();
        }
        if ok1 && ok2 {
            logger::info(&format!(
                "God Mode: {}",
                if enabled {
                    "ВКЛЮЧЁН"
                } else {
                    "ВЫКЛЮЧЕН"
                }
            ));
        } else {
            logger::error("God Mode: не удалось записать флаги");
        }
        ok1 && ok2
    }

    /// Активен ли God Mode.
    pub fn is_god_mode(&self) -> bool {
        self.is_invulnerable().unwrap_or(false) && self.is_demigod().unwrap_or(false)
    }
}
