//! Транспорт.

use crate::memory;

use super::Player;

impl Player {
    pub fn is_in_vehicle(&self) -> Option<bool> {
        unsafe { self.human().map(|h| !h.actor.owner.is_null()) }
    }

    pub fn get_vehicle_ptr(&self) -> Option<usize> {
        let owner = unsafe { self.human()?.actor.owner as usize };
        memory::is_valid_ptr(owner).then_some(owner)
    }
}
