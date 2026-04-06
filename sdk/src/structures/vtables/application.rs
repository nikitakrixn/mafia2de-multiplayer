//! VTable `C_Application` — 5 слотов.
//!
//! Адрес в `.rdata`: `M2DE_VT_CApplication` (`0x1418538F8`).
//! Идентификатор класса: строка `"C_Application"` через слот `[2]`.
//!
//! ## Карта слотов
//!
//! | Слот | Имя | Описание |
//! |:----:|:----|:---------|
//! | 0 | `dtor` | Деструктор |
//! | 1 | `get_module_id` | Возвращает `7` |
//! | 2 | `get_class_name` | Возвращает `"C_Application"` |
//! | 3 | `static_register` | Регистрирует все callbacks |
//! | 4 | `get_fixed_time_step` | Возвращает `0.005f` (200 Гц) |

use std::ffi::{c_char, c_void};

type FnDtor = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnU32 = unsafe extern "system" fn(this: *const c_void) -> u32;
type FnF32 = unsafe extern "system" fn(this: *const c_void) -> f32;
type FnVoid = unsafe extern "system" fn(this: *mut c_void);

/// VTable `C_Application` — `M2DE_VT_CApplication` @ `0x1418538F8`.
#[repr(C)]
pub struct CApplicationVTable {
    /// `[0]` Деструктор.
    pub dtor: FnDtor,

    /// `[1]` Возвращает module id = `7`.
    pub get_module_id: FnU32,

    /// `[2]` Возвращает строку `"C_Application"`.
    pub get_class_name: unsafe extern "system" fn(this: *const c_void) -> *const c_char,

    /// `[3]` Регистрирует все callbacks в `GameCallbackManager`.
    pub static_register: FnVoid,

    /// `[4]` Фиксированный шаг симуляции: `0.005f` (200 Гц).
    pub get_fixed_time_step: FnF32,
}

const _: () = {
    assert!(std::mem::size_of::<CApplicationVTable>() == 5 * 8);
    assert!(std::mem::offset_of!(CApplicationVTable, dtor) == 0 * 8);
    assert!(std::mem::offset_of!(CApplicationVTable, get_module_id) == 1 * 8);
    assert!(std::mem::offset_of!(CApplicationVTable, get_class_name) == 2 * 8);
    assert!(std::mem::offset_of!(CApplicationVTable, static_register) == 3 * 8);
    assert!(std::mem::offset_of!(CApplicationVTable, get_fixed_time_step) == 4 * 8);
};
