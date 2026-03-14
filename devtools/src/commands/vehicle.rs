//! Vehicle entity дамп.

use common::logger;
use sdk::{memory, game::Player};

pub fn dump_vehicle_entity() {
    let Some(player) = Player::get_active() else { return; };
    let pp = player.as_ptr();

    let owner = unsafe { memory::read_ptr_raw(pp + 0x80).unwrap_or(0) };
    if owner == 0 {
        logger::info("[vehicle] Not in vehicle (owner=NULL)");
        return;
    }

    let vtable = unsafe { memory::read_ptr_raw(owner).unwrap_or(0) };
    let entity_type = unsafe { memory::read_value::<u8>(owner + 0x24).unwrap_or(0xFF) };

    logger::info(&format!(
        "[vehicle] owner=0x{owner:X} type=0x{entity_type:02X} vtbl=0x{vtable:X}"
    ));
    logger::info(&memory::hex_dump(owner, 0x100));
}