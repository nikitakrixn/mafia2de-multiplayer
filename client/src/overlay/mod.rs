//! Overlay — egui поверх D3D11.
//!
//! Единственная точка входа для рендера:
//! - `init()` — вызывается при старте клиента
//! - `render_frame()` — вызывается из Present1 хука каждый кадр

pub mod d3d11_state;
pub mod demo;
pub mod input;
pub mod input_lock;
pub mod state;
pub mod theme;
pub mod ui;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use common::logger;

static VISIBLE: AtomicBool = AtomicBool::new(true);
static RENDERING: AtomicBool = AtomicBool::new(false);
static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);

struct Inner {
    ctx: egui::Context,
    renderer: egui_directx11::Renderer,
    screen_w: u32,
    screen_h: u32,
    start_time: Instant,
    fps_counter: FpsCounter,
}

unsafe impl Send for Inner {}

struct FpsCounter {
    last_time: Instant,
    frame_count: u32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            last_time: Instant::now(),
            frame_count: 0,
        }
    }

    fn tick(&mut self) -> Option<f32> {
        self.frame_count += 1;
        let elapsed = self.last_time.elapsed().as_secs_f32();
        if elapsed >= 0.5 {
            let fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.last_time = Instant::now();
            Some(fps)
        } else {
            None
        }
    }
}

static OVERLAY: LazyLock<Mutex<Option<Inner>>> = LazyLock::new(|| Mutex::new(None));

/// Попытка инициализации. Если D3D11 не готов — вернёт Ok(false).
pub fn init() -> Result<(), String> {
    logger::info("[overlay] инициализация egui + D3D11");
    match try_init() {
        Ok(true) => {
            logger::info("[overlay] egui инициализирован");
            Ok(())
        }
        Ok(false) => {
            logger::info("[overlay] D3D11 не готов, отложим до первого Present");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Рендер кадра — вызывается из Present1 хука.
pub fn render_frame() {
    if !VISIBLE.load(Ordering::Relaxed) {
        return;
    }
    if !crate::utils::is_window_focused() {
        return;
    }

    // Защита от реентрантности
    if RENDERING.swap(true, Ordering::AcqRel) {
        return;
    }
    let _guard = scopeguard(|| RENDERING.store(false, Ordering::Release));

    let frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

    // Ленивая инициализация
    ensure_initialized(frame);

    // D3D11 ресурсы
    let Some(ctx_ptr) = sdk::game::render::get_d3d_context_ptr() else {
        return;
    };
    let Some(rtv_ptr) = sdk::game::render::get_backbuffer_rtv_ptr() else {
        return;
    };
    let Some((bb_w, bb_h)) = sdk::game::render::get_swapchain_size().filter(|&(w, h)| w > 0 && h > 0) else {
        return;
    };

    let mut guard = match OVERLAY.try_lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let Some(inner) = guard.as_mut() else {
        return;
    };

    // FPS
    if let Some(fps) = inner.fps_counter.tick() {
        state::set_fps(fps);
    }

    // Resize
    if bb_w != inner.screen_w || bb_h != inner.screen_h {
        logger::info(&format!(
            "[overlay] resize: {}x{} -> {bb_w}x{bb_h}",
            inner.screen_w, inner.screen_h
        ));
        inner.screen_w = bb_w;
        inner.screen_h = bb_h;
    }

    // Snapshot данных для UI
    let snap = state::snapshot();

    // Собираем ввод
    let hwnd = sdk::game::render::get_hwnd();
    let wants_input = state::wants_input();
    input_lock::tick(wants_input);
    let mut raw_input = input::collect(
        inner.screen_w as f32,
        inner.screen_h as f32,
        hwnd,
        wants_input,
    );
    raw_input.time = Some(inner.start_time.elapsed().as_secs_f64());

    // egui frame
    let full_output = inner.ctx.run(raw_input, |ctx| {
        ui::draw(ctx, &snap);
    });

    // D3D11 рендер
    let d3d_ctx = unsafe { d3d11_state::borrow_context(ctx_ptr) };
    let rtv = unsafe { d3d11_state::borrow_rtv(rtv_ptr) };
    let backup = unsafe { d3d11_state::StateBackup::save(&d3d_ctx) };

    let (renderer_output, _, _) = egui_directx11::split_output(full_output);

    if let Err(e) = inner.renderer.render(&*d3d_ctx, &*rtv, &inner.ctx, renderer_output) {
        if frame % 600 == 0 {
            logger::warn(&format!("[overlay] render error: {e:?}"));
        }
    }

    unsafe { backup.restore(&d3d_ctx) };
}

pub fn toggle_visibility() {
    let v = !VISIBLE.load(Ordering::Relaxed);
    VISIBLE.store(v, Ordering::Release);
    logger::info(&format!("[overlay] visible: {v}"));
}

fn try_init() -> Result<bool, String> {
    let mut guard = OVERLAY.lock().map_err(|e| format!("lock: {e}"))?;
    if guard.is_some() {
        return Ok(true);
    }

    let Some(dev_ptr) = sdk::game::render::get_d3d_device_ptr() else {
        return Ok(false);
    };

    let (w, h) = sdk::game::render::get_swapchain_size()
        .or_else(|| sdk::game::render::get_render_size())
        .unwrap_or((1920, 1080));

    let device = unsafe { d3d11_state::borrow_device(dev_ptr) };

    let renderer = egui_directx11::Renderer::new(&*device)
        .map_err(|e| format!("Renderer::new: {e:?}"))?;

    let ctx = egui::Context::default();
    theme::apply(&ctx);

    *guard = Some(Inner {
        ctx,
        renderer,
        screen_w: w,
        screen_h: h,
        start_time: Instant::now(),
        fps_counter: FpsCounter::new(),
    });

    logger::info(&format!("[overlay] ready ({w}x{h}, device=0x{dev_ptr:X})"));
    Ok(true)
}

fn ensure_initialized(frame: u64) {
    if OVERLAY.lock().map(|g| g.is_some()).unwrap_or(true) {
        return;
    }
    match try_init() {
        Ok(true) => logger::info("[overlay] deferred init OK"),
        Ok(false) => {}
        Err(e) => {
            if frame % 300 == 0 {
                logger::warn(&format!("[overlay] init error: {e}"));
            }
        }
    }
}

/// Простой scope guard без внешних зависимостей.
fn scopeguard<F: FnOnce()>(f: F) -> impl Drop {
    struct Guard<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Guard<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f();
            }
        }
    }
    Guard(Some(f))
}