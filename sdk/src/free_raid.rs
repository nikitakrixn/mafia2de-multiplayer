use std::ffi::c_void;
use crate::physical_processor::CPhysicalProcessor;

/// Класс FreeRaid (подкласс PhysicalProcessor с другой vtable)
#[repr(C)]
pub struct CFreeRaid {
    // Наследует все поля от CPhysicalProcessor
    pub base: CPhysicalProcessor,
    // Дополнительные поля можно добавить здесь
}

impl CFreeRaid {
    /// Инициализирует FreeRaid режим
    pub fn initialize(&mut self) {
        unsafe {
            let vtable = self.base.vtable as *const *const c_void;
            let init_fn = *vtable.offset(3);
            let init_fn: unsafe extern "fastcall" fn(*mut CFreeRaid) = std::mem::transmute(init_fn);
            init_fn(self);
        }
    }
    
    /// Устанавливает режим игры
    pub fn set_game_mode(&mut self, mode: i32) {
        unsafe {
            let vtable = self.base.vtable as *const *const c_void;
            let set_mode_fn = *vtable.offset(4);
            let set_mode_fn: unsafe extern "fastcall" fn(*mut CFreeRaid, i32) = std::mem::transmute(set_mode_fn);
            set_mode_fn(self, mode);
        }
    }
    
    /// Преобразует указатель на CPhysicalProcessor в CFreeRaid, если это возможно
    pub unsafe fn from_physical_processor(processor: *mut CPhysicalProcessor) -> Option<&'static mut CFreeRaid> {
        // Проверка типа обычно происходит через RTTI или другие механизмы
        // Здесь мы просто делаем простую проверку на ненулевой указатель
        if !processor.is_null() {
            // В реальном коде здесь должна быть дополнительная проверка типа
            unsafe {
                Some(&mut *(processor as *mut CFreeRaid))
            }
        } else {
            None
        }
    }
}

// Реализация методов CPhysicalProcessor для CFreeRaid через делегирование
impl std::ops::Deref for CFreeRaid {
    type Target = CPhysicalProcessor;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CFreeRaid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
} 