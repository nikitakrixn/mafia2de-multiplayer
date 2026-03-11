//! Камерная система — чтение и управление FOV.
//!
//! CameraManager — статический объект в .bss секции игры.
//! Содержит два PlayerCameraView (Interier/Exterier),
//! каждый из которых хранит 15 state-блоков с параметрами.
//!
//! Параметр FOV — индекс 4 в массиве из 27 float параметров.
//! Типичные значения: 65.0 (interier), 75.0 (exterier).
//!
//! Подтверждение из реверса:
//! - Строковая таблица параметров: 0x1418ED230 ("Distance", ..., "Fov" @ index 4, ...)
//! - Загрузка: sub_140E7D020 (CameraView_ParseFromXML)
//! - Глобальный объект: 0x1431430F0 (static, NOT pointer)
//! - Вызов: sub_141008230 → sub_140E767E0 → sub_140E75CC0 → sub_140E7D020

use common::logger;
use crate::{addresses, memory};
use super::base;

use addresses::fields::{camera_manager as cm, camera_view as cv};
use addresses::constants::camera_params;

// ═══════════════════════════════════════════════════════════════════
//  Внутренние хелперы
// ═══════════════════════════════════════════════════════════════════

/// Адрес статического объекта CameraManager.
///
/// ⚠️ Это сам объект, НЕ указатель. Не разыменовывать!
fn camera_mgr() -> usize {
    base() + addresses::globals::CAMERA_MANAGER
}

/// Абсолютный адрес DefaultFOV для заданного view.
fn default_fov_addr(view_offset: usize) -> usize {
    camera_mgr() + view_offset + cv::DEFAULT_PARAMS + camera_params::FOV * 4
}

/// Абсолютный адрес State\[n\] FOV для заданного view.
fn state_fov_addr(view_offset: usize, state: usize) -> usize {
    camera_mgr()
        + view_offset
        + cv::STATES_BASE
        + state * cv::STATE_STRIDE
        + cv::STATE_PARAMS_OFFSET
        + camera_params::FOV * 4
}

/// Абсолютный адрес State\[n\] FOV flag для заданного view.
fn state_fov_flag_addr(view_offset: usize, state: usize) -> usize {
    camera_mgr()
        + view_offset
        + cv::STATES_BASE
        + state * cv::STATE_STRIDE
        + cv::STATE_PARAM_FLAGS_OFFSET
        + camera_params::FOV
}

fn is_valid_fov(fov: f32) -> bool {
    fov > 0.0 && fov <= 180.0 && fov.is_finite()
}

// ═══════════════════════════════════════════════════════════════════
//  Public API
// ═══════════════════════════════════════════════════════════════════

/// Проверяет, загружен ли камерный конфиг.
///
/// Читает Interier DefaultFOV — если это валидный float (1..180),
/// значит playerCamera.xml был распарсен.
pub fn is_initialized() -> bool {
    let fov = unsafe {
        memory::read_value::<f32>(default_fov_addr(cm::INTERIER_VIEW))
    };
    matches!(fov, Some(v) if is_valid_fov(v))
}

/// Прочитать текущий Interier default FOV.
pub fn get_interier_fov() -> Option<f32> {
    if !is_initialized() { return None; }
    unsafe { memory::read_value::<f32>(default_fov_addr(cm::INTERIER_VIEW)) }
}

/// Прочитать текущий Exterier default FOV.
pub fn get_exterier_fov() -> Option<f32> {
    if !is_initialized() { return None; }
    unsafe { memory::read_value::<f32>(default_fov_addr(cm::EXTERIER_VIEW)) }
}

/// Прочитать FOV конкретного state (0..14) для заданного view.
pub fn get_state_fov(view_offset: usize, state: usize) -> Option<f32> {
    if !is_initialized() || state >= cv::NUM_STATES { return None; }
    unsafe { memory::read_value::<f32>(state_fov_addr(view_offset, state)) }
}

/// Установить FOV для одного view (Interier или Exterier).
///
/// Пишет в default И во все 15 state-слотов.
/// Очищает FOV-флаг каждого state в 0 (= "есть override").
fn set_view_fov(view_offset: usize, fov: f32) -> bool {
    // Default FOV
    unsafe {
        if !memory::write_value(default_fov_addr(view_offset), fov) {
            return false;
        }
    }

    // All 15 states
    for n in 0..cv::NUM_STATES {
        unsafe {
            memory::write_value(state_fov_addr(view_offset, n), fov);
            memory::write_value::<u8>(state_fov_flag_addr(view_offset, n), 0);
        }
    }

    true
}

