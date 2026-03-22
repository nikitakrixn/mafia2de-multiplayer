//! Камерная система — чтение и управление FOV.
//!
//! # Архитектура камерной системы M2DE
//!
//! `CameraManager` — статический объект в `.bss` секции (НЕ указатель!).
//! Глобал: `globals::CAMERA_MANAGER` (IDA: `0x1431430F0`).
//!
//! Содержит:
//! - **PlayerCamera** — два `CameraView` (Interier/Exterier), каждый по 0xD18 байт
//! - **Car cameras** — плоские float-массивы параметров для каждого типа камеры
//! - **Misc cameras** — fpv, death, meelee
//!
//! ## PlayerCameraView (0xD18 байт)
//!
//! Каждый view хранит 15 state-блоков (Stay, Walk, Run, Sprint, ...)
//! и массивы дефолтных значений:
//!
//! ```text
//! +0x0004: State[0..14]       — 15 × 0xD4 байт
//! +0x0C70: DefaultParams[27]  — 27 float (Distance, MinDistance, ..., Fov=index 4, ...)
//! +0x0CDC: DefaultSpeeds[15]  — 15 float
//! ```
//!
//! Каждый state-блок (0xD4 = 212 байт):
//! ```text
//! +0x00: Params[27]      float[27]   — значения параметров
//! +0x6C: Speeds[15]      float[15]   — скорости переходов
//! +0xA8: ParamFlags[27]  byte[27]    — 0 = есть override, 1 = брать из default
//! +0xC3: SpeedFlags[15]  byte[15]
//! ```
//!
//! ## Car cameras
//!
//! Автомобильные камеры хранят параметры в плоских float-массивах
//! (без per-state блоков). Индекс FOV зависит от типа камеры:
//!
//! | Камера             | Params base | FOV index | FovMax index |
//! |--------------------|-------------|-----------|--------------|
//! | carCameraBumper     | +0x21D8     | 0         | —            |
//! | carCameraWheel      | +0x21E8     | 0         | —            |
//! | carCameraHood       | +0x21F8     | 0         | —            |
//! | carCameraLookback   | +0x230C     | 0         | —            |
//! | carCameraDynamic    | +0x2244     | 11        | 16           |
//! | carCameraDynamicLong| +0x22A8     | 11        | 16           |
//! | carCameraShoot      | +0x2208     | 3         | —            |
//! | carCameraGamepad    | +0x231C     | 10        | 14           |
//! | fpvCamera           | +0x275C     | 0         | —            |
//! | deathCamera         | +0x277C     | 0         | —            |
//! | meeleeCamera        | +0x2784     | 5         | —            |
//!
//! FovMax — дельта, которая прибавляется к базовому Fov на высокой скорости.
//! При принудительной установке FOV обнуляем, чтобы FOV не «уплывал».
//!
//! ## Подтверждение из реверса
//!
//! - Строковая таблица playerCamera: `0x1418ED230` (27 записей × 0x23 байт, "Fov" @ index 4)
//! - Строковая таблица carCameraDynamic: `0x1418EE3A0` (25 записей × 0x20, "Fov" @ index 11)
//! - Строковая таблица carCameraShoot: `0x1418EE190` (15 записей × 0x20, "Fov" @ index 3)
//! - Строковая таблица carCameraGamepad: `0x1418EE720` (24 записей × 0x20, "Fov" @ index 10)
//! - Парсер: `sub_140E7D020` (`M2DE_CameraView_ParseFromXML`)
//! - Инициализация: `sub_141008230` -> `sub_140E767E0` -> `sub_140E75CC0` -> `sub_140E7D020`
//! - Runtime скан подтвердил все индексы (значения 61..72 в ожидаемых позициях)

use super::base;
use crate::{addresses, memory};
use common::logger;

use addresses::constants::camera_params;
use addresses::fields::{camera_manager as cm, camera_view as cv};

// =============================================================================
//  Внутренние хелперы
// =============================================================================

/// Адрес статического объекта CameraManager в памяти процесса.
///
/// ⚠️ Это сам объект, НЕ указатель. Не разыменовывать!
fn camera_mgr() -> usize {
    base() + addresses::globals::CAMERA_MANAGER
}

/// Абсолютный адрес DefaultFOV для заданного view (Interier/Exterier).
///
/// `view_offset` = `cm::INTERIER_VIEW` (0x0) или `cm::EXTERIER_VIEW` (0xD18).
fn default_fov_addr(view_offset: usize) -> usize {
    camera_mgr() + view_offset + cv::DEFAULT_PARAMS + camera_params::FOV * 4
}

