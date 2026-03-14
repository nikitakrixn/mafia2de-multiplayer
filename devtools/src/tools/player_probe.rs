//! Player probe

use common::logger;
use sdk::{game::Player, memory, addresses::fields};

pub fn dump_player_range(start_off: usize, size: usize) {
    let Some(player) = Player::get_active() else {
        logger::warn("[player-probe] no active player");
        return;
    };
    let base = player.as_ptr() + start_off;
    logger::info(&format!("[player-probe] dump +0x{start_off:X} size=0x{size:X}"));
    logger::info(&memory::hex_dump(base, size));
}

pub fn dump_frame_directions() {
    let Some(player) = Player::get_active() else {
        logger::warn("[frame-probe] no player");
        return;
    };

    let Some(frame) = (unsafe {
        memory::read_ptr_raw(player.as_ptr() + fields::player::FRAME_NODE)
    }) else {
        logger::warn("[frame-probe] frame ptr unreadable");
        return;
    };

    if frame == 0 {
        logger::warn("[frame-probe] frame is null");
        return;
    }

    unsafe {
        let px = memory::read_value::<f32>(frame + fields::entity_frame::POS_X).unwrap_or(0.0);
        let py = memory::read_value::<f32>(frame + fields::entity_frame::POS_Y).unwrap_or(0.0);
        let pz = memory::read_value::<f32>(frame + fields::entity_frame::POS_Z).unwrap_or(0.0);

        let fx = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_X).unwrap_or(0.0);
        let fy = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_Y).unwrap_or(0.0);
        let fz = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_Z).unwrap_or(0.0);

        let rx = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_X).unwrap_or(0.0);
        let ry = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_Y).unwrap_or(0.0);
        let rz = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_Z).unwrap_or(0.0);

        logger::info(&format!("[frame] pos=({px:.2}, {py:.2}, {pz:.2})"));
        logger::info(&format!("[frame] forward=({fx:.3}, {fy:.3}, {fz:.3})"));
        logger::info(&format!("[frame] right=({rx:.3}, {ry:.3}, {rz:.3})"));
    }
}

pub fn dump_player_vtable() {
    let Some(player) = Player::get_active() else { return; };
    let ptr = player.as_ptr();
    let Some(vtable) = (unsafe { memory::read_ptr_raw(ptr) }) else { return; };

    logger::info(&format!("[vtable] player=0x{ptr:X} vtable=0x{vtable:X}"));
    for i in 0..96 {
        let fn_ptr = unsafe { memory::read_ptr_raw(vtable + i * 8).unwrap_or(0) };
        logger::info(&format!("[vtable][{i:02}] 0x{fn_ptr:X}"));
    }
}