/// Установить FOV для обоих view (Interier + Exterier).
///
/// Пишет:
/// - DefaultFOV для обоих view
/// - FOV во все 15 per-state слотов обоих view
/// - Очищает per-state FOV flags (0 = override active)
///
/// Возвращает false если камера не инициализирована
/// или значение невалидно.
pub fn set_player_fov(fov: f32) -> bool {
    if !is_initialized() {
        logger::warn("[camera] не инициализирована, FOV не установлен");
        return false;
    }

    if !is_valid_fov(fov) {
        logger::warn(&format!("[camera] невалидный FOV: {fov}"));
        return false;
    }

    let ok_int = set_view_fov(cm::INTERIER_VIEW, fov);
    let ok_ext = set_view_fov(cm::EXTERIER_VIEW, fov);

    if ok_int && ok_ext {
        logger::info(&format!("[camera] FOV установлен: {fov}"));
        true
    } else {
        logger::error("[camera] ошибка записи FOV");
        false
    }
}

/// Установить FOV раздельно для Interier и Exterier.
pub fn set_player_fov_separate(interier_fov: f32, exterier_fov: f32) -> bool {
    if !is_initialized() { return false; }
    if !is_valid_fov(interier_fov) || !is_valid_fov(exterier_fov) { return false; }

    let ok_int = set_view_fov(cm::INTERIER_VIEW, interier_fov);
    let ok_ext = set_view_fov(cm::EXTERIER_VIEW, exterier_fov);

    ok_int && ok_ext
}

/// Прочитать произвольный camera param по индексу из DefaultParams.
///
/// Индексы — см. `constants::camera_params`.
pub fn get_default_param(view_offset: usize, param_index: usize) -> Option<f32> {
    if !is_initialized() || param_index >= cv::NUM_PARAMS { return None; }
    let addr = camera_mgr() + view_offset + cv::DEFAULT_PARAMS + param_index * 4;
    unsafe { memory::read_value::<f32>(addr) }
}

/// Записать произвольный camera param в DefaultParams + все states.
pub fn set_default_param(view_offset: usize, param_index: usize, value: f32) -> bool {
    if !is_initialized() || param_index >= cv::NUM_PARAMS { return false; }

    let mgr = camera_mgr();

    // Write default
    let default_addr = mgr + view_offset + cv::DEFAULT_PARAMS + param_index * 4;
    unsafe { memory::write_value(default_addr, value); }

    // Write all states
    for n in 0..cv::NUM_STATES {
        let state_base = mgr + view_offset + cv::STATES_BASE + n * cv::STATE_STRIDE;
        let param_addr = state_base + cv::STATE_PARAMS_OFFSET + param_index * 4;
        let flag_addr = state_base + cv::STATE_PARAM_FLAGS_OFFSET + param_index;
        unsafe {
            memory::write_value(param_addr, value);
            memory::write_value::<u8>(flag_addr, 0);
        }
    }

    true
}

// ═══════════════════════════════════════════════════════════════════
//  Диагностика
// ═══════════════════════════════════════════════════════════════════

/// Вывести текущее состояние камерной системы в лог.
pub fn log_status() {
    if !is_initialized() {
        logger::info("[camera] не инициализирована (конфиг не загружен)");
        return;
    }

    let int_fov = get_interier_fov().unwrap_or(0.0);
    let ext_fov = get_exterier_fov().unwrap_or(0.0);

    logger::info(&format!(
        "[camera] FOV: interier={int_fov:.1}, exterier={ext_fov:.1}"
    ));

    // Показать Distance для контекста
    let int_dist = get_default_param(cm::INTERIER_VIEW, camera_params::DISTANCE).unwrap_or(0.0);
    let ext_dist = get_default_param(cm::EXTERIER_VIEW, camera_params::DISTANCE).unwrap_or(0.0);
    logger::info(&format!(
        "[camera] Distance: interier={int_dist:.1}, exterier={ext_dist:.1}"
    ));

    // Первый state FOV для проверки sync
    let s0_int = get_state_fov(cm::INTERIER_VIEW, 0).unwrap_or(0.0);
    let s0_ext = get_state_fov(cm::EXTERIER_VIEW, 0).unwrap_or(0.0);
    logger::debug(&format!(
        "[camera] State[0] FOV: interier={s0_int:.1}, exterier={s0_ext:.1}"
    ));
}