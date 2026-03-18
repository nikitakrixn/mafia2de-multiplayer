// Состояние оверлея — единый источник правды для UI
//
// Тут храним всё что нужно показывать в интерфейсе:
// - FPS и позицию игрока
// - Уведомления
// - Флаги видимости

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use sdk::game::Player;

// Как часто обновлять позицию (в миллисекундах)
const POS_UPDATE_INTERVAL_MS: u64 = 500;

pub struct OverlayState {
    // Видимость элементов
    pub show_debug: AtomicBool,
    
    // Данные для отладки
    pub fps: AtomicU32,
    pub pos_x: AtomicU32,
    pub pos_y: AtomicU32,
    pub pos_z: AtomicU32,
    
    // Уведомления
    pub notification: Mutex<Option<(String, Instant)>>,
    
    // Время последнего обновления позиции
    last_pos_update_ms: AtomicU32,
}

static STATE: OnceLock<OverlayState> = OnceLock::new();
static START_TIME: OnceLock<Instant> = OnceLock::new();

fn uptime_ms() -> u32 {
    START_TIME
        .get_or_init(Instant::now)
        .elapsed()
        .as_millis() as u32
}

fn state() -> &'static OverlayState {
    STATE.get_or_init(|| OverlayState {
        show_debug: AtomicBool::new(true),
        fps: AtomicU32::new(0f32.to_bits()),
        pos_x: AtomicU32::new(0f32.to_bits()),
        pos_y: AtomicU32::new(0f32.to_bits()),
        pos_z: AtomicU32::new(0f32.to_bits()),
        notification: Mutex::new(None),
        last_pos_update_ms: AtomicU32::new(0),
    })
}

// ═══════════════════════════════════════════════════════
//  Публичный API
// ═══════════════════════════════════════════════════════

pub fn set_fps(fps: f32) {
    state().fps.store(fps.to_bits(), Ordering::Relaxed);
}

pub fn toggle_debug() {
    let s = state();
    let new_val = !s.show_debug.load(Ordering::Relaxed);
    s.show_debug.store(new_val, Ordering::Relaxed);
    common::logger::info(&format!("[overlay] debug: {new_val}"));
}

pub fn show_notification(text: &str) {
    if let Ok(mut n) = state().notification.lock() {
        *n = Some((text.to_string(), Instant::now()));
    }
}

// Обновляем данные из игры (вызывается из main thread)
pub fn sync_from_game() {
    let s = state();
    
    // Берём данные с игрока если он есть
    let Some(player) = Player::get_active() else {
        return;
    };
    
    if !player.is_ready() {
        return;
    }
    
    // Позицию обновляем не чаще чем раз в 500мс
    let now = uptime_ms();
    let last = s.last_pos_update_ms.load(Ordering::Relaxed);
    
    if now.saturating_sub(last) >= POS_UPDATE_INTERVAL_MS as u32 {
        if let Some(pos) = player.get_position() {
            s.pos_x.store(pos.x.to_bits(), Ordering::Relaxed);
            s.pos_y.store(pos.y.to_bits(), Ordering::Relaxed);
            s.pos_z.store(pos.z.to_bits(), Ordering::Relaxed);
            s.last_pos_update_ms.store(now, Ordering::Relaxed);
        }
    }
}

// Снимок данных для рендера (вызывается из render thread)
#[derive(Clone)]
pub struct Snapshot {
    pub show_debug: bool,
    pub fps: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub notification: String,
    pub game_state: String,
}

pub fn snapshot() -> Snapshot {
    let s = state();
    
    let show_debug = s.show_debug.load(Ordering::Relaxed);
    let fps = f32::from_bits(s.fps.load(Ordering::Relaxed));
    let pos_x = f32::from_bits(s.pos_x.load(Ordering::Relaxed));
    let pos_y = f32::from_bits(s.pos_y.load(Ordering::Relaxed));
    let pos_z = f32::from_bits(s.pos_z.load(Ordering::Relaxed));
    
    // Уведомление показываем только 3 секунды
    let notification = s.notification.lock().ok()
        .and_then(|n| n.as_ref()
            .filter(|(_, time)| time.elapsed().as_secs() < 3)
            .map(|(text, _)| text.clone()))
        .unwrap_or_default();
    
    let game_state = match crate::state::get() {
        crate::state::GameSessionState::Boot => "Загрузка",
        crate::state::GameSessionState::FrontendMenu => "Меню",
        crate::state::GameSessionState::Loading => "Загрузка миссии",
        crate::state::GameSessionState::InGame => "В игре",
        crate::state::GameSessionState::Paused => "Пауза",
        crate::state::GameSessionState::ShuttingDown => "Выход",
    }.to_string();
    
    Snapshot {
        show_debug,
        fps,
        pos_x,
        pos_y,
        pos_z,
        notification,
        game_state,
    }
}
