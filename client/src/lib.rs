//! Клиентская DLL для Mafia II: DE Multiplayer.
mod events;
mod hooks;
mod lua_queue;
mod main_thread;
mod runtime;
mod state;
mod player_tracker;

use std::ffi::c_void;
use std::time::Duration;

use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

use common::logger;
use sdk::game::{self, Player};
use state::GameSessionState;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;
const TRUE: i32 = 1;

const MONITOR_INTERVAL: u64 = 5;
const INPUT_POLL_MS: u64 = 100;

const DUMP_CALLBACK_REGISTRY_ON_START: bool = false;
const DUMP_CALLBACK_EVENTS_ON_START: bool = true;

// Клавиши
const VK_INSERT: i32 = 0x2D; // Queue Lua command on main thread
const VK_DELETE: i32 = 0x2E; // Shutdown runtime/hooks

const VK_F1: i32  = 0x70;
const VK_F2: i32  = 0x71;
const VK_F3: i32  = 0x72;
const VK_F4: i32  = 0x73;
const VK_F5: i32  = 0x74;
const VK_F6: i32  = 0x75;
const VK_F7: i32  = 0x76;
const VK_F8: i32  = 0x77;
const VK_F9: i32  = 0x78;
const VK_F10: i32 = 0x79;
const VK_F11: i32 = 0x7A;
const VK_F12: i32 = 0x7B;

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
            runtime::shutdown();
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
    unsafe {
        let _ = AllocConsole();
    }

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

    lua_queue::init();
    player_tracker::init();
    let _ = state::refresh_from_runtime();

    sdk::game::lua::log_chain();

    if DUMP_CALLBACK_EVENTS_ON_START {
        sdk::game::callbacks::dump_interesting_events();
    }

    if DUMP_CALLBACK_REGISTRY_ON_START {
        sdk::game::callbacks::dump_registry();
    }

    logger::info("Installing hooks...");
    if let Err(e) = hooks::install() {
        logger::error(&format!("Failed to install hooks: {e}"));
        return;
    }

    logger::info("Runtime services online");
    logger::info("Client now starts immediately after injection");
    logger::info("Waiting for world/player state...");

    logger::info("======================================");
    logger::info("  Keybinds:");
    logger::info("    INSERT — Queue Lua +$100 via main-thread dispatcher");
    logger::info("    DELETE — Shutdown hook/runtime");
    logger::info("    F1  — Lock controls");
    logger::info("    F2  — Unlock controls");
    logger::info("    F3  — Status (controls + position)");
    logger::info("    F4  — Teleport to need");
    logger::info("    F5  — Add $100  (with HUD)");
    logger::info("    F6  — Add $500  (with HUD)");
    logger::info("    F7  — Add $1000 (with HUD)");
    logger::info("    F8  — Remove $500 (with HUD)");
    logger::info("    F9  — Set $9999.99 (memory)");
    logger::info("    F10 — Show balance");
    logger::info("    F11 — Give Thompson + 200 ammo");
    logger::info("    F12 — Give Colt 1911 + 50 ammo");
    logger::info("======================================");
    logger::info("  Client initialized!");
    logger::info("======================================");

    std::thread::spawn(monitor_loop);
    input_loop();
}

