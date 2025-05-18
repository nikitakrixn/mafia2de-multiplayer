use std::ffi::c_void;
use std::slice;
use crate::globals::addresses;

/// Структура для управления списком игровых объектов
#[repr(C)]
pub struct ObjectListManager {
    pub objects: *mut *mut c_void,   // Массив указателей на объекты
    pub count: u32,                  // Количество объектов в списке
    pub capacity: u32,               // Максимальная емкость списка
}

impl ObjectListManager {
    /// Получает глобальный экземпляр менеджера объектов
    pub fn get_instance() -> Option<&'static mut Self> {
        unsafe {
            let ptr = addresses::OBJECT_LIST_ADDR as *mut Self;
            if ptr.is_null() || (*ptr).objects.is_null() {
                None
            } else {
                Some(&mut *ptr)
            }
        }
    }
    
    /// Получает текущее количество объектов из адреса счетчика
    pub fn get_count() -> u32 {
        unsafe {
            *(addresses::OBJECT_LIST_COUNT_ADDR as *const u32)
        }
    }
    
    /// Получает массив указателей на объекты в виде среза
    pub fn get_objects_slice(&self) -> &[*mut c_void] {
        unsafe {
            slice::from_raw_parts(self.objects, self.count as usize)
        }
    }
    
    /// Получает мутабельный массив указателей на объекты в виде среза
    pub fn get_objects_slice_mut(&mut self) -> &mut [*mut c_void] {
        unsafe {
            slice::from_raw_parts_mut(self.objects, self.count as usize)
        }
    }
    
    /// Получает объект по индексу
    pub fn get_object(&self, index: usize) -> Option<*mut c_void> {
        if index < self.count as usize {
            unsafe {
                let ptr = *self.objects.add(index);
                if ptr.is_null() {
                    None
                } else {
                    Some(ptr)
                }
            }
        } else {
            None
        }
    }
    
    /// Проверяет, является ли данный указатель объектом из списка
    pub fn is_valid_object(&self, obj: *mut c_void) -> bool {
        if obj.is_null() {
            return false;
        }
        
        for i in 0..self.count as usize {
            unsafe {
                if *self.objects.add(i) == obj {
                    return true;
                }
            }
        }
        
        false
    }
}

/// Функции для работы с объектным списком
pub mod object_list {
    use super::*;
    
    /// Получает количество объектов в списке
    pub fn get_object_count() -> u32 {
        ObjectListManager::get_count()
    }
    
    /// Получает указатель на объект по индексу
    pub fn get_object(index: usize) -> Option<*mut c_void> {
        ObjectListManager::get_instance()
            .and_then(|list| list.get_object(index))
    }
    
    /// Перебирает все объекты и применяет к ним функцию обратного вызова
    pub fn for_each<F>(mut callback: F) 
    where 
        F: FnMut(usize, *mut c_void),
    {
        if let Some(list) = ObjectListManager::get_instance() {
            for (i, &obj) in list.get_objects_slice().iter().enumerate() {
                if !obj.is_null() {
                    callback(i, obj);
                }
            }
        }
    }
} 