//! Здоровье и god mode.

use common::logger;
use sdk::game::Player;

pub fn toggle_god_mode() {
    let Some(player) = Player::get_active() else {
        logger::warn("[god-mode] Player not found");
        return;
    };

    let new_state = !player.is_god_mode();
    player.set_god_mode(new_state);

    let hp = player.get_health().unwrap_or(0.0);
    let max_hp = player.get_health_max().unwrap_or(0.0);
    logger::info(&format!(
        "[god-mode] HP: {:.0}/{:.0} ({:.0}%) | Invuln: {} | Demigod: {}",
        hp, max_hp,
        player.get_health_percent().unwrap_or(0.0),
        player.is_invulnerable().unwrap_or(false),
        player.is_demigod().unwrap_or(false),
    ));
}

pub fn dump_health_candidates() {
    let Some(player) = Player::get_active() else {
        logger::warn("[health-probe] no player");
        return;
    };

    let offsets: &[usize] = &[
        0x140, 0x144, 0x148, 0x14C,
        0x150, 0x154, 0x158, 0x15C,
        0x160, 0x164, 0x168, 0x16C,
        0x170, 0x174, 0x178, 0x17C,
    ];

    let ptr = player.as_ptr();
    logger::info(&format!("[health-probe] player=0x{ptr:X}"));

    for &off in offsets {
        match unsafe { sdk::memory::read_value::<f32>(ptr + off) } {
            Some(v) => logger::info(&format!("  +0x{off:03X} = {v:.6}")),
            None => logger::warn(&format!("  +0x{off:03X} unreadable")),
        }
    }
}