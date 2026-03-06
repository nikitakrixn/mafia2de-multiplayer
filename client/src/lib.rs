//! Клиентская DLL для Mafia II: DE Multiplayer.

use std::ffi::c_void;
use std::time::Duration;

use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

use common::logger;
use sdk::game::{self, Player};

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;
const TRUE: i32 = 1;

const GAME_LOAD_TIMEOUT: u64 = 180;
const MONITOR_INTERVAL: u64 = 5;
const INPUT_POLL_MS: u64 = 100;

// Клавиши
const VK_F5: i32  = 0x74;  // +$100  (HUD)
const VK_F6: i32  = 0x75;  // +$500  (HUD)
const VK_F7: i32  = 0x76;  // +$1000 (HUD)
const VK_F8: i32  = 0x77;  // -$500  (HUD)
const VK_F9: i32  = 0x78;  // =$9999.99 (прямая запись)
const VK_F10: i32 = 0x79;  // баланс

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(
    _module: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(initialize);
            TRUE
        }
        DLL_PROCESS_DETACH => {
            logger::info("Client shutting down...");
            TRUE
        }
        _ => TRUE,
    }
}

fn is_key_just_pressed(vk: i32) -> bool {
    let state = unsafe { GetAsyncKeyState(vk) };
    (state & 0x0001) != 0
}

fn initialize() {
    unsafe { let _ = AllocConsole(); }

    if let Err(e) = logger::init(
        logger::Level::Debug,
        logger::Target::Both,
        Some("logs/m2mp_client.log"),
    ) {
        eprintln!("[m2mp] Logger failed: {e}");
    }

    logger::info("======================================");
    logger::info("  Mafia II: DE Multiplayer Client");
    logger::info("  Version 0.1.0 | x86_64");
    logger::info("======================================");

    game::log_module_info();
    logger::info("Waiting for game to fully load...");

    let player = match Player::wait_until_ready(GAME_LOAD_TIMEOUT) {
        Some(p) => p,
        None => {
            logger::error(&format!("Timeout ({GAME_LOAD_TIMEOUT}s)"));
            return;
        }
    };

    player.log_debug_info();

    if !player.is_wallet_ready() {
        logger::info("Wallet not ready, waiting...");
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if let Some(p) = Player::get_active() {
                if p.is_wallet_ready() {
                    logger::info("Wallet initialized!");
                    break;
                }
            }
            if std::time::Instant::now() > deadline {
                logger::warn("Wallet timeout — will work once money appears in-game");
                break;
            }
        }
    }

    log_balance(&player);

    logger::info("======================================");
    logger::info("  Keybinds:");
    logger::info("    F5  — Add $100  (with HUD)");
    logger::info("    F6  — Add $500  (with HUD)");
    logger::info("    F7  — Add $1000 (with HUD)");
    logger::info("    F8  — Remove $500 (with HUD)");
    logger::info("    F9  — Set $9999.99 (memory)");
    logger::info("    F10 — Show balance");
    logger::info("======================================");
    logger::info("  Client initialized!");
    logger::info("======================================");

    std::thread::spawn(monitor_loop);
    input_loop();
}

fn input_loop() {
    logger::debug("[input] Input loop started");

    loop {
        std::thread::sleep(Duration::from_millis(INPUT_POLL_MS));

        let Some(player) = Player::get_active() else { continue };

        if is_key_just_pressed(VK_F5) {
            do_add_money(&player, 100);
        }
        if is_key_just_pressed(VK_F6) {
            do_add_money(&player, 500);
        }
        if is_key_just_pressed(VK_F7) {
            do_add_money(&player, 1000);
        }
        if is_key_just_pressed(VK_F8) {
            do_add_money(&player, -500);
        }
        if is_key_just_pressed(VK_F9) {
            logger::info("Setting money to $9999.99");
            player.set_money(999_999);
            log_balance(&player);
        }
        if is_key_just_pressed(VK_F10) {
            log_balance(&player);
        }
    }
}

/// Добавляет деньги с HUD. Если HUD недоступен — прямая запись.
fn do_add_money(player: &Player, dollars: i32) {
    let sign = if dollars >= 0 { "Adding" } else { "Removing" };
    logger::info(&format!("{sign} ${}", dollars.abs()));

    // Пробуем с HUD уведомлением
    if player.add_money_dollars_with_hud(dollars) {
        log_balance(player);
    } else {
        // Fallback — прямая запись
        logger::warn("HUD unavailable, using direct write");
        match player.add_money_dollars(dollars) {
            Some(new) => logger::info(&format!(
                "  → Balance: $ {}.{:02}", new / 100, (new % 100).abs()
            )),
            None => logger::error("  → Failed: wallet not allocated"),
        }
    }
}

fn log_balance(player: &Player) {
    match player.get_money_cents() {
        Some(c) => logger::info(&format!(
            "Balance: {} cents = $ {}.{:02}",
            c, c / 100, (c % 100).abs()
        )),
        None => logger::info("Balance: wallet not allocated"),
    }
}

fn monitor_loop() {
    loop {
        std::thread::sleep(Duration::from_secs(MONITOR_INTERVAL));

        let Some(player) = Player::get_active() else {
            logger::debug("[monitor] Player not available");
            continue;
        };

        let money = player.get_money_formatted().unwrap_or_else(|| "N/A".into());
        logger::debug(&format!("[monitor] {money}"));
    }
}