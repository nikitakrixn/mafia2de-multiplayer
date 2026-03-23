use crate::{addresses, memory};

use super::base;
use super::player::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Car {
    ptr: usize,
}

impl Car {
    pub fn from_ptr(ptr: usize) -> Option<Self> {
        if ptr == 0 || !unsafe { memory::is_valid_ptr(ptr) } {
            return None;
        }

        let table_id = unsafe { memory::read_value::<u32>(ptr + 0x24)? };
        if (table_id & 0xFF) != 0x12 {
            return None;
        }

        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> usize {
        self.ptr
    }

    /// Позиция из встроенной матрицы C_Car.
    /// Подтверждено: slot[36] CCar_GetPos читает +0x27C/+0x28C/+0x29C.
    pub fn get_position(&self) -> Option<Vec3> {
        unsafe {
            let x = memory::read_value::<f32>(self.ptr + 0x27C)?;
            let y = memory::read_value::<f32>(self.ptr + 0x28C)?;
            let z = memory::read_value::<f32>(self.ptr + 0x29C)?;
            if x.is_finite() && y.is_finite() && z.is_finite() {
                Some(Vec3 { x, y, z })
            } else {
                None
            }
        }
    }

    pub fn get_table_id(&self) -> Option<u32> {
        unsafe { memory::read_value::<u32>(self.ptr + 0x24) }
    }

    pub fn get_entity_subtype(&self) -> Option<u32> {
        unsafe { memory::read_value::<u32>(self.ptr + 0xA0) }
    }

    pub fn get_car_flags(&self) -> Option<u64> {
        unsafe { memory::read_value::<u64>(self.ptr + 0xF30) }
    }

    pub fn has_physics(&self) -> bool {
        self.get_car_flags().map(|f| (f & 1) != 0).unwrap_or(false)
    }

    pub fn is_dirty(&self) -> bool {
        self.get_car_flags().map(|f| (f & 0x1000) != 0).unwrap_or(false)
    }

    pub fn get_variant_index(&self) -> Option<u32> {
        unsafe { memory::read_value::<u32>(self.ptr + 0xF88) }
    }

    pub fn has_collision_body(&self) -> bool {
        unsafe {
            memory::read_ptr(self.ptr + 0x1210)
                .map(|p| p != 0)
                .unwrap_or(false)
        }
    }

    pub fn record_count(&self) -> usize {
        unsafe {
            let begin = memory::read_ptr_raw(self.ptr + 0x0C8).unwrap_or(0);
            let end = memory::read_ptr_raw(self.ptr + 0x0D0).unwrap_or(0);
            if begin != 0 && end > begin {
                (end - begin) / 24
            } else {
                0
            }
        }
    }
}

fn estimate_entity_db_bucket_count(db: usize) -> Option<usize> {
    let entity_count = unsafe { memory::read_value::<u64>(db + 0x18)? } as usize;

    if entity_count == 0 || entity_count > 100_000 {
        return None;
    }

    let candidate_20 = unsafe { memory::read_value::<u32>(db + 0x20).unwrap_or(0) } as usize;
    if candidate_20.is_power_of_two() && (256..=16384).contains(&candidate_20) {
        return Some(candidate_20);
    }

    let candidate_30 = unsafe { memory::read_value::<u32>(db + 0x30).unwrap_or(0) } as usize;
    if candidate_30.is_power_of_two() && (256..=16384).contains(&candidate_30) {
        return Some(candidate_30);
    }

    Some((entity_count * 2).next_power_of_two().clamp(256, 8192))
}

/// Полный безопасный скан всех C_Car через EntityDatabase.
///
/// ВАЖНО:
/// вызывать только с game thread.
pub fn scan_all_cars() -> Vec<Car> {
    let mut out = Vec::new();

    let Some(db) = (unsafe {
        memory::read_ptr(base() + addresses::globals::ENTITY_DATABASE)
    }) else {
        return out;
    };

    let Some(entity_count) = (unsafe {
        memory::read_value::<u64>(db + 0x18)
    }).map(|v| v as usize) else {
        return out;
    };

    let Some(bucket_count) = estimate_entity_db_bucket_count(db) else {
        return out;
    };

    // ВАЖНО: таблица inline, а НЕ через ptr
    let hash_table_base = db + 0x38;

    let mut seen_entities = 0usize;

    for bucket in 0..bucket_count {
        let slot = hash_table_base + bucket * 8;

        let entity_ptr = match unsafe { memory::read_value_safe::<u64>(slot) } {
            Some(v) if v != 0 => v as usize,
            _ => continue,
        };

        if !memory::is_valid_ptr(entity_ptr) || !memory::is_readable(entity_ptr, 0x30) {
            continue;
        }

        let table_id = match unsafe { memory::read_value_safe::<u32>(entity_ptr + 0x24) } {
            Some(v) => v,
            None => continue,
        };

        seen_entities += 1;

        if (table_id & 0xFF) == 0x12 {
            if let Some(car) = Car::from_ptr(entity_ptr) {
                out.push(car);
            }
        }

        // Early stop: если уже увидели все entity из DB count
        if seen_entities >= entity_count {
            break;
        }
    }

    out
}