//! Единое состояние overlay.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use sdk::game::Player;

const NOTIFICATION_DURATION_SECS: u64 = 4;
const MAX_NOTIFICATIONS: usize = 5;
const NOTIFY_DEDUP_SECS: u64 = 2;

static SHOW_DEBUG: AtomicBool = AtomicBool::new(true);
static FPS: AtomicU32 = AtomicU32::new(0);
static POS_X: AtomicU32 = AtomicU32::new(0);
static POS_Y: AtomicU32 = AtomicU32::new(0);
static POS_Z: AtomicU32 = AtomicU32::new(0);

pub(crate) static SHOW_CONNECT: AtomicBool = AtomicBool::new(false);
pub(crate) static SHOW_PLAYERS: AtomicBool = AtomicBool::new(false);
pub(crate) static SHOW_SCOREBOARD: AtomicBool = AtomicBool::new(false);
pub(crate) static SHOW_CONSOLE: AtomicBool = AtomicBool::new(false);
pub(crate) static CHAT_INPUT_OPEN: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct ConnectionInfo {
    pub ip: String,
    pub port: String,
    pub nickname: String,
    pub connected: bool,
    pub status: String,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: "7788".into(),
            nickname: "Player".into(),
            connected: false,
            status: "Не подключен".into(),
        }
    }
}

#[derive(Clone)]
pub struct PlayerEntry {
    pub id: u32,
    pub name: String,
    pub ping: u32,
    pub is_local: bool,
}

#[derive(Clone)]
pub struct ChatMsg {
    pub author: String,
    pub text: String,
    pub time: String,
    pub system: bool,
    pub created: Instant,
}

#[derive(Clone)]
pub struct Notification {
    pub text: String,
    pub created: Instant,
}

#[derive(Clone)]
pub struct ConsoleEntry {
    pub id: u32,
    pub code: String,
    pub result: ConsoleResult,
    pub time: String,
}

#[derive(Clone)]
pub enum ConsoleResult {
    Queued,
    Ok,
    Error(String),
}

pub(crate) static CONNECTION: LazyLock<Mutex<ConnectionInfo>> =
    LazyLock::new(|| Mutex::new(ConnectionInfo::default()));

static PLAYERS: LazyLock<Mutex<Vec<PlayerEntry>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

static CHAT: LazyLock<Mutex<Vec<ChatMsg>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

static CHAT_INPUT: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::new()));

static NOTIFICATIONS: LazyLock<Mutex<Vec<Notification>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

static CONSOLE_ENTRIES: LazyLock<Mutex<Vec<ConsoleEntry>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

static CONSOLE_INPUT: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::new()));

static CONSOLE_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn set_fps(fps: f32) {
    FPS.store(fps.to_bits(), Ordering::Relaxed);
}

pub fn toggle_debug() {
    let v = !SHOW_DEBUG.load(Ordering::Relaxed);
    SHOW_DEBUG.store(v, Ordering::Relaxed);
}

pub fn notify(text: &str) {
    if let Ok(mut n) = NOTIFICATIONS.lock() {
        // Дедупликация: не добавлять одинаковый текст в течение DEDUP секунд
        if let Some(last) = n.last() {
            if last.text == text && last.created.elapsed().as_secs() < NOTIFY_DEDUP_SECS {
                return;
            }
        }
        n.push(Notification {
            text: text.to_string(),
            created: Instant::now(),
        });
        while n.len() > MAX_NOTIFICATIONS {
            n.remove(0);
        }
    }
}

pub fn toggle_connect() {
    let v = !SHOW_CONNECT.load(Ordering::Relaxed);
    SHOW_CONNECT.store(v, Ordering::Relaxed);
}

pub fn close_connect() {
    SHOW_CONNECT.store(false, Ordering::Relaxed);
}

pub fn toggle_players() {
    let v = !SHOW_PLAYERS.load(Ordering::Relaxed);
    SHOW_PLAYERS.store(v, Ordering::Relaxed);
}

pub fn set_scoreboard(visible: bool) {
    SHOW_SCOREBOARD.store(visible, Ordering::Relaxed);
}

pub fn toggle_console() {
    let v = !SHOW_CONSOLE.load(Ordering::Relaxed);
    SHOW_CONSOLE.store(v, Ordering::Relaxed);
}

pub fn open_chat_input() {
    CHAT_INPUT_OPEN.store(true, Ordering::Relaxed);
    super::input::flush();
}