/// Абсолютный адрес State\[n\] FOV для заданного view.
///
/// Формула: `mgr + view + 0x04 + n * 0xD4 + FOV_INDEX * 4`
fn state_fov_addr(view_offset: usize, state: usize) -> usize {
    camera_mgr()
        + view_offset
        + cv::STATES_BASE
        + state * cv::STATE_STRIDE
        + cv::STATE_PARAMS_OFFSET
        + camera_params::FOV * 4
}

/// Абсолютный адрес State\[n\] FOV flag для заданного view.
///
/// Flag: 0 = есть override-значение, 1 = использовать default.
/// При записи FOV в state нужно выставить flag в 0.
fn state_fov_flag_addr(view_offset: usize, state: usize) -> usize {
    camera_mgr()
        + view_offset
        + cv::STATES_BASE
        + state * cv::STATE_STRIDE
        + cv::STATE_PARAM_FLAGS_OFFSET
        + camera_params::FOV
}

/// Валидация значения FOV.
///
/// Игра использует значения 50–95, но мы допускаем расширенный диапазон
/// для экспериментов. Защита от NaN / Inf / отрицательных.
fn is_valid_fov(fov: f32) -> bool {
    fov > 0.0 && fov <= 180.0 && fov.is_finite()
}

// =============================================================================
//  Все известные FOV-оффсеты
// =============================================================================

/// Все оффсеты базового FOV (абсолютный угол обзора).
///
/// Для каждого: `CameraManager + offset` -> `float`.
/// Подтверждено runtime-сканом и строковыми таблицами из IDA.
const ALL_FOV_OFFSETS: &[usize] = &[
    // Простые car cameras (FOV = params[0])
    cm::CAR_BUMPER_FOV,   // carCameraBumper
    cm::CAR_WHEEL_FOV,    // carCameraWheel
    cm::CAR_HOOD_FOV,     // carCameraHood
    cm::CAR_LOOKBACK_FOV, // carCameraLookback
    // Сложные car cameras (FOV по вычисленному индексу)
    cm::CAR_DYNAMIC_FOV,      // carCameraDynamic, index 11
    cm::CAR_DYNAMIC_LONG_FOV, // carCameraDynamicLong, index 11
    cm::CAR_SHOOT_FOV,        // carCameraShoot, index 3
    cm::CAR_GAMEPAD_FOV,      // carCameraGamepad, index 10
    // Остальные камеры
    cm::FPV_FOV,    // fpvCamera
    cm::DEATH_FOV,  // deathCamera
    cm::MEELEE_FOV, // meeleeCamera, index 5
];

/// Оффсеты FovMax — дельта FOV на высокой скорости.
///
/// FovMax прибавляется к базовому Fov пропорционально скорости.
/// При принудительной установке FOV обнуляем, чтобы камера
/// не выходила за целевое значение на скорости.
const FOV_MAX_OFFSETS: &[usize] = &[
    cm::CAR_DYNAMIC_FOV_MAX,      // carCameraDynamic, index 16, default ~10
    cm::CAR_DYNAMIC_LONG_FOV_MAX, // carCameraDynamicLong, index 16, default ~15
    cm::CAR_GAMEPAD_FOV_MAX,      // carCameraGamepad, index 14, default ~20
];

// =============================================================================
//  Чтение текущего состояния
// =============================================================================

/// Проверяет, загружен ли камерный конфиг (playerCamera.xml распарсен).
///
/// Читает Interier DefaultFOV — если это валидный float,
/// значит `M2DE_CameraManager_LoadPlayerCamera` уже отработал.
pub fn is_initialized() -> bool {
    let fov = unsafe { memory::read_value::<f32>(default_fov_addr(cm::INTERIER_VIEW)) };
    matches!(fov, Some(v) if is_valid_fov(v))
}

/// Текущий Interier default FOV (камера в помещении / по умолчанию).
pub fn get_interier_fov() -> Option<f32> {
    if !is_initialized() {
        return None;
    }
    unsafe { memory::read_value::<f32>(default_fov_addr(cm::INTERIER_VIEW)) }
}

/// Текущий Exterier default FOV (камера на улице).
pub fn get_exterier_fov() -> Option<f32> {
    if !is_initialized() {
        return None;
    }
    unsafe { memory::read_value::<f32>(default_fov_addr(cm::EXTERIER_VIEW)) }
}

/// FOV конкретного state (0..14) для заданного view.
///
/// `view_offset` = `cm::INTERIER_VIEW` или `cm::EXTERIER_VIEW`.
/// State 0 = Stay, 1 = Walk, 2 = Run, и т.д.
pub fn get_state_fov(view_offset: usize, state: usize) -> Option<f32> {
    if !is_initialized() || state >= cv::NUM_STATES {
        return None;
    }
    unsafe { memory::read_value::<f32>(state_fov_addr(view_offset, state)) }
}

