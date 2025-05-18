use std::ffi::{c_void, CStr};
use crate::translocator::CTranslocator;

/// Тип функции обработки транслокатора
pub type ProcessTranslocatorFunc = unsafe extern "fastcall" fn(*mut CPhysicalProcessor, *mut CTranslocator) -> *mut c_void;

/// Класс физического процессора (C_PhysicalProcessor)
#[repr(C)]
pub struct CPhysicalProcessor {
    pub vtable: *const c_void,        // 0x00 - Указатель на виртуальную таблицу (0x14186ded8)
    pub vtable_func: *const c_void,   // 0x08 - Указатель на функцию (0x14186e040)
    pub padding1: u64,                // 0x10 - Выравнивание
    pub flags: u32,                   // 0x20 - Флаги управления (значение 1)
    pub padding2: u32,                // 0x24 - Выравнивание
    pub ptr1: *mut c_void,            // 0x30 - Указатель на объект 1
    pub ptr2: *mut c_void,            // 0x38 - Указатель на объект 2
    pub ptr3: *mut c_void,            // 0x40 - Указатель на объект 3
    pub padding3: [u64; 3],           // 0x48-0x60 - Выравнивание
    pub constant: u64,                // 0x60 - Константа 0x02000007D0
    pub padding4: [u64; 3],           // 0x68-0x80 - Выравнивание
    pub timer_name: *mut i8,          // 0x80 - Название таймера ("Tic")
    pub padding5: [u64; 15],          // 0x88-0x100 - Выравнивание
    pub obj_ptr1: *mut c_void,        // 0x100 - Указатель на объект связанный с физикой
    pub obj_ptr2: *mut c_void,        // 0x108 - Указатель на объект связанный с физикой
    pub obj_ptr3: *mut c_void,        // 0x110 - Указатель на объект связанный с физикой
    pub padding6: [u64; 24],          // 0x118-0x1D0 - Выравнивание
    pub control_flags: u32,           // 0x1D0 - Флаги управления (0x38)
    pub padding7: u32,                // 0x1D4 - Выравнивание
    pub flag_value: u32,              // 0x1D8 - Значение флага (3)
    pub padding8: u32,                // 0x1DC - Выравнивание
    pub obj_ptr4: *mut c_void,        // 0x1E0 - Указатель на рабочий объект 1
    pub obj_ptr5: *mut c_void,        // 0x1E8 - Указатель на рабочий объект 2
    pub obj_ptr6: *mut c_void,        // 0x1F0 - Указатель на рабочий объект 3
    pub locale: [u8; 8],              // 0x1F8 - Строка локализации ("Korean")
}

impl CPhysicalProcessor {
    /// Обрабатывает транслокатор
    pub fn process_translocator(&mut self, translocator: &mut CTranslocator) {
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let process_fn: ProcessTranslocatorFunc = std::mem::transmute(*vtable.offset(8));
            process_fn(self, translocator);
        }
    }
    
    /// Обновляет физику
    pub fn update_physics(&mut self) {
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let update_fn = *vtable.offset(6);
            let update_fn: unsafe extern "fastcall" fn(*mut CPhysicalProcessor) = std::mem::transmute(update_fn);
            update_fn(self);
        }
    }
    
    /// Находит транслокатор по имени
    pub fn find_translocator_by_name(&mut self, name: &str) -> Option<&mut CTranslocator> {
        use std::ffi::CString;
        
        let c_name = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return None,
        };
        
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let find_fn = *vtable.offset(9);
            let find_fn: unsafe extern "fastcall" fn(*mut CPhysicalProcessor, *const i8) -> *mut CTranslocator = 
                std::mem::transmute(find_fn);
            
            let translocator_ptr = find_fn(self, c_name.as_ptr());
            if translocator_ptr.is_null() {
                None
            } else {
                Some(&mut *translocator_ptr)
            }
        }
    }
    
    /// Получает имя таймера
    pub fn get_timer_name(&self) -> Option<&str> {
        if self.timer_name.is_null() {
            None
        } else {
            unsafe {
                CStr::from_ptr(self.timer_name).to_str().ok()
            }
        }
    }
    
    /// Получает локаль из структуры
    pub fn get_locale(&self) -> Option<&str> {
        unsafe {
            let locale_ptr = self.locale.as_ptr() as *const i8;
            if *locale_ptr == 0 {
                None
            } else {
                CStr::from_ptr(locale_ptr).to_str().ok()
            }
        }
    }
} 