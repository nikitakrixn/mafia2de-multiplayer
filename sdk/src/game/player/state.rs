//! State machine — коды состояний, флаги, sub45c.

use crate::addresses::{constants, fields};

use super::Player;

impl Player {
    /// Главный state code (`CPlayer.state_code_430`).
    pub fn get_state_code_430(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.state_code_430) }
    }

    /// State/flags dword (`CPlayer.state_flags_3d8`).
    pub fn get_state_flags_3d8(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.state_flags_3d8) }
    }

    /// State mask / profile (`CPlayer.state_mask_438`).
    pub fn get_state_mask_438(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.state_mask_438) }
    }

    /// Bitfield (`CPlayer.state_flags_490`).
    pub fn get_state_flags_490(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.state_flags_490) }
    }

    /// `CPlayer.sub45c.state`.
    pub fn get_sub45c_state(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.sub45c.state) }
    }

    /// Дополнительный state/flags dword (`+0x510`).
    /// Не в именованном поле CPlayer — в `_player_tail`.
    pub fn get_state_flags_510(&self) -> Option<u32> {
        unsafe { self.ptr.read_at::<u32>(fields::player::STATE_FLAGS_510) }
    }

    // =========================================================================
    //  Bit extraction из state_flags_490
    // =========================================================================

    pub fn get_state_flags_bits_1_3(&self) -> Option<u32> {
        let flags = self.get_state_flags_490()?;
        Some((flags & constants::player_state_flags_490::MASK_BITS_1_3) >> 1)
    }

    pub fn get_state_flags_bits_4_6(&self) -> Option<u32> {
        let flags = self.get_state_flags_490()?;
        Some((flags & constants::player_state_flags_490::MASK_BITS_4_6) >> 4)
    }

    pub fn get_state_flags_bits_7_13(&self) -> Option<u32> {
        let flags = self.get_state_flags_490()?;
        Some((flags & constants::player_state_flags_490::MASK_BITS_7_13) >> 7)
    }

    pub fn is_state_flag_bit_14_set(&self) -> Option<bool> {
        let flags = self.get_state_flags_490()?;
        Some((flags & constants::player_state_flags_490::BIT_14) != 0)
    }

    pub fn is_state_flag_bit_15_set(&self) -> Option<bool> {
        let flags = self.get_state_flags_490()?;
        Some((flags & constants::player_state_flags_490::BIT_15) != 0)
    }
}