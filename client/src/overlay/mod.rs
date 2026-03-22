pub mod egui_input;
pub mod state;
pub mod state_backup;
pub mod ui_new;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use common::logger;

// Флаги видимости и рендера
static VISIBLE: AtomicBool = AtomicBool::new(true);
static RENDERING: AtomicBool = AtomicBool::new(false);
static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);

// Внутреннее состояние оверлея
struct OverlayInner {
    egui_ctx: egui::Context,
    egui_renderer: egui_directx11::Renderer,
    screen_w: u32,
    screen_h: u32,
    last_fps_time: Instant,
    fps_frame_count: u32,
    start_time: Instant,
}

unsafe impl Send for OverlayInner {}

static OVERLAY: OnceLock<Mutex<OverlayInner>> = OnceLock::new();

// Пытаемся инициализировать оверлей
// Может быть вызвана несколько раз — если D3D11 ещё не готов, вернём Ok(false)
fn try_init() -> Result<bool, String> {
    // Уже инициализирован?
    if OVERLAY.get().is_some() {
        return Ok(true);
    }

    // Проверяем готовность D3D11
    let dev_ptr = match sdk::game::render::get_d3d_device_ptr() {
        Some(p) => p,
        None => return Ok(false), // ещё не готов
    };

    let (width, height) = sdk::game::render::get_swapchain_size()
        .or_else(|| sdk::game::render::get_render_size())
        .unwrap_or((1280, 720));

    let device = unsafe { state_backup::borrow_device(dev_ptr) };

    // Создаём egui renderer
    let egui_renderer = egui_directx11::Renderer::new(&*device)
        .map_err(|e| format!("egui Renderer::new: {e:?}"))?;

    let egui_ctx = egui::Context::default();

    // Стиль — тёмный, полностью прозрачный фон
    let mut style = (*egui_ctx.style()).clone();
    style.visuals.window_fill = egui::Color32::TRANSPARENT;
    style.visuals.panel_fill = egui::Color32::TRANSPARENT;
    style.visuals.window_shadow = egui::Shadow::NONE;
    style.visuals.popup_shadow = egui::Shadow::NONE;
    style.visuals.window_stroke = egui::Stroke::NONE;
    egui_ctx.set_style(style);

    let _ = OVERLAY.set(Mutex::new(OverlayInner {
        egui_ctx,
        egui_renderer,
        screen_w: width,
        screen_h: height,
        last_fps_time: Instant::now(),
        fps_frame_count: 0,
        start_time: Instant::now(),
    }));

    logger::info(&format!(
        "[overlay] egui инициализирован ({width}x{height}, device=0x{dev_ptr:X})"
    ));
    Ok(true)
}

// Инициализация оверлея — вызывается из lib.rs при старте
// Если D3D11 не готов — ничего страшного, доинициализируем позже
pub fn init() -> Result<(), String> {
    logger::info("[overlay] инициализация egui + D3D11");
    match try_init() {
        Ok(true) => Ok(()),
        Ok(false) => {
            logger::info("[overlay] D3D11 ещё не готов, доинициализируем при первом Present1");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// Рендер кадра — вызывается из Present1 хука каждый кадр
pub fn render_frame() {
    if !VISIBLE.load(Ordering::Relaxed) {
        return;
    }

    // Не рендерим если окно не в фокусе
    if !is_window_focused() {
        return;
    }

    // Защита от реентрантности
    if RENDERING.swap(true, Ordering::AcqRel) {
        return;
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            RENDERING.store(false, Ordering::Release);
        }
    }
    let _g = Guard;

    let frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

    // Ленивая инициализация — если ещё не создан renderer
    if OVERLAY.get().is_none() {
        match try_init() {
            Ok(true) => {
                logger::info("[overlay] отложенная инициализация успешна");
            }
            Ok(false) => {
                // D3D11 всё ещё не готов
                return;
            }
            Err(e) => {
                // Логируем ошибку раз в 300 кадров
                if frame % 300 == 0 {
                    logger::warn(&format!("[overlay] ошибка инициализации: {e}"));
                }
                return;
            }
        }
    }

    // Проверяем D3D11 ресурсы
    let Some(ctx_ptr) = sdk::game::render::get_d3d_context_ptr() else {
        return;
    };
    let Some(rtv_ptr) = sdk::game::render::get_backbuffer_rtv_ptr() else {
        return;
    };

    let (bb_w, bb_h) = match sdk::game::render::get_swapchain_size() {
        Some(s) if s.0 > 0 && s.1 > 0 => s,
        _ => return,
    };

    let mut guard = match OVERLAY.get() {
        Some(m) => match m.try_lock() {
            Ok(g) => g,
            Err(_) => return,
        },
        None => return,
    };
    let inner = &mut *guard;

    // Считаем FPS
    inner.fps_frame_count += 1;
    let elapsed = inner.last_fps_time.elapsed();
    if elapsed.as_secs_f32() >= 2.0 {
        state::set_fps(inner.fps_frame_count as f32 / elapsed.as_secs_f32());
        inner.fps_frame_count = 0;
        inner.last_fps_time = Instant::now();
    }

    // Обрабатываем изменение разрешения
    if bb_w != inner.screen_w || bb_h != inner.screen_h {
        logger::info(&format!(
            "[overlay] изменение разрешения: {}x{} -> {bb_w}x{bb_h}",
            inner.screen_w, inner.screen_h
        ));
        inner.screen_w = bb_w;
        inner.screen_h = bb_h;
    }

    // Снимок данных для UI
    let snap = state::snapshot();

    // Собираем ввод
    let hwnd = sdk::game::render::get_hwnd();
    let mut raw_input = egui_input::collect_raw_input(
        inner.screen_w as f32,
        inner.screen_h as f32,
        hwnd,
        false, // пока не нужен захват мыши
    );
    raw_input.time = Some(inner.start_time.elapsed().as_secs_f64());

    // Запускаем egui frame
    let full_output = inner.egui_ctx.run(raw_input, |ctx| {
        ui_new::draw_overlay(ctx, &snap);
    });

    // Рендерим в D3D11 backbuffer
    let ctx_d3d = unsafe { state_backup::borrow_context(ctx_ptr) };
    let rtv = unsafe { state_backup::borrow_rtv(rtv_ptr) };

    // Сохраняем состояние игрового pipeline
    let backup = unsafe { state_backup::D3D11StateBackup::save(&ctx_d3d) };

    let (renderer_output, _platform_output, _viewport_output) =
        egui_directx11::split_output(full_output);

    // Рендерим egui
    if let Err(e) = inner
        .egui_renderer
        .render(&*ctx_d3d, &*rtv, &inner.egui_ctx, renderer_output)
    {
        if frame % 600 == 0 {
            logger::warn(&format!("[overlay] ошибка рендера: {e:?}"));
        }
    }

    // Восстанавливаем состояние игрового pipeline
    unsafe { backup.restore(&ctx_d3d) };
}

pub fn toggle_visibility() {
    let v = !VISIBLE.load(Ordering::Relaxed);
    VISIBLE.store(v, Ordering::Release);
    logger::info(&format!("[overlay] видимость: {v}"));
}

// Проверяет, находится ли окно игры в фокусе
fn is_window_focused() -> bool {
    crate::utils::is_window_focused()
}