pub fn close_chat_input() {
    CHAT_INPUT_OPEN.store(false, Ordering::Relaxed);
}

pub fn close_topmost() {
    if SHOW_CONSOLE.load(Ordering::Relaxed) {
        SHOW_CONSOLE.store(false, Ordering::Relaxed);
    } else if CHAT_INPUT_OPEN.load(Ordering::Relaxed) {
        close_chat_input();
    } else if SHOW_CONNECT.load(Ordering::Relaxed) {
        close_connect();
    } else if SHOW_PLAYERS.load(Ordering::Relaxed) {
        SHOW_PLAYERS.store(false, Ordering::Relaxed);
    }
}

pub fn wants_input() -> bool {
    SHOW_CONNECT.load(Ordering::Relaxed)
        || SHOW_PLAYERS.load(Ordering::Relaxed)
        || CHAT_INPUT_OPEN.load(Ordering::Relaxed)
        || SHOW_CONSOLE.load(Ordering::Relaxed)
}

pub fn set_connection(connected: bool, status: &str) {
    if let Ok(mut c) = CONNECTION.lock() {
        c.connected = connected;
        c.status = status.to_string();
    }
    notify(if connected { "Подключено к серверу" } else { status });
}

pub fn set_connection_status(connected: bool, status: String) {
    set_connection(connected, &status);
}

pub fn add_player(id: u32, name: String, ping: u32, is_local: bool) {
    if let Ok(mut p) = PLAYERS.lock() {
        if !p.iter().any(|e| e.id == id) {
            p.push(PlayerEntry { id, name, ping, is_local });
        }
    }
}

pub fn remove_player(id: u32) {
    if let Ok(mut p) = PLAYERS.lock() {
        p.retain(|e| e.id != id);
    }
}

pub fn update_ping(id: u32, ping: u32) {
    if let Ok(mut p) = PLAYERS.lock() {
        if let Some(e) = p.iter_mut().find(|e| e.id == id) {
            e.ping = ping;
        }
    }
}

pub fn update_player_ping(id: u32, ping: u32) {
    update_ping(id, ping);
}

pub fn clear_players() {
    if let Ok(mut p) = PLAYERS.lock() {
        p.clear();
    }
}

pub fn add_chat_msg(author: &str, text: &str) {
    if let Ok(mut msgs) = CHAT.lock() {
        msgs.push(ChatMsg {
            author: author.to_string(),
            text: text.to_string(),
            time: chrono::Local::now().format("%H:%M").to_string(),
            system: false,
            created: Instant::now(),
        });
        if msgs.len() > 100 { msgs.remove(0); }
    }
}

pub fn add_chat_message(author: String, text: String) {
    add_chat_msg(&author, &text);
}

pub fn add_system_msg(text: &str) {
    if let Ok(mut msgs) = CHAT.lock() {
        msgs.push(ChatMsg {
            author: String::new(),
            text: text.to_string(),
            time: chrono::Local::now().format("%H:%M").to_string(),
            system: true,
            created: Instant::now(),
        });
        if msgs.len() > 100 { msgs.remove(0); }
    }
    notify(text);
}

pub fn add_system_message(text: String) {
    add_system_msg(&text);
}

pub fn get_nickname() -> String {
    CONNECTION.lock().map(|c| c.nickname.clone()).unwrap_or_else(|_| "Player".into())
}

pub fn save_chat_input(text: &str) {
    if let Ok(mut s) = CHAT_INPUT.lock() { *s = text.to_string(); }
}

