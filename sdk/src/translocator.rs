use std::ffi::{c_void, CStr};
use std::ptr::null_mut;
use crate::types::Vector3;

/// Тип указателя на виртуальную функцию активации транслокатора
pub type ActivateFunc = unsafe extern "fastcall" fn(*mut CTranslocator, i32) -> *mut c_void;

/// Класс транслокатора, соответствующий C_Translocator из C++ кода
#[repr(C)]
pub struct CTranslocator {
    pub vtable: *const c_void,      // 0x00 - Указатель на виртуальную таблицу
    pub flags: u32,                 // 0x08 - Флаги состояния
    pub padding1: u32,              // 0x0C - Выравнивание
    pub name: *mut i8,              // 0x10 - Название объекта
    pub position: Vector3,          // 0x18 - Позиция в мире
    pub rotation: Vector3,          // 0x24 - Вращение
    pub scale: Vector3,             // 0x30 - Масштаб
    pub flags2: u32,                // 0x3C - Дополнительные флаги
    pub unknown1: u64,              // 0x40 - Неизвестное поле
}

impl CTranslocator {
    /// Устанавливает позицию транслокатора
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position.x = x;
        self.position.y = y;
        self.position.z = z;
    }
    
    /// Устанавливает вращение транслокатора
    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.rotation.x = x;
        self.rotation.y = y;
        self.rotation.z = z;
    }
    
    /// Устанавливает масштаб транслокатора
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale.x = x;
        self.scale.y = y;
        self.scale.z = z;
    }
    
    /// Активирует транслокатор
    pub fn activate(&mut self) {
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let activate_fn: ActivateFunc = std::mem::transmute(*vtable.offset(5));
            activate_fn(self, 0);
        }
    }
    
    /// Получает имя транслокатора
    pub fn get_name(&self) -> Option<&str> {
        if self.name.is_null() {
            None
        } else {
            unsafe {
                CStr::from_ptr(self.name).to_str().ok()
            }
        }
    }
    
    /// Создает новый транслокатор с заданным именем (для тестирования)
    pub fn new_test(name: &str) -> Self {
        use std::ffi::CString;
        
        let name_cstring = CString::new(name).unwrap_or_default();
        let name_ptr = name_cstring.into_raw();
        
        Self {
            vtable: null_mut(),
            flags: 0,
            padding1: 0,
            name: name_ptr,
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            flags2: 0,
            unknown1: 0,
        }
    }
} 