use std::ffi::{c_void, CStr};
use std::ptr::null_mut;

/// Тип функции загрузки ресурса из менеджера ресурсов
type LoadResourceFn = unsafe extern "fastcall" fn(*mut CResourceManager, *const i8) -> *mut c_void;

/// Тип функции получения ресурса по имени
type GetResourceByNameFn = unsafe extern "fastcall" fn(*mut CResourceManager, *const i8) -> *mut c_void;

/// Тип функции получения пути к игре
type GetGamePathFn = unsafe extern "fastcall" fn(*mut CResourceManager) -> *const i8;

/// Класс менеджера ресурсов (C_ResourceManager)
#[repr(C)]
pub struct CResourceManager {
    pub vtable: *const c_void,        // 0x00 - Указатель на виртуальную таблицу (0x14188df88)
    pub vtable_ext1: *const c_void,   // 0x08 - Дополнительный указатель на таблицу функций (0x14188df98)
    pub vtable_ext2: *const c_void,   // 0x10 - Дополнительный указатель на таблицу функций (0x14188dfa8)
    pub data_pointer: *mut c_void,    // 0x18 - Указатель на данные
    pub resource_path: *mut c_void,   // 0x20 - Указатель на путь к ресурсам (может быть 0)
    pub object_pointer: *mut c_void,  // 0x28 - Указатель на служебный объект
    pub data_ptr1: *mut c_void,       // 0x30 - Указатель на дополнительные данные
    pub data_ptr2: *mut c_void,       // 0x38 - Указатель на дополнительные данные
    pub flags: u32,                   // 0x40 - Флаги
    pub padding1: u32,                // 0x44 - Выравнивание
    pub render_param1: f32,           // 0x48 - Параметр рендеринга (1.0)
    pub render_param2: f32,           // 0x4C - Параметр рендеринга
    pub game_path: *mut c_void,       // 0x50 - Путь к файлам игры
    pub params: [u32; 10],            // 0x58-0x80 - Различные параметры
    pub render_settings: [f32; 16],   // 0x80-0xC0 - Настройки рендеринга
    pub steam_path: [u8; 64],         // 0x1C0 - Путь к установке Steam
}

impl CResourceManager {
    /// Загружает ресурс по имени
    pub fn load_resource(&mut self, resource_name: &str) -> *mut c_void {
        use std::ffi::CString;
        
        let c_resource_name = match CString::new(resource_name) {
            Ok(s) => s,
            Err(_) => return null_mut(),
        };
        
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let load_resource_fn: LoadResourceFn = std::mem::transmute(*vtable.offset(3));
            load_resource_fn(self, c_resource_name.as_ptr())
        }
    }
    
    /// Получает ресурс по имени, если он уже загружен
    pub fn get_resource_by_name(&mut self, resource_name: &str) -> *mut c_void {
        use std::ffi::CString;
        
        let c_resource_name = match CString::new(resource_name) {
            Ok(s) => s,
            Err(_) => return null_mut(),
        };
        
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let get_resource_fn: GetResourceByNameFn = std::mem::transmute(*vtable.offset(4));
            get_resource_fn(self, c_resource_name.as_ptr())
        }
    }
    
    /// Получает путь к файлам игры
    pub fn get_game_path(&self) -> Option<&str> {
        unsafe {
            let vtable = self.vtable as *const *const c_void;
            let get_path_fn: GetGamePathFn = std::mem::transmute(*vtable.offset(2));
            let path_ptr = get_path_fn(self as *const _ as *mut _);
            
            if path_ptr.is_null() {
                None
            } else {
                CStr::from_ptr(path_ptr).to_str().ok()
            }
        }
    }
    
    /// Получает путь к Steam из структуры
    pub fn get_steam_path(&self) -> Option<&str> {
        unsafe {
            let path_ptr = self.steam_path.as_ptr() as *const i8;
            if *path_ptr == 0 {
                None
            } else {
                CStr::from_ptr(path_ptr).to_str().ok()
            }
        }
    }
} 