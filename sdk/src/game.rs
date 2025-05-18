use std::ffi::{c_void, c_char, CStr};
use crate::physical_processor::CPhysicalProcessor;
use crate::resource_manager::CResourceManager;

/// Типы функций виртуальной таблицы для C_Game
pub type InitializeT = unsafe extern "fastcall" fn(*mut CGame, *const c_char);
pub type SetMapNameT = unsafe extern "fastcall" fn(*mut CGame, *const c_char);
pub type ShutdownT = unsafe extern "fastcall" fn(*mut CGame);
pub type GetTimeDeltaT = unsafe extern "fastcall" fn(*mut CGame) -> f32;
pub type GetResourcePtrT = unsafe extern "fastcall" fn(*mut CGame) -> *mut CResourceManager;
pub type GetMapNamePtrT = unsafe extern "fastcall" fn(*mut CGame) -> *const c_char;
pub type SetValue11CT = unsafe extern "fastcall" fn(*mut CGame, u16);
pub type GetValue11CT = unsafe extern "fastcall" fn(*mut CGame) -> u16;
pub type InitResourcesT = unsafe extern "fastcall" fn(*mut CGame) -> bool;
pub type CleanupResourcesT = unsafe extern "fastcall" fn(*mut CGame);

/// Структура виртуальной таблицы для C_Game
#[repr(C)]
pub struct CGameVTable {
    pub get_class_name: *const c_void,           // Индекс 0 - Возвращает "C_Game" - 0x1403d5dd0
    pub initialize: InitializeT,                // Индекс 1 - Инициализирует игру - 0x1400a78d0
    pub set_map_name: SetMapNameT,              // Индекс 2 - Устанавливает имя карты - 0x1403eb2d0
    pub shutdown: ShutdownT,                    // Индекс 3 - Завершает работу игры - 0x1403ba1a0
    pub get_time_delta: GetTimeDeltaT,          // Индекс 4 - Возвращает дельту времени - 0x1400a7d80
    pub get_resource_ptr: GetResourcePtrT,      // Индекс 5 - Возвращает указатель на ресурсы - 0x1403eab30
    pub get_map_name_ptr: GetMapNamePtrT,       // Индекс 6 - Возвращает имя карты - 0x1403e9e90
    pub set_value_11c: SetValue11CT,            // Индекс 7 - Устанавливает значение 11C - 0x1403f7cd0
    pub get_value_11c: GetValue11CT,            // Индекс 8 - Получает значение 11C - 0x1403e9ea0
    pub init_resources: InitResourcesT,         // Индекс 9 - Инициализирует ресурсы - 0x1403ebab0
    pub cleanup_resources: CleanupResourcesT,   // Индекс 10 - Очищает ресурсы - 0x1403e1050
}

/// Основной класс игры (C_Game)
#[repr(C)]
pub struct CGame {
    pub vtable: *const CGameVTable,            // 0x00 - Указатель на vtable (0x14186EFE8)
    pub flags: u8,                             // 0x08 - Флаг бит 0: если установлен, вызывает CleanupResources при уничтожении
                                              //        Флаг бит 1: игра запущена
    pub padding1: [u8; 7],                     // 0x09 - Выравнивание до 0x10
    pub resources: *mut CResourceManager,      // 0x10 - Указатель на ResourceManager
    pub map_name: [c_char; 16],                // 0x18 - Имя карты ("CITY_trick")
    pub value28: u32,                          // 0x28 - Значение (может быть связано с состоянием)
    pub value2c: u32,                          // 0x2C - Значение (может быть связано с состоянием)
    pub value30: u32,                          // 0x30 - Значение (обычно 1)
    pub padding2: u32,                         // 0x34 - Выравнивание
    pub ptr_object1: *mut c_void,              // 0x38 - Указатель на объект 1
    // Дополнительные поля, основанные на дампе памяти
    pub param_40: f32,                         // 0x40 - Значение с плавающей точкой (7.0)
    pub param_44: f32,                         // 0x44 - Значение с плавающей точкой (-1.0)
    pub param_48: f32,                         // 0x48 - Значение с плавающей точкой (0.0)
    pub count_4c: u32,                         // 0x4C - Счетчик или индекс (1)
    pub obj_ptr_50: *mut c_void,               // 0x50 - Указатель на объект
    // ... другие поля до 0x118 ...
    pub padding3: [u8; 200],                   // Заполнитель для полей между 0x50 и 0x118
    pub physical_processor: *mut CPhysicalProcessor, // 0x118 - Указатель на физический процессор
    // ... и далее ...
    pub value11c: u16,                         // 0x11C - Значение, доступное через методы vtable
}

impl CGame {
    /// Инициализирует игру с указанным именем карты
    pub fn initialize(&mut self, map_name: &str) {
        use std::ffi::CString;
        
        if let Ok(c_map_name) = CString::new(map_name) {
            unsafe {
                ((*self.vtable).initialize)(self, c_map_name.as_ptr());
            }
        }
    }
    
    /// Устанавливает имя карты
    pub fn set_map_name(&mut self, map_name: &str) {
        use std::ffi::CString;
        
        if let Ok(c_map_name) = CString::new(map_name) {
            unsafe {
                ((*self.vtable).set_map_name)(self, c_map_name.as_ptr());
            }
        }
    }
    
    /// Завершает работу игры
    pub fn shutdown(&mut self) {
        unsafe {
            ((*self.vtable).shutdown)(self);
        }
    }
    
    /// Возвращает дельту времени
    pub fn get_time_delta(&self) -> f32 {
        unsafe {
            ((*self.vtable).get_time_delta)(self as *const _ as *mut _)
        }
    }
    
    /// Возвращает указатель на менеджер ресурсов
    pub fn get_resource_manager(&mut self) -> Option<&mut CResourceManager> {
        unsafe {
            let resource_ptr = ((*self.vtable).get_resource_ptr)(self);
            if resource_ptr.is_null() {
                None
            } else {
                Some(&mut *resource_ptr)
            }
        }
    }
    
    /// Возвращает имя карты
    pub fn get_map_name(&self) -> Option<&str> {
        unsafe {
            let map_name_ptr = ((*self.vtable).get_map_name_ptr)(self as *const _ as *mut _);
            if map_name_ptr.is_null() {
                None
            } else {
                CStr::from_ptr(map_name_ptr).to_str().ok()
            }
        }
    }
    
    /// Устанавливает значение 11C
    pub fn set_value(&mut self, value: u16) {
        unsafe {
            ((*self.vtable).set_value_11c)(self, value);
        }
    }
    
    /// Получает значение 11C
    pub fn get_value(&self) -> u16 {
        unsafe {
            ((*self.vtable).get_value_11c)(self as *const _ as *mut _)
        }
    }
    
    /// Инициализирует ресурсы
    pub fn init_resources(&mut self) -> bool {
        unsafe {
            ((*self.vtable).init_resources)(self)
        }
    }
    
    /// Очищает ресурсы
    pub fn cleanup_resources(&mut self) {
        unsafe {
            ((*self.vtable).cleanup_resources)(self);
        }
    }
    
    /// Получает указатель на физический процессор
    pub fn get_physical_processor(&mut self) -> Option<&mut CPhysicalProcessor> {
        if self.physical_processor.is_null() {
            None
        } else {
            unsafe {
                Some(&mut *self.physical_processor)
            }
        }
    }
    
    /// Проверяет, запущена ли игра
    pub fn is_running(&self) -> bool {
        (self.flags & 0x2) != 0
    }
} 