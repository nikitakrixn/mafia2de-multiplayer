//! Модель, внешний вид, коллизия, damage scale.

use std::ffi::CStr;

use crate::addresses::fields;
use crate::memory;

use super::Player;

impl Player {
    /// ID текущего внешнего вида (одежда/модель). `None` если не активен.
    pub fn get_appearance_id(&self) -> Option<u32> {
        unsafe {
            let comp = self.human()?.frame_colors as usize;
            if !memory::is_valid_ptr(comp) {
                return None;
            }
            let ptr_data = memory::read_ptr(comp)?;
            let id = memory::read_value::<u32>(ptr_data)?;
            (id != 0xFFFF_FFFF).then_some(id)
        }
    }

    /// Имя текущей модели из model descriptor (+0xA8).
    pub fn get_model_name(&self) -> Option<String> {
        unsafe {
            let desc = self.human()?.model_descriptor as usize;
            if !memory::is_valid_ptr(desc) {
                return None;
            }
            let name_addr = desc + fields::model_descriptor::MODEL_NAME;
            if !memory::is_valid_ptr(name_addr) {
                return None;
            }
            let cstr = CStr::from_ptr(name_addr as *const i8);
            let s = cstr.to_string_lossy().into_owned();
            (!s.is_empty()).then_some(s)
        }
    }

    /// Есть ли активный water detector.
    pub fn has_water_detector(&self) -> Option<bool> {
        unsafe { self.human().map(|h| !h.water_detector.is_null()) }
    }

    /// Указатель на locomotion controller.
    pub fn locomotion_controller_ptr(&self) -> Option<usize> {
        let ptr = unsafe { self.human()?.locomotion as usize };
        memory::is_valid_ptr(ptr).then_some(ptr)
    }

    /// Damage scale factor (`CHuman.damage_scale_factor`).
    pub fn get_damage_scale(&self) -> Option<f32> {
        unsafe { self.human().map(|h| h.damage_scale_factor) }
    }

    /// Установить damage scale factor.
    pub fn set_damage_scale(&self, scale: f32) -> bool {
        if !self.ptr.is_valid() {
            return false;
        }
        unsafe {
            (&raw mut (*self.ptr.raw()).base.damage_scale_factor).write(scale);
        }
        true
    }
}