/// Произвольный параметр камеры из DefaultParams по индексу.
///
/// Индексы — см. `constants::camera_params` (0 = Distance, 4 = FOV, и т.д.).
pub fn get_default_param(view_offset: usize, param_index: usize) -> Option<f32> {
    if !is_initialized() || param_index >= cv::NUM_PARAMS {
        return None;
    }
    let addr = camera_mgr() + view_offset + cv::DEFAULT_PARAMS + param_index * 4;
    unsafe { memory::read_value::<f32>(addr) }
}

// =============================================================================
//  Запись FOV — Player camera
// =============================================================================

/// Записать FOV для одного PlayerCameraView (Interier или Exterier).
///
/// Пишет в DefaultFOV и во все 15 per-state FOV слотов.
/// Очищает per-state FOV flag в 0 (= «есть override»),
/// чтобы движок не перезаписал наше значение из дефолта.
fn set_view_fov(view_offset: usize, fov: f32) -> bool {
    // DefaultFOV
    unsafe {
        if !memory::write_value(default_fov_addr(view_offset), fov) {
            return false;
        }
    }

    // Все 15 state-слотов + их флаги
    for n in 0..cv::NUM_STATES {
        unsafe {
            memory::write_value(state_fov_addr(view_offset, n), fov);
            memory::write_value::<u8>(state_fov_flag_addr(view_offset, n), 0);
        }
    }

    true
}

/// Установить FOV только для player camera (Interier + Exterier).
///
/// Пишет DefaultFOV и все 15 per-state слотов для обоих view.
/// Не затрагивает автомобильные камеры.
pub fn set_player_fov(fov: f32) -> bool {
    if !is_initialized() || !is_valid_fov(fov) {
        return false;
    }

    let ok_int = set_view_fov(cm::INTERIER_VIEW, fov);
    let ok_ext = set_view_fov(cm::EXTERIER_VIEW, fov);

    if ok_int && ok_ext {
        logger::info(&format!("[camera] player FOV -> {fov:.1}"));
    } else {
        logger::error("[camera] ошибка записи player FOV");
    }

    ok_int && ok_ext
}

/// Установить FOV раздельно для Interier и Exterier.
///
/// Полезно, если нужен разный FOV для помещений и улицы.
pub fn set_player_fov_separate(interier_fov: f32, exterier_fov: f32) -> bool {
    if !is_initialized() {
        return false;
    }
    if !is_valid_fov(interier_fov) || !is_valid_fov(exterier_fov) {
        return false;
    }

    let ok_int = set_view_fov(cm::INTERIER_VIEW, interier_fov);
    let ok_ext = set_view_fov(cm::EXTERIER_VIEW, exterier_fov);

    ok_int && ok_ext
}

// =============================================================================
//  Запись FOV — Все камеры
// =============================================================================

/// Установить FOV для ВСЕХ камер: player + car + misc.
///
/// Что пишет:
/// - Player camera: Interier/Exterier DefaultFOV + все 15 per-state слотов
/// - Все car cameras: базовый Fov (11 оффсетов)
/// - FovMax (speed-delta): обнуляется, чтобы FOV не «уплывал» на скорости
/// - Misc: fpvCamera, deathCamera, meeleeCamera
///
/// Возвращает `false` если камера не инициализирована или FOV невалиден.
pub fn set_all_fov(fov: f32) -> bool {
    if !is_initialized() || !is_valid_fov(fov) {
        return false;
    }

    // Player camera (Interier + Exterier, defaults + все states)
    let ok_player = set_player_fov(fov);

    let mgr = camera_mgr();
    let mut ok_car = true;

    // Базовый Fov всех car/misc камер
    for &offset in ALL_FOV_OFFSETS {
        unsafe {
            if !memory::write_value(mgr + offset, fov) {
                ok_car = false;
            }
        }
    }

    // Обнуляем FovMax (speed-delta), иначе на скорости камера
    // добавит дельту к нашему FOV и угол обзора «поплывёт»
    for &offset in FOV_MAX_OFFSETS {
        unsafe {
            memory::write_value(mgr + offset, 0.0f32);
        }
    }

    if ok_player && ok_car {
        logger::info(&format!("[camera] all FOV -> {fov:.1}"));
    } else {
        logger::warn(&format!(
            "[camera] partial FOV write (player={ok_player}, car={ok_car})"
        ));
    }

    ok_player && ok_car
}

