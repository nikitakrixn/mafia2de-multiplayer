//! Команды игрока: статус, телепорт, деньги, оружие.

use common::logger;
use sdk::game::Player;
use sdk::game::player::Vec3;

pub fn show_status(player: &Player) {
    logger::info(&format!("State: {}", crate::state::get().as_str()));
    logger::info(&format!("Noclip: {}", crate::tools::noclip::is_active()));

    match player.are_controls_locked() {
        Some(locked) => logger::info(&format!("Controls locked: {locked}")),
        None => logger::error("Controls: unreadable"),
    }

    match player.get_control_style_str() {
        Some(style) => logger::info(&format!("Control style: \"{style}\"")),
        None => logger::error("Control style: unavailable"),
    }

    match player.get_position() {
        Some(pos) => logger::info(&format!("Position: {pos}")),
        None => logger::error("Position: unavailable"),
    }
}

pub fn teleport(player: &Player) {
    match player.get_position() {
        Some(mut pos) => {
            logger::info(&format!("Current: {pos}"));
            pos.x = 1261.0;
            pos.y = 1169.0;
            pos.z = 0.5;
            if player.set_position(&pos) {
                logger::info(&format!("Teleported to: {pos}"));
            } else {
                logger::error("Teleport failed");
            }
        }
        None => logger::error("Position unavailable"),
    }
}

pub fn show_balance(player: &Player) {
    match player.get_money_cents() {
        Some(c) => logger::info(&format!(
            "Balance: {c} cents = $ {}.{:02}", c / 100, (c % 100).abs()
        )),
        None => logger::info("Balance: wallet not ready"),
    }
}

pub fn add_money(player: &Player, dollars: i32) {
    let action = if dollars >= 0 { "Adding" } else { "Removing" };
    logger::info(&format!("{action} ${}", dollars.abs()));

    if player.add_money_dollars_with_hud(dollars) {
        show_balance(player);
    } else {
        logger::warn("HUD unavailable, writing directly");
        match player.add_money_dollars(dollars) {
            Some(new) => logger::info(&format!("Balance: $ {}.{:02}", new / 100, (new % 100).abs())),
            None => logger::error("Wallet not initialized"),
        }
    }
}

pub fn set_money(player: &Player, cents: i64) {
    player.set_money(cents);
    show_balance(player);
}

pub fn give_weapon(player: &Player, weapon_id: u32, ammo: u32, name: &str) {
    logger::info(&format!("Giving {name} + {ammo} ammo"));
    if player.add_weapon(weapon_id, ammo) {
        logger::info(&format!("  → {name} added!"));
    } else {
        logger::error(&format!("  → Failed to add {name}"));
    }
}