fn input_loop() {
    logger::debug("[input] Input loop started");

    loop {
        if runtime::is_shutting_down() {
            logger::debug("[input] stopping");
            break;
        }

        std::thread::sleep(Duration::from_millis(INPUT_POLL_MS));

        if is_key_just_pressed(VK_INSERT) {
            logger::info("Queueing Lua command on main thread...");
            lua_queue::queue_exec_named(
                "game.navigation:DisableFarVisibility(game.navigation:RegisterIconPos('-284.647','1148.50',0,3,'2155010008',true))",
                "=m2mp_insert_test",
            );
        }

        if is_key_just_pressed(VK_DELETE) {
            logger::info("Manual shutdown requested...");
            runtime::shutdown();
            break;
        }

        let player = Player::get_active();

        let Some(player) = player else {
            continue;
        };

        if is_key_just_pressed(VK_F1) {
            logger::info("Locking controls...");
            if player.lock_controls(true) {
                logger::info("  → Controls LOCKED");
            } else {
                logger::error("  → Failed to lock");
            }
        }

        if is_key_just_pressed(VK_F2) {
            logger::info("Unlocking controls...");
            if player.lock_controls(false) {
                logger::info("  → Controls UNLOCKED");
            } else {
                logger::error("  → Failed to unlock");
            }
        }

        if is_key_just_pressed(VK_F3) {
            logger::info(&format!("Hooks installed: {}", hooks::is_installed()));
            logger::info(&format!("App focus: {:?}", crate::events::app_focus_state()));
            logger::info(&format!("Session state: {}", crate::state::get().as_str()));

            match player.are_controls_locked() {
                Some(locked) => logger::info(&format!("Controls locked: {locked}")),
                None => logger::error("Failed to read control state"),
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

        if is_key_just_pressed(VK_F4) {
            match player.get_position() {
                Some(mut pos) => {
                    logger::info(&format!("Current position: {pos}"));
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

        if is_key_just_pressed(VK_F11) {
            logger::info("Adding Thompson 1928 + 200 ammo");
            if player.add_weapon(sdk::addresses::constants::weapons::THOMPSON_1928, 200) {
                logger::info("  → Thompson added!");
            }
        }

        if is_key_just_pressed(VK_F12) {
            logger::info("Adding Colt M1911A1 + 50 ammo");
            if player.add_weapon(sdk::addresses::constants::weapons::COLT_M1911A1, 50) {
                logger::info("  → Colt added!");
            }
        }
    }
}

fn do_add_money(player: &Player, dollars: i32) {
    let sign = if dollars >= 0 { "Adding" } else { "Removing" };
    logger::info(&format!("{sign} ${}", dollars.abs()));

    if player.add_money_dollars_with_hud(dollars) {
        log_balance(player);
    } else {
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

fn log_player_snapshot(player: &Player) {
    player.log_debug_info();

    if player.is_wallet_ready() {
        log_balance(player);
    } else {
        logger::info("Wallet not ready yet");
    }
}

fn monitor_loop() {
    logger::debug("[monitor] started");

    let mut last_state = state::get();
    let mut snapshot_done = false;

    loop {
        if runtime::is_shutting_down() {
            logger::debug("[monitor] stopping");
            break;
        }

        std::thread::sleep(Duration::from_secs(MONITOR_INTERVAL));

        let current = state::refresh_from_runtime();

        if current != last_state {
            match current {
                GameSessionState::InGame => {
                    if let Some(player) = Player::get_active() {
                        logger::info("[monitor] entered in-game state");
                        log_player_snapshot(&player);
                        snapshot_done = true;
                    }
                }
                GameSessionState::FrontendMenu => {
                    logger::info("[monitor] frontend/menu state");
                    snapshot_done = false;
                }
                GameSessionState::Loading => {
                    logger::info("[monitor] loading state");
                    snapshot_done = false;
                }
                GameSessionState::Paused => {
                    logger::info("[monitor] paused state");
                }
                GameSessionState::Boot => {
                    logger::info("[monitor] boot state");
                    snapshot_done = false;
                }
                GameSessionState::ShuttingDown => {}
            }

            last_state = current;
        }

        match current {
            GameSessionState::InGame => {
                if let Some(player) = Player::get_active() {
                    if !snapshot_done && player.is_ready() {
                        log_player_snapshot(&player);
                        snapshot_done = true;
                    }

                    let money = player
                        .get_money_formatted()
                        .unwrap_or_else(|| "wallet-not-ready".into());
                    logger::debug(&format!("[monitor] in-game | {money}"));
                } else {
                    logger::debug("[monitor] in-game but player pointer missing");
                }
            }
            GameSessionState::FrontendMenu => {
                logger::debug("[monitor] frontend/menu");
            }
            GameSessionState::Loading => {
                logger::debug("[monitor] loading");
            }
            GameSessionState::Paused => {
                logger::debug("[monitor] paused");
            }
            GameSessionState::Boot => {
                logger::debug("[monitor] boot");
            }
            GameSessionState::ShuttingDown => break,
        }
    }
}

fn dump_human_message_ids() {
    let items = [
        "enums.EventType.HUMAN",
        "enums.HumanMessages.DAMAGE",
        "enums.HumanMessages.DEATH",
        "enums.HumanMessages.ANIM_NOTIFY",
        "enums.HumanMessages.ENTER_VEHICLE",
        "enums.HumanMessages.LEAVE_VEHICLE",
        "enums.HumanMessages.ENTER_VEHICLE_DONE",
        "enums.HumanMessages.LEAVE_VEHICLE_DONE",
        "enums.HumanMessages.PLAYER_WEAPON_SELECT",
        "enums.HumanMessages.PLAYER_WEAPON_HIDE",
        "enums.HumanMessages.SHOT",
    ];

    for expr in items {
        let code = format!("return tostring({expr})");
        match sdk::game::lua::eval_chunk_named(&code, "=dump_human_message_ids") {
            Ok(Some(v)) => logger::info(&format!("[human-msg] {expr} = {v}")),
            Ok(None) => logger::warn(&format!("[human-msg] {expr} = <nil>")),
            Err(e) => logger::error(&format!("[human-msg] failed for {expr}: {e}")),
        }
    }
}