//! Высокоуровневый API игрока.

mod appearance;
mod controls;
mod debug;
mod health;
mod money;
mod movement;
mod state;
mod vehicle;
mod weapons;

use std::ffi::c_void;
use std::time::{Duration, Instant};

use crate::addresses;
use crate::memory::{self, Ptr};
use crate::structures::{CEntity, CHuman, CHumanVTable, CPlayer, GameManager};
use common::logger;

use super::base;
use super::entity_types::{EntityType, FactoryType};

/// Обёртка над указателем на CPlayer в памяти игры.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    ptr: Ptr<CPlayer>,
}

// =============================================================================
//  Конструкторы
// =============================================================================

impl Player {
    pub fn get_active() -> Option<Self> {
        unsafe {
            let mgr_addr = memory::read_ptr(base() + addresses::globals::GAME_MANAGER)?;
            let mgr_ptr = Ptr::<GameManager>::new(mgr_addr);
            let mgr = mgr_ptr.as_ref()?;

            let ptr = mgr.player_ptr()?;
            let player = ptr.as_ref()?;

            if !memory::is_valid_ptr(player.base.actor.base.vtable as usize) {
                return None;
            }

            Some(Self { ptr })
        }
    }

    pub fn wait_until_ready(timeout_secs: u64) -> Option<Self> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let mut reported = false;
        loop {
            if let Some(player) = Self::get_active() {
                if !reported {
                    logger::info(&format!(
                        "Указатель на игрока: 0x{:X}, жду инвентарь...",
                        player.as_ptr(),
                    ));
                    reported = true;
                }
                if player.is_ready() {
                    logger::info("Игрок полностью инициализирован");
                    return Some(player);
                }
            }
            if Instant::now() > deadline {
                logger::error(if reported {
                    "Таймаут: инвентарь так и не появился"
                } else {
                    "Таймаут: указатель на игрока так и не появился"
                });
                return None;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}

// =============================================================================
//  Базовые аксессоры
// =============================================================================

impl Player {
    #[inline]
    pub fn as_ptr(&self) -> usize {
        self.ptr.addr()
    }

    #[inline]
    pub fn typed_ptr(&self) -> Ptr<CPlayer> {
        self.ptr
    }

    pub fn is_ready(&self) -> bool {
        self.inventory_ptr().is_some()
    }

    pub fn inventory_ptr(&self) -> Option<usize> {
        let inv = unsafe { self.human()?.inventory };
        let addr = inv as usize;
        memory::is_valid_ptr(addr).then_some(addr)
    }

    pub fn table_id(&self) -> Option<u32> {
        unsafe { self.entity().map(|e| e.table_id) }
    }

    pub fn factory_type_byte(&self) -> Option<u8> {
        unsafe { self.entity().map(|e| e.factory_type()) }
    }

    pub fn factory_type(&self) -> Option<FactoryType> {
        FactoryType::from_byte(self.factory_type_byte()?)
    }

    pub fn entity_type(&self) -> Option<EntityType> {
        let ft = self.factory_type()?;
        EntityType::from_factory_type(ft as u8)
    }

    pub fn name_hash(&self) -> Option<u64> {
        unsafe { self.entity().map(|e| e.name_hash) }
    }
}

// =============================================================================
//  Внутренние typed helpers
// =============================================================================

impl Player {
    #[inline]
    pub(crate) unsafe fn entity(&self) -> Option<&CEntity> {
        unsafe { self.ptr.as_ref().map(|p| &p.base.actor.base) }
    }

    #[inline]
    pub(crate) unsafe fn human(&self) -> Option<&CHuman> {
        unsafe { self.ptr.as_ref().map(|p| &p.base) }
    }

    #[inline]
    pub(crate) unsafe fn player(&self) -> Option<&CPlayer> {
        unsafe { self.ptr.as_ref() }
    }

    /// Типизированный доступ к VTable.
    ///
    /// Реальная vtable в `.rdata` содержит 110 слотов.
    /// Наша структура описывает 83 — достаточно для всех текущих вызовов.
    ///
    /// # Safety
    ///
    /// `self.ptr` должен указывать на валидный Player/Human объект.
    #[inline]
    pub(crate) unsafe fn vtable(&self) -> Option<&CHumanVTable> {
        let entity = unsafe { self.entity()? };
        let vt_ptr = entity.vtable as *const CHumanVTable;
        if vt_ptr.is_null() {
            return None;
        }
        Some(unsafe { &*vt_ptr })
    }

    /// `this` pointer как `*const c_void` для vtable-вызовов.
    #[inline]
    pub(crate) fn this_const(&self) -> *const c_void {
        self.ptr.raw() as *const c_void
    }

    /// `this` pointer как `*mut c_void` для vtable-вызовов.
    #[inline]
    pub(crate) fn this_mut(&self) -> *mut c_void {
        self.ptr.raw() as *mut c_void
    }
}
