// Состояние оверлея — единый источник данных для UI.
//
// Обновляется из game thread (sync_from_game), читается из render thread (snapshot).
// Все данные через атомики и try_lock — никогда не блокируем render thread.
//
// Таймеры обновления:
//   - Позиция игрока: каждые 500мс

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use sdk::game::Player;

// Интервалы обновления (миллисекунды)
const POS_UPDATE_INTERVAL_MS: u32 = 500;

/// Внутреннее состояние — thread-safe через атомики и Mutex.
pub struct OverlayState {
    // Видимость элементов
    pub show_debug: AtomicBool,

    // FPS и позиция (float как u32 bits для атомарности)
    pub fps: AtomicU32,
    pub pos_x: AtomicU32,
    pub pos_y: AtomicU32,
    pub pos_z: AtomicU32,

    // Уведомления (текст + время показа)
    pub notification: Mutex<Option<(String, Instant)>>,

    // Таймеры обновления (uptime в мс)
    last_pos_update_ms: AtomicU32,
}

static STATE: OnceLock<OverlayState> = OnceLock::new();
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Uptime в миллисекундах (для таймеров обновления).
fn uptime_ms() -> u32 {
    START_TIME.get_or_init(Instant::now).elapsed().as_millis() as u32
}

/// Ленивая инициализация состояния.
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

// =============================================================================
//  Публичный API
// =============================================================================

/// Обновить FPS (вызывается из render thread).
pub fn set_fps(fps: f32) {
    state().fps.store(fps.to_bits(), Ordering::Relaxed);
}

/// Переключить показ отладочной информации.
pub fn toggle_debug() {
    let s = state();
    let v = !s.show_debug.load(Ordering::Relaxed);
    s.show_debug.store(v, Ordering::Relaxed);
    common::logger::info(&format!("[overlay] debug: {v}"));
}

/// Показать уведомление (исчезает через 3 секунды).
pub fn show_notification(text: &str) {
    if let Ok(mut n) = state().notification.lock() {
        *n = Some((text.to_string(), Instant::now()));
    }
}

// =============================================================================
//  Синхронизация с игрой (вызывается из game thread)
// =============================================================================

/// Обновляет данные из игры — позицию игрока.
///
/// Вызывается каждый game tick из main_thread::on_main_thread_tick().
pub fn sync_from_game() {
    let s = state();
    let now = uptime_ms();

    // Проверяем наличие активного игрока
    let Some(player) = Player::get_active() else {
        return;
    };
    if !player.is_ready() {
        return;
    }

    // Позиция игрока (каждые 500мс)
    let last_pos = s.last_pos_update_ms.load(Ordering::Relaxed);
    if now.saturating_sub(last_pos) >= POS_UPDATE_INTERVAL_MS {
        if let Some(pos) = player.get_position() {
            s.pos_x.store(pos.x.to_bits(), Ordering::Relaxed);
            s.pos_y.store(pos.y.to_bits(), Ordering::Relaxed);
            s.pos_z.store(pos.z.to_bits(), Ordering::Relaxed);
            s.last_pos_update_ms.store(now, Ordering::Relaxed);
        }
    }
}

// =============================================================================
//  Снимок данных (вызывается из render thread)
// =============================================================================

/// Копия данных для рендера — можно передавать между потоками.
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

/// Создаёт атомарный снимок всех данных для render thread.
pub fn snapshot() -> Snapshot {
    let s = state();

    // Уведомление — показываем 3 секунды
    let notification = s
        .notification
        .lock()
        .ok()
        .and_then(|n| {
            n.as_ref()
                .filter(|(_, t)| t.elapsed().as_secs() < 3)
                .map(|(text, _)| text.clone())
        })
        .unwrap_or_default();

    let game_state = match crate::state::get() {
        crate::state::GameSessionState::Boot => "Загрузка",
        crate::state::GameSessionState::FrontendMenu => "Меню",
        crate::state::GameSessionState::Loading => "Загрузка миссии",
        crate::state::GameSessionState::InGame => "В игре",
        crate::state::GameSessionState::Paused => "Пауза",
        crate::state::GameSessionState::ShuttingDown => "Выход",
    }
    .to_string();

    Snapshot {
        show_debug: s.show_debug.load(Ordering::Relaxed),
        fps: f32::from_bits(s.fps.load(Ordering::Relaxed)),
        pos_x: f32::from_bits(s.pos_x.load(Ordering::Relaxed)),
        pos_y: f32::from_bits(s.pos_y.load(Ordering::Relaxed)),
        pos_z: f32::from_bits(s.pos_z.load(Ordering::Relaxed)),
        notification,
        game_state,
    }
}
