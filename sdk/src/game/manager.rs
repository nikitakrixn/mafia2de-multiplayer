//! Высокоуровневый API для глобального `C_Game` / GameManager.
//!
//! Этот модуль — удобная runtime-обёртка над `structures::GameManager`.
//! Используется для:
//! - чтения глобального состояния игры,
//! - доступа к active player,
//! - проверки флагов загрузки/инициализации,
//! - вызова отдельных virtual methods `C_Game`.

use std::ffi::CStr;

use crate::addresses;
use crate::memory::{self, Ptr};
use crate::structures::{CPlayer, GameManager};

use super::base;

/// Высокоуровневая обёртка над глобальным `C_Game`.
///
/// Хранит типизированный указатель на `GameManager`.
#[derive(Debug, Clone, Copy)]
pub struct Game {
    ptr: Ptr<GameManager>,
}

impl Game {
    // =========================================================================
    //  Конструкторы
    // =========================================================================

    /// Получить глобальный `C_Game`.
    ///
    /// Возвращает `None`, если:
    /// - модуль игры не найден,
    /// - глобальный указатель ещё не инициализирован,
    /// - адрес невалиден.
    #[inline]
    pub fn get() -> Option<Self> {
        let mgr_addr = unsafe { memory::read_ptr(base() + addresses::globals::GAME_MANAGER)? };
        Some(Self {
            ptr: Ptr::new(mgr_addr),
        })
    }

    // =========================================================================
    //  Базовые аксессоры
    // =========================================================================

    /// Типизированный указатель на `GameManager`.
    #[inline]
    pub fn typed_ptr(&self) -> Ptr<GameManager> {
        self.ptr
    }

    /// Сырой адрес `C_Game`.
    #[inline]
    pub fn as_ptr(&self) -> usize {
        self.ptr.addr()
    }

    /// Ссылка на `GameManager`.
    ///
    /// # Safety
    ///
    /// Глобальный объект должен быть жив и валиден.
    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&GameManager> {
        unsafe { self.ptr.as_ref() }
    }

    // =========================================================================
    //  Поля структуры
    // =========================================================================

    /// Битовые флаги состояния игры.
    #[inline]
    pub fn state_flags(&self) -> Option<u32> {
        unsafe { self.as_ref().map(|g| g.game_state_flags) }
    }

    /// Глобальный счётчик игровых тиков.
    #[inline]
    pub fn tick_counter(&self) -> Option<u32> {
        unsafe { self.as_ref().map(|g| g.game_tick_counter) }
    }

    /// Путь SDS для city/shop ресурсов.
    #[inline]
    pub fn city_sds_path(&self) -> Option<&str> {
        let gm = unsafe { self.as_ref()? };
        let ptr = gm.sds_path_city;
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// Путь SDS для streaming ресурсов.
    #[inline]
    pub fn streaming_sds_path(&self) -> Option<&str> {
        let gm = unsafe { self.as_ref()? };
        let ptr = gm.sds_path_streaming;
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// Путь SDS для traffic ресурсов.
    #[inline]
    pub fn traffic_sds_path(&self) -> Option<&str> {
        let gm = unsafe { self.as_ref()? };
        let ptr = gm.sds_path_traffic;
        if ptr.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }

    /// Raw entity pointer из одного из 4 слотов.
    #[inline]
    pub fn entity_slot_raw(&self, index: usize) -> Option<usize> {
        let gm = unsafe { self.as_ref()? };
        if index >= gm.entity_slots.len() {
            return None;
        }
        let ptr = gm.entity_slots[index] as usize;
        if ptr == 0 || !memory::is_valid_ptr(ptr) {
            None
        } else {
            Some(ptr)
        }
    }

    /// Типизированный указатель на активного игрока (slot 0).
    #[inline]
    pub fn active_player_ptr(&self) -> Option<Ptr<CPlayer>> {
        let ptr = self.entity_slot_raw(0)?;
        Some(Ptr::new(ptr))
    }

    /// Ссылка на активного игрока.
    ///
    /// # Safety
    ///
    /// - slot 0 должен указывать на живой `CPlayer`
    /// - объект не должен быть уничтожен движком во время использования ссылки
    #[inline]
    pub unsafe fn active_player(&self) -> Option<&CPlayer> {
        let ptr = self.active_player_ptr()?;
        unsafe { ptr.to_ref() }
    }

    /// Есть ли активный игрок.
    #[inline]
    pub fn has_player(&self) -> bool {
        self.active_player_ptr().is_some()
    }

    // =========================================================================
    //  Флаги / состояние
    // =========================================================================

    /// Мир загружен (`bit 1`).
    #[inline]
    pub fn is_loaded(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|g| g.is_loaded()) }
    }

    /// Мир инициализирован (`bit 0`).
    #[inline]
    pub fn is_initialized(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|g| g.is_initialized()) }
    }

    /// Мир приостановлен (`bit 2`).
    #[inline]
    pub fn is_suspended(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|g| g.is_suspended()) }
    }

    /// Полностью готов ли мир: и загружен, и инициализирован.
    #[inline]
    pub fn is_ready(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|g| g.is_ready()) }
    }

    /// Идёт ли сейчас обход entity в `Tick`.
    #[inline]
    pub fn is_tick_in_progress(&self) -> Option<bool> {
        unsafe { self.as_ref().map(|g| g.is_tick_in_progress()) }
    }

    // =========================================================================
    //  VTable вызовы
    // =========================================================================

    /// VTable[17] — `GetTickCounter`.
    #[inline]
    pub fn vtbl_get_tick_counter(&self) -> Option<u32> {
        let gm = unsafe { self.as_ref()? };
        let vt = unsafe { &*gm.vtable };
        Some(unsafe { (vt.get_tick_counter)(gm as *const _ as *const _) })
    }

    /// VTable[18] — `GetMissionManager`.
    #[inline]
    pub fn vtbl_get_mission_manager(&self) -> Option<usize> {
        let gm = unsafe { self.as_ref()? };
        let vt = unsafe { &*gm.vtable };
        let ptr = unsafe { (vt.get_mission_manager)(gm as *const _ as *const _) } as usize;
        if ptr == 0 { None } else { Some(ptr) }
    }

    /// VTable[20] — `GetEntityFromIndex`.
    #[inline]
    pub fn vtbl_get_entity_from_index(&self, index: u32) -> Option<usize> {
        let gm = unsafe { self.as_ref()? };
        let vt = unsafe { &*gm.vtable };
        let ptr = unsafe { (vt.get_entity_from_index)(gm as *const _ as *const _, index) } as usize;
        if ptr == 0 || !memory::is_valid_ptr(ptr) {
            None
        } else {
            Some(ptr)
        }
    }

    /// VTable[22] — `GetGamePhase`.
    #[inline]
    pub fn vtbl_get_game_phase(&self) -> Option<u8> {
        let gm = unsafe { self.as_ref()? };
        let vt = unsafe { &*gm.vtable };
        Some(unsafe { (vt.get_game_phase)(gm as *const _ as *const _) })
    }
}