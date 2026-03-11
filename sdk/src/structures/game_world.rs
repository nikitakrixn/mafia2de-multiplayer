use super::CHuman;

/// Глобальный менеджер игры.
///
/// Указатель: [`crate::addresses::globals::GAME_MANAGER`]
#[repr(C)]
pub struct GameManager {
    _pad: [u8; 0x180],
    pub active_player: *mut CHuman,
}

const _: () = {
    assert!(std::mem::offset_of!(GameManager, active_player) == 0x180);
};