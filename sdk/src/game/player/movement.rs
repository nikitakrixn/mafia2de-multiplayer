//! Позиция, направление, скорость — через vtable.

use crate::addresses::fields;
use crate::memory;
use crate::types::Vec3;

use super::Player;

// =============================================================================
//  Позиция (через vtable)
// =============================================================================

impl Player {
    /// Мировая позиция через vtable\[36\] `GetPos`.
    ///
    /// Автоматически вызывает правильную реализацию:
    /// physics provider → fallback на frame node.
    pub fn get_position(&self) -> Option<Vec3> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = Vec3::ZERO;
            let ret = (vt.get_pos)(self.this_const(), &mut out);
            if ret.is_null() || !out.is_finite() {
                return None;
            }
            Some(out)
        }
    }

    /// Установить позицию через vtable\[32\] `SetPos`.
    ///
    /// Обновляет physics + cache + dirty flags.
    pub fn set_position(&self, pos: &Vec3) -> bool {
        if !pos.is_finite() {
            return false;
        }
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_pos)(self.this_mut(), pos);
        }
        true
    }

    /// Прямое чтение позиции из frame node (debug/fallback).
    pub fn get_position_from_frame(&self) -> Option<Vec3> {
        let frame = unsafe { self.human()?.actor.frame_node as usize };
        if !memory::is_valid_ptr(frame) {
            return None;
        }
        unsafe {
            let x = memory::read_value::<f32>(frame + fields::entity_frame::POS_X)?;
            let y = memory::read_value::<f32>(frame + fields::entity_frame::POS_Y)?;
            let z = memory::read_value::<f32>(frame + fields::entity_frame::POS_Z)?;
            let pos = Vec3::new(x, y, z);
            pos.is_finite().then_some(pos)
        }
    }
}

// =============================================================================
//  Направление (через vtable)
// =============================================================================

impl Player {
    /// Forward-вектор из frame matrix.
    pub fn get_forward(&self) -> Option<Vec3> {
        let frame = unsafe { self.human()?.actor.frame_node as usize };
        if !memory::is_valid_ptr(frame) {
            return None;
        }
        unsafe {
            let x = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_X)?;
            let y = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_Y)?;
            let z = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_Z)?;
            let v = Vec3::new(x, y, z);
            v.is_finite().then_some(v)
        }
    }

    /// Right-вектор из frame matrix.
    pub fn get_right(&self) -> Option<Vec3> {
        let frame = unsafe { self.human()?.actor.frame_node as usize };
        if !memory::is_valid_ptr(frame) {
            return None;
        }
        unsafe {
            let x = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_X)?;
            let y = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_Y)?;
            let z = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_Z)?;
            let v = Vec3::new(x, y, z);
            v.is_finite().then_some(v)
        }
    }

    /// Direction через vtable\[37\] `GetDir`.
    pub fn get_direction(&self) -> Option<Vec3> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = Vec3::ZERO;
            let ret = (vt.get_dir)(self.this_const(), &mut out);
            if ret.is_null() || !out.is_finite() {
                return None;
            }
            Some(out)
        }
    }

    /// Позиция головы через vtable\[43\] `GetHeadPos`.
    pub fn get_head_position(&self) -> Option<Vec3> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = Vec3::ZERO;
            let ret = (vt.get_head_pos)(self.this_const(), &mut out);
            if ret.is_null() || !out.is_finite() {
                return None;
            }
            Some(out)
        }
    }

    pub fn get_forward_vector(&self) -> Option<Vec3> {
        self.get_forward()
    }
}

// =============================================================================
//  Скорость (через vtable)
// =============================================================================

impl Player {
    /// Velocity через vtable\[68\] `GetVelocity`.
    pub fn get_velocity(&self) -> Option<Vec3> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = Vec3::ZERO;
            let ret = (vt.get_velocity)(self.this_const(), &mut out);
            if ret.is_null() || !out.is_finite() {
                return None;
            }
            Some(out)
        }
    }

    /// Текущая скорость через vtable\[76\].
    pub fn get_movement_speed(&self) -> Option<f32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.get_movement_speed_current)(self.this_const()))
        }
    }

    /// Целевая скорость (прямое чтение поля — vtable\[77\] только пишет).
    pub fn get_movement_speed_target(&self) -> Option<f32> {
        unsafe { self.human().map(|h| h.movement_speed_target) }
    }

    /// Установить скорость мгновенно через vtable\[75\].
    pub fn set_movement_speed(&self, speed: f32) -> bool {
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_movement_speed)(self.this_mut(), speed);
        }
        true
    }

    /// Установить целевую скорость через vtable\[77\].
    pub fn set_movement_speed_target(&self, speed: f32) -> bool {
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_movement_speed_target)(self.this_mut(), speed);
        }
        true
    }
}