// =============================================================================
//  Запись произвольного параметра камеры
// =============================================================================

/// Записать произвольный параметр в DefaultParams + все 15 states.
///
/// Работает только с PlayerCameraView (Interier/Exterier).
/// Для car cameras используй `set_car_camera_param()`.
pub fn set_default_param(view_offset: usize, param_index: usize, value: f32) -> bool {
    if !is_initialized() || param_index >= cv::NUM_PARAMS {
        return false;
    }

    let mgr = camera_mgr();

    // DefaultParams
    let default_addr = mgr + view_offset + cv::DEFAULT_PARAMS + param_index * 4;
    unsafe {
        memory::write_value(default_addr, value);
    }

    // Все states + flags
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

/// Записать float в конкретный индекс параметров car-камеры.
///
/// `params_offset` — базовый оффсет массива (e.g. `cm::CAR_DYNAMIC_PARAMS`).
/// `param_index` — индекс в массиве.
pub fn set_car_camera_param(params_offset: usize, param_index: usize, value: f32) -> bool {
    let addr = camera_mgr() + params_offset + param_index * 4;
    unsafe { memory::write_value(addr, value) }
}

// =============================================================================
//  Диагностика
// =============================================================================

/// Вывести текущее состояние камерной системы в лог.
///
/// Показывает FOV и Distance для обоих view,
/// плюс State[0] FOV для проверки синхронизации.
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

    // Distance для контекста
    let int_dist = get_default_param(cm::INTERIER_VIEW, camera_params::DISTANCE).unwrap_or(0.0);
    let ext_dist = get_default_param(cm::EXTERIER_VIEW, camera_params::DISTANCE).unwrap_or(0.0);
    logger::info(&format!(
        "[camera] Distance: interier={int_dist:.1}, exterier={ext_dist:.1}"
    ));

    // State[0] FOV — должен совпадать с default после нашей записи
    let s0_int = get_state_fov(cm::INTERIER_VIEW, 0).unwrap_or(0.0);
    let s0_ext = get_state_fov(cm::EXTERIER_VIEW, 0).unwrap_or(0.0);
    logger::debug(&format!(
        "[camera] State[0] FOV: interier={s0_int:.1}, exterier={s0_ext:.1}"
    ));

    // Car camera FOV
    let mgr = camera_mgr();
    let dyn_fov = unsafe { memory::read_value::<f32>(mgr + cm::CAR_DYNAMIC_FOV) }.unwrap_or(0.0);
    let dyn_max =
        unsafe { memory::read_value::<f32>(mgr + cm::CAR_DYNAMIC_FOV_MAX) }.unwrap_or(0.0);
    logger::info(&format!(
        "[camera] carDynamic: Fov={dyn_fov:.1}, FovMax={dyn_max:.1}"
    ));
}

/// Дамп параметров car-камеры для поиска/верификации FOV-индексов.
///
/// Вызывать из main thread после загрузки уровня (InGame state).
/// Помечает float-значения 50..120 как потенциальные FOV.
pub fn scan_car_camera_params() {
    if !is_initialized() {
        logger::warn("[camera-scan] камера не инициализирована");
        return;
    }

    let mgr = camera_mgr();

    let cameras: &[(&str, usize, usize)] = &[
        (
            "carCameraDynamic",
            cm::CAR_DYNAMIC_PARAMS,
            cm::CAR_DYNAMIC_PARAM_COUNT,
        ),
        (
            "carCameraDynamicLong",
            cm::CAR_DYNAMIC_LONG_PARAMS,
            cm::CAR_DYNAMIC_LONG_PARAM_COUNT,
        ),
        (
            "carCameraShoot",
            cm::CAR_SHOOT_PARAMS,
            cm::CAR_SHOOT_PARAM_COUNT,
        ),
        (
            "carCameraGamepad",
            cm::CAR_GAMEPAD_PARAMS,
            cm::CAR_GAMEPAD_PARAM_COUNT,
        ),
    ];

    for &(name, params_offset, count) in cameras {
        logger::info(&format!(
            "[camera-scan] {name} ({count} params at +0x{params_offset:X}):"
        ));

        for i in 0..count {
            let addr = mgr + params_offset + i * 4;
            let val = unsafe { memory::read_value::<f32>(addr) }.unwrap_or(0.0);

            let marker = if (50.0..=120.0).contains(&val) {
                " ◄ FOV?"
            } else {
                ""
            };
            logger::info(&format!(
                "  [{i:2}] +0x{:04X} = {val:12.4}{marker}",
                params_offset + i * 4
            ));
        }
    }
}
