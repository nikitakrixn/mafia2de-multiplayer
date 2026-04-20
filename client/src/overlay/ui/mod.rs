pub mod chat;
pub mod connect;
pub mod console;
pub mod cursor;
pub mod hud;
pub mod notifications;
pub mod player_list;
pub mod scoreboard;

use super::input;
use super::state::{self, Snapshot};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub fn draw(ctx: &egui::Context, snap: &Snapshot) {
    handle_hotkeys();

    if snap.show_debug {
        hud::draw(ctx, snap);
    }

    notifications::draw(ctx, snap);
    chat::draw(ctx, snap);

    if snap.show_connect {
        connect::draw(ctx, snap);
    }

    if snap.show_players {
        player_list::draw(ctx, snap);
    }

    if snap.show_scoreboard && !snap.show_connect {
        scoreboard::draw(ctx, snap);
    }

    if snap.show_console {
        console::draw(ctx, snap);
    }

    if snap.connection.connected {
        hud::draw_connection_badge(ctx, &snap.connection);
    }

    hud::draw_version(ctx);

    if state::wants_input() {
        cursor::draw(ctx);
    }
}

fn handle_hotkeys() {
    if input::just_pressed(VK_F2) {
        state::toggle_connect();
    }
    if input::just_pressed(VK_F3) {
        state::toggle_players();
    }
    if input::just_pressed(VK_F4) {
        state::toggle_console();
    }

    if input::just_pressed(VK_T) && !state::wants_input() {
        state::open_chat_input();
    }

    if input::just_pressed(VK_ESCAPE) {
        state::close_topmost();
    }

    let tab = input::is_held(VK_TAB) && !state::wants_input();
    state::set_scoreboard(tab);
}