/// Отправить команду из консоли. Возвращает ID для отслеживания результата.
pub fn submit_console_command(code: &str) -> u32 {
    let id = CONSOLE_COUNTER.fetch_add(1, Ordering::Relaxed);

    if let Ok(mut entries) = CONSOLE_ENTRIES.lock() {
        entries.push(ConsoleEntry {
            id,
            code: code.to_string(),
            result: ConsoleResult::Queued,
            time: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        // Ограничиваем историю
        if entries.len() > 200 { entries.drain(0..50); }
    }

    // Ставим в очередь Lua с особым chunk_name
    crate::lua_queue::queue_exec_named(code, format!("=console:{id}"));

    id
}

/// Обновить результат команды консоли (вызывается из main_thread после исполнения).
pub fn update_console_result(id: u32, result: ConsoleResult) {
    if let Ok(mut entries) = CONSOLE_ENTRIES.lock() {
        if let Some(entry) = entries.iter_mut().find(|e| e.id == id) {
            entry.result = result;
        }
    }
}

pub fn save_console_input(text: &str) {
    if let Ok(mut s) = CONSOLE_INPUT.lock() { *s = text.to_string(); }
}

pub fn clear_console() {
    if let Ok(mut e) = CONSOLE_ENTRIES.lock() { e.clear(); }
}

pub fn sync_from_game() {
    let Some(player) = Player::get_active() else { return };
    if !player.is_ready() { return; }

    if let Some(pos) = player.get_position() {
        POS_X.store(pos.x.to_bits(), Ordering::Relaxed);
        POS_Y.store(pos.y.to_bits(), Ordering::Relaxed);
        POS_Z.store(pos.z.to_bits(), Ordering::Relaxed);
    }
}

pub fn sync_player_controls() {
    let should_lock = wants_input();
    let Some(player) = Player::get_active() else { return };
    if !player.is_ready() { return; }
    player.lock_controls(should_lock);
}

#[derive(Clone)]
pub struct Snapshot {
    pub show_debug: bool,
    pub fps: f32,
    pub pos: [f32; 3],
    pub game_state: &'static str,
    pub local_ping: u32,

    pub show_connect: bool,
    pub show_players: bool,
    pub show_scoreboard: bool,
    pub show_console: bool,
    pub chat_input_open: bool,

    pub connection: ConnectionInfo,
    pub players: Vec<PlayerEntry>,
    pub chat_msgs: Vec<ChatMsg>,
    pub chat_input: String,
    pub notifications: Vec<Notification>,

    pub console_entries: Vec<ConsoleEntry>,
    pub console_input: String,
}

pub fn snapshot() -> Snapshot {
    let connection = CONNECTION.lock().map(|c| c.clone()).unwrap_or_default();
    let players = PLAYERS.lock().map(|p| p.clone()).unwrap_or_default();
    let chat_msgs = CHAT.lock().map(|m| m.clone()).unwrap_or_default();
    let chat_input = CHAT_INPUT.lock().map(|s| s.clone()).unwrap_or_default();
    let console_entries = CONSOLE_ENTRIES.lock().map(|e| e.clone()).unwrap_or_default();
    let console_input = CONSOLE_INPUT.lock().map(|s| s.clone()).unwrap_or_default();

    let notifications = NOTIFICATIONS
        .lock()
        .map(|n| {
            n.iter()
                .filter(|n| n.created.elapsed().as_secs() < NOTIFICATION_DURATION_SECS)
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    if let Ok(mut n) = NOTIFICATIONS.lock() {
        n.retain(|n| n.created.elapsed().as_secs() < NOTIFICATION_DURATION_SECS);
    }

    let game_state = match crate::state::get() {
        crate::state::GameSessionState::Boot => "Загрузка",
        crate::state::GameSessionState::FrontendMenu => "Меню",
        crate::state::GameSessionState::Loading => "Загрузка миссии",
        crate::state::GameSessionState::InGame => "В игре",
        crate::state::GameSessionState::Paused => "Пауза",
        crate::state::GameSessionState::ShuttingDown => "Выход",
    };

    let local_ping = players
        .iter()
        .find(|p| p.is_local)
        .map(|p| p.ping)
        .unwrap_or(0);

    Snapshot {
        show_debug: SHOW_DEBUG.load(Ordering::Relaxed),
        fps: f32::from_bits(FPS.load(Ordering::Relaxed)),
        pos: [
            f32::from_bits(POS_X.load(Ordering::Relaxed)),
            f32::from_bits(POS_Y.load(Ordering::Relaxed)),
            f32::from_bits(POS_Z.load(Ordering::Relaxed)),
        ],
        game_state,
        local_ping,
        show_connect: SHOW_CONNECT.load(Ordering::Relaxed),
        show_players: SHOW_PLAYERS.load(Ordering::Relaxed),
        show_scoreboard: SHOW_SCOREBOARD.load(Ordering::Relaxed),
        show_console: SHOW_CONSOLE.load(Ordering::Relaxed),
        chat_input_open: CHAT_INPUT_OPEN.load(Ordering::Relaxed),
        connection, players, chat_msgs, chat_input, notifications,
        console_entries, console_input,
    }
}