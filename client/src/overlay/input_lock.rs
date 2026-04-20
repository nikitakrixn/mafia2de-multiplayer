//! Пауза игрового input-subsystem на время открытого overlay-меню.
//!
//! Когда у нас открыт чат / список игроков / окно подключения, игроку не
//! должны идти ни мышиные клики, ни WASD, ни поворот камеры. Делаем это
//! через [`GameInputModule::pause_input`]: внутри движок поднимает
//! `m_b_input_paused`, после чего `Tick` пропускает весь
//! `C_GameInput::Update`, плюс паузит сами DI-устройства, ресетит
//! `C_InputLayer`-ы и Force Feedback.
//!
//! Дёргаем функцию только на переходах состояния — она идемпотентна,
//! но лишний раз тревожить движок незачем.

use std::sync::atomic::{AtomicBool, Ordering};

use common::logger;
use sdk::game::GameInputModule;

static PAUSED_BY_OVERLAY: AtomicBool = AtomicBool::new(false);

/// Вызывать каждый кадр overlay перед сбором ввода.
pub fn tick(wants_input: bool) {
    let current = PAUSED_BY_OVERLAY.load(Ordering::Relaxed);
    if wants_input == current {
        return;
    }

    let Some(module) = GameInputModule::get() else {
        // Движок ещё не сконструировал singleton (mid-injection или
        // рендер до WinMain). Попробуем на следующем тике.
        return;
    };

    if module.pause_input(wants_input) {
        PAUSED_BY_OVERLAY.store(wants_input, Ordering::Relaxed);
        logger::debug(&format!("[input-lock] pause_input({wants_input}) ok"));
    }
}
