//! Корневые игровые структуры.

use super::CHuman;
use crate::macros::assert_field_offsets;

/// Глобальный менеджер игры.
///
/// Через него движок хранит указатель на текущего активного игрока.
/// Глобальный указатель: [`crate::addresses::globals::GAME_MANAGER`]
///
/// Цепочка доступа:
/// `*(module_base + GAME_MANAGER)` → `GameManager*`
/// `GameManager + 0x180` → `C_Human*` (активный игрок)
#[repr(C)]
pub struct GameManager {
    _pad: [u8; 0x180],
    /// Указатель на C_Human активного игрока.
    /// NULL если игрок ещё не создан (меню, загрузка).
    pub active_player: *mut CHuman,
}

assert_field_offsets!(GameManager {
    active_player == 0x180,
});