//! State machine — коды состояний, флаги, sub45c.

use crate::addresses::constants;

use super::Player;

impl Player {
    /// Главный state code (`CPlayer.state_code`, +0x430).
    pub fn get_state_code(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.state_code) }
    }

    /// Состояние CarWrapper (`CPlayer.car_wrapper.data[0xA0]`, +0x3D8).
    pub fn get_car_wrapper_state(&self) -> Option<u8> {
        unsafe { self.player().map(|p| p.car_wrapper_state()) }
    }

    /// Маска/профиль стиля управления (`CPlayer.ctrl_style_mask`, +0x438).
    pub fn get_ctrl_style_mask(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.ctrl_style_mask) }
    }

    /// Битовое поле боевой системы (`CPlayer.fight_flags`, +0x490).
    pub fn get_fight_flags(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.fight_flags) }
    }

    /// `CPlayer.sub45c.state`.
    pub fn get_sub45c_state(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.sub45c.state) }
    }

    /// Флаги состояния (`CPlayer.state_flags_510`, +0x510).
    pub fn get_state_flags_510(&self) -> Option<u32> {
        unsafe { self.player().map(|p| p.state_flags_510) }
    }

    // =========================================================================
    //  Bit extraction из fight_flags (+0x490)
    // =========================================================================

    pub fn get_fight_ability(&self) -> Option<u32> {
        let flags = self.get_fight_flags()?;
        Some((flags & constants::player_state_flags_490::MASK_BITS_1_3) >> 1)
    }

    pub fn get_fight_control_style(&self) -> Option<u32> {
        let flags = self.get_fight_flags()?;
        Some((flags & constants::player_state_flags_490::MASK_BITS_4_6) >> 4)
    }

    pub fn get_fight_hint(&self) -> Option<u32> {
        let flags = self.get_fight_flags()?;
        Some((flags & constants::player_state_flags_490::MASK_BITS_7_13) >> 7)
    }

    pub fn is_fight_grab_time_scale(&self) -> Option<bool> {
        let flags = self.get_fight_flags()?;
        Some((flags & constants::player_state_flags_490::BIT_14) != 0)
    }

    pub fn is_forced_drop_weapon(&self) -> Option<bool> {
        let flags = self.get_fight_flags()?;
        Some((flags & constants::player_state_flags_490::BIT_15) != 0)
    }
}
