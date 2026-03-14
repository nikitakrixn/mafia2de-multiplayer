//! Background monitoring
//! Опциональный, может быть запущен из initialize().

use std::time::Duration;
use common::logger;
use sdk::game::Player;
use crate::{runtime, state::{self, GameSessionState}};

const INTERVAL_SECS: u64 = 5;

pub fn run() {
    logger::debug("[monitor] started");

    loop {
        if runtime::is_shutting_down() { break; }
        std::thread::sleep(Duration::from_secs(INTERVAL_SECS));

        let current = state::refresh_from_runtime();

        if current == GameSessionState::InGame {
            if let Some(player) = Player::get_active() {
                let money = player.get_money_formatted()
                    .unwrap_or_else(|| "wallet not ready".into());
                logger::debug(&format!("[monitor] in-game | {money}"));
            }
        }

        if current == GameSessionState::ShuttingDown { break; }
    }
}