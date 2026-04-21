//! VTable `C_GameInputModule` — 5 слотов.
//!
//! Адрес в `.rdata`: `M2DE_VT_CGameInputModule` (`0x141908AC0`).
//! Идентификатор класса: строка `"GameInputModule"` через слот `[2]`.
//!
//! `C_GameInputModule` наследуется от `C_TickedModule`, поэтому первые
//! четыре слота имеют фиксированный TickedModule-интерфейс (тот же, что
//! и у `C_Application`):
//!
//! | Слот | Имя                  | Описание |
//! |:----:|:---------------------|:---------|
//! | 0    | `dtor`               | Деструктор (`M2DE_GameInputModule_VT_Dtor`, `0x140FDCB90`) |
//! | 1    | `get_module_id`      | Возвращает `7` (общий для всех C_TickedModule) |
//! | 2    | `get_class_name`     | Возвращает `"GameInputModule"` |
//! | 3    | `register_callbacks` | `M2DE_GameInputModule_RegisterCallbacks` (`0x1410064F0`) |
//! | 4    | `get_fixed_time_step`| Возвращает `0.005f` (200 Гц), общий метод TickedModule |

use std::ffi::{c_char, c_void};

type FnDtor = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnU32 = unsafe extern "system" fn(this: *const c_void) -> u32;
type FnF32 = unsafe extern "system" fn(this: *const c_void) -> f32;
type FnVoid = unsafe extern "system" fn(this: *mut c_void);

/// VTable `C_GameInputModule` — `M2DE_VT_CGameInputModule` @ `0x141908AC0`.
#[repr(C)]
pub struct CGameInputModuleVTable {
    /// `[0]` Деструктор.
    pub dtor: FnDtor,

    /// `[1]` Возвращает module id = `7`.
    ///
    /// Общий метод `C_TickedModule` (один и тот же бинарный код шарится
    /// между Application и GameInputModule).
    pub get_module_id: FnU32,

    /// `[2]` Возвращает строку `"GameInputModule"`.
    pub get_class_name: unsafe extern "system" fn(this: *const c_void) -> *const c_char,

    /// `[3]` Регистрирует все callbacks модуля в `GameCallbackManager`
    /// (включая per-frame `Tick` на event `22`/`3`/`4`/`37` и
    /// `OnGamePaused`/`OnGameUnpaused` на `34`/`35`).
    pub register_callbacks: FnVoid,

    /// `[4]` Фиксированный шаг симуляции: `0.005f` (200 Гц).
    ///
    /// Общий метод `C_TickedModule`.
    pub get_fixed_time_step: FnF32,
}

const _: () = {
    assert!(std::mem::size_of::<CGameInputModuleVTable>() == 5 * 8);
    assert!(std::mem::offset_of!(CGameInputModuleVTable, dtor) == 0 * 8);
    assert!(std::mem::offset_of!(CGameInputModuleVTable, get_module_id) == 1 * 8);
    assert!(std::mem::offset_of!(CGameInputModuleVTable, get_class_name) == 2 * 8);
    assert!(std::mem::offset_of!(CGameInputModuleVTable, register_callbacks) == 3 * 8);
    assert!(std::mem::offset_of!(CGameInputModuleVTable, get_fixed_time_step) == 4 * 8);
};
