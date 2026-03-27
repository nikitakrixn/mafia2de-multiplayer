//! Корневые игровые структуры.

use super::CPlayer;
use crate::macros::assert_field_offsets;
use crate::memory::Ptr;

/// Глобальный менеджер игры.
///
/// Через него движок хранит указатель на текущего активного игрока.
/// Глобальный указатель: [`crate::addresses::globals::GAME_MANAGER`]
///
/// Цепочка доступа:
/// ```text
/// *(module_base + GAME_MANAGER) -> GameManager*
/// GameManager + 0x180 -> CPlayer* (активный игрок)
/// ```
#[repr(C)]
pub struct GameManager {
    _pad: [u8; 0x180],
    /// Указатель на CPlayer активного игрока.
    /// NULL если игрок ещё не создан (меню, загрузка).
    pub active_player: *mut CPlayer,
}

assert_field_offsets!(GameManager {
    active_player == 0x180,
});

impl GameManager {
    /// Типизированный указатель на активного игрока.
    ///
    /// Возвращает `None`, если игрок ещё не создан или указатель невалиден.
    #[inline]
    pub fn player_ptr(&self) -> Option<Ptr<CPlayer>> {
        let addr = self.active_player as usize;
        crate::memory::is_valid_ptr(addr).then_some(Ptr::new(addr))
    }

    /// Получить ссылку на активного игрока.
    ///
    /// Возвращает `None`, если игрок ещё не создан.
    ///
    /// # Safety
    ///
    /// - `GameManager` должен быть валиден.
    /// - `active_player` должен указывать на живой `CPlayer`.
    /// - Использовать только в корректном игровом контексте.
    #[inline]
    pub unsafe fn get_player(&self) -> Option<&CPlayer> {
        let ptr = self.active_player;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }

    /// Получить мутабельную ссылку на активного игрока.
    ///
    /// # Safety
    ///
    /// - Те же требования, что и для [`get_player`](Self::get_player)
    /// - Не должно быть других активных ссылок на этот объект.
    #[inline]
    pub unsafe fn get_player_mut(&mut self) -> Option<&mut CPlayer> {
        let ptr = self.active_player;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *ptr })
    }

    /// Проверить, что игрок создан.
    #[inline]
    pub fn has_player(&self) -> bool {
        !self.active_player.is_null()
    }
}
