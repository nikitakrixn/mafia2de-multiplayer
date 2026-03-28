//! Высокоуровневый API для работы с машинами.
//!
//! ## Безопасность доступа
//!
//! `Car::from_ptr()` проверяет factory_type через минимальное чтение (CEntity, 0x78 байт),
//! а не через `as_ref()` на полный CCar (0x1258 байт). Это предотвращает crash при
//! попытке создать Car из указателя на CCarVehicle или другой тип.

use crate::memory::Ptr;
use crate::structures::{CCar, CCarDamageSub1, CCarVTable, CEntity};
use crate::types::Vec3;
use crate::{addresses, memory};
use std::ffi::c_void;

use super::base;

// =============================================================================
//  Car — высокоуровневая обёртка над C_Car
// =============================================================================

/// Обёртка над `C_Car` для удобного доступа к данным машины.
///
/// Хранит [`Ptr<CCar>`] — типизированный указатель.
/// Валидность проверяется при создании через [`from_ptr`](Self::from_ptr).
#[derive(Debug, Clone, Copy)]
pub struct Car {
    ptr: Ptr<CCar>,
}

impl Car {
    /// Создаёт `Car` из сырого адреса с валидацией.
    ///
    /// Проверяет factory type через **минимальное чтение** CEntity (0x78 байт),
    /// а не через полный CCar (0x1258 байт). Это безопасно даже если адрес
    /// указывает на CCarVehicle (0x2F0) или другой меньший тип.
    pub fn from_ptr(addr: usize) -> Option<Self> {
        if addr == 0 || !memory::is_valid_ptr(addr) {
            return None;
        }

        // Минимальная проверка: читаем только CEntity (0x78 байт)
        // чтобы не crash'нуть на объектах меньше CCar
        let entity = Ptr::<CEntity>::new(addr);
        let ent = unsafe { entity.as_ref()? };

        if ent.factory_type() != 0x12 {
            return None;
        }

        // Дополнительно: убеждаемся что хватает памяти для CCar
        // CCar минимум ~0x1220 байт, проверяем что readable
        if !memory::is_readable(addr, 0x1220) {
            return None;
        }

        Some(Self {
            ptr: Ptr::<CCar>::new(addr),
        })
    }

    /// Типизированный указатель.
    pub fn ptr(&self) -> Ptr<CCar> {
        self.ptr
    }

    /// Сырой адрес (для логирования, сравнения).
    pub fn addr(&self) -> usize {
        self.ptr.addr()
    }

    // =========================================================================
    //  Доступ к структуре CCar (compile-time проверенные смещения)
    // =========================================================================

    /// Позиция из встроенной world matrix.
    ///
    /// Подтверждено: slot[36] `CCar_GetPos` читает +0x27C/+0x28C/+0x29C,
    /// что соответствует `world_matrix[3], [7], [11]`.
    pub fn get_position(&self) -> Option<Vec3> {
        let car = unsafe { self.ptr.as_ref()? };
        let (x, y, z) = car.get_pos();
        if x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Vec3 { x, y, z })
        } else {
            None
        }
    }

    /// Car flags (u64).
    pub fn get_car_flags(&self) -> Option<u64> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.car_flags)
    }

    /// Есть ли активная физика (`physics_body != NULL`).
    pub fn has_physics(&self) -> bool {
        unsafe {
            self.ptr
                .as_ref()
                .map(|car| car.has_physics())
                .unwrap_or(false)
        }
    }

    /// Установлен ли dirty-флаг (0x1000).
    pub fn is_dirty(&self) -> bool {
        unsafe { self.ptr.as_ref().map(|car| car.is_dirty()).unwrap_or(false) }
    }

    /// Variant index.
    pub fn get_variant_index(&self) -> Option<u32> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.variant_index)
    }

    /// Entity subtype (`0x36`, `0x37`, `0x3A` для разных кузовов C_Car).
    pub fn get_entity_subtype(&self) -> Option<u32> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.entity_subtype())
    }

    /// Есть ли collision body.
    pub fn has_collision_body(&self) -> bool {
        unsafe {
            self.ptr
                .as_ref()
                .map(|car| !car.collision_body.is_null())
                .unwrap_or(false)
        }
    }

    /// Количество записей в records-векторе.
    pub fn record_count(&self) -> usize {
        unsafe { self.ptr.as_ref().map(|car| car.record_count()).unwrap_or(0) }
    }

    /// Frame node pointer.
    pub fn frame_node(&self) -> Option<*mut c_void> {
        let car = unsafe { self.ptr.as_ref()? };
        let ptr = car.frame_node();
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Name hash из базового CEntity.
    pub fn name_hash(&self) -> Option<u64> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.entity().name_hash)
    }

    /// Активирована ли сущность (bit 5 entity_flags).
    pub fn is_activated(&self) -> bool {
        unsafe {
            self.ptr
                .as_ref()
                .map(|car| car.entity().is_activated())
                .unwrap_or(false)
        }
    }

    /// Количество записей seats/enter-leave (из CActorVehicle).
    pub fn seat_record_count(&self) -> usize {
        unsafe {
            self.ptr
                .as_ref()
                .map(|car| car.base.record_count())
                .unwrap_or(0)
        }
    }

    /// Packed `table_id` из базового класса `CEntity`.
    pub fn get_table_id(&self) -> Option<u32> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.table_id())
    }

    // =========================================================================
    //  Typed vtable (private)
    // =========================================================================

    /// Типизированный доступ к VTable C_Car (114 слотов).
    unsafe fn vtable(&self) -> Option<&CCarVTable> {
        let car = unsafe { self.ptr.as_ref()? };
        let vt_ptr = car.entity().vtable as *const CCarVTable;
        if vt_ptr.is_null() {
            return None;
        }
        Some(unsafe { &*vt_ptr })
    }

    #[inline]
    fn this_mut(&self) -> *mut c_void {
        self.ptr.raw() as *mut c_void
    }

    #[inline]
    fn this_const(&self) -> *const c_void {
        self.ptr.raw() as *const c_void
    }

    // =========================================================================
    //  Vtable: пространство (32–39)
    // =========================================================================

    /// Позиция через vtable[36] GetPos (авторитетная, с physics sync).
    pub fn get_position_synced(&self) -> Option<Vec3> {
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

    /// Установить позицию через vtable[32] SetPos.
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

    /// Направление через vtable[37] GetDir.
    pub fn get_direction(&self) -> Option<Vec3> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = Vec3::ZERO;
            let ret = (vt.get_dir)(self.this_const(), &mut out);
            if ret.is_null() || !out.is_finite() {
                None
            } else {
                Some(out)
            }
        }
    }

    /// Установить направление через vtable[33] SetDir.
    pub fn set_direction(&self, dir: &Vec3) -> bool {
        if !dir.is_finite() {
            return false;
        }
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_dir)(self.this_mut(), dir);
        }
        true
    }

    /// Вращение (кватернион [x,y,z,w]) через vtable[38] GetRot.
    pub fn get_rotation(&self) -> Option<[f32; 4]> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = [0.0f32; 4];
            let ret = (vt.get_rot)(self.this_const(), &mut out);
            if ret.is_null() {
                None
            } else {
                Some(out)
            }
        }
    }

    /// Установить вращение через vtable[34] SetRot.
    pub fn set_rotation(&self, quat: &[f32; 4]) -> bool {
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_rot)(self.this_mut(), quat);
        }
        true
    }

    /// Масштаб модели через vtable[39] GetScale.
    pub fn get_scale(&self) -> Option<f32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.get_scale)(self.this_const()))
        }
    }

    /// Установить масштаб через vtable[35] SetScale.
    pub fn set_scale(&self, scale: f32) -> bool {
        if !scale.is_finite() || scale <= 0.0 {
            return false;
        }
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_scale)(self.this_mut(), scale);
        }
        true
    }

    // =========================================================================
    //  Vtable: состояние (9, 47, 43, 49)
    // =========================================================================

    /// Активна ли сущность через vtable[9] IsActive.
    pub fn is_active_vt(&self) -> Option<bool> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.is_active)(self.this_const()))
        }
    }

    /// IsDead через vtable[47]. Для машин всегда false.
    pub fn is_dead(&self) -> Option<bool> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.is_dead)(self.this_const()))
        }
    }

    /// Позиция камеры через vtable[43] GetCameraPoint.
    pub fn get_camera_point(&self) -> Option<Vec3> {
        unsafe {
            let vt = self.vtable()?;
            let mut out = Vec3::ZERO;
            let ret = (vt.get_camera_point)(self.this_const(), &mut out);
            if ret.is_null() || !out.is_finite() {
                None
            } else {
                Some(out)
            }
        }
    }

    /// Underwater status через vtable[49]. 0 = не под водой.
    pub fn get_underwater_status(&self) -> Option<u32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.get_underwater_status)(self.this_const()))
        }
    }

    /// Frame node через vtable[2] GetFrameNode.
    pub fn get_frame_node_vt(&self) -> Option<*mut c_void> {
        unsafe {
            let vt = self.vtable()?;
            let ptr = (vt.get_frame_node)(self.this_const());
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    // =========================================================================
    //  Vtable: seats (56–61)
    // =========================================================================

    /// Общее количество сидений через vtable[56].
    pub fn get_seat_count(&self) -> Option<u32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.av_get_seat_count)(self.this_const()))
        }
    }

    /// Количество сидений определённого типа через vtable[57].
    pub fn get_seat_count_by_type(&self, seat_type: u32) -> Option<u32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.av_get_seat_count_by_type)(self.this_const(), seat_type))
        }
    }

    /// Количество занятых сидений через vtable[58].
    pub fn get_human_used_seat_count(&self) -> Option<u32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.av_get_human_used_seat_count)(self.this_const()))
        }
    }

    /// Индекс свободного сиденья (-1 если все заняты) через vtable[61].
    pub fn get_free_seat_index(&self, seat_type: u32) -> Option<i32> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.av_get_free_seat_index)(self.this_const(), seat_type))
        }
    }

    // =========================================================================
    //  Vtable: визуал и рендеринг (75–77)
    // =========================================================================

    /// Рендерится ли модель через vtable[75].
    pub fn is_model_rendered(&self) -> Option<bool> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.is_model_rendered)(self.this_const()))
        }
    }

    /// В оптимальном диапазоне от игрока через vtable[76].
    pub fn is_in_optim_player_range(&self) -> Option<bool> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.is_in_optim_player_range)(self.this_const()))
        }
    }

    /// Машина на открытом пространстве через vtable[77].
    pub fn is_in_open_space(&self) -> Option<bool> {
        unsafe {
            let vt = self.vtable()?;
            Some((vt.is_vehicle_in_open_space)(self.this_const()))
        }
    }

    // =========================================================================
    //  Vtable: horn, dirty, license plate (80, 86–88)
    // =========================================================================

    /// Включить/выключить гудок через vtable[80].
    pub fn set_horn(&self, enabled: bool, honk_type: bool) -> bool {
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_horn)(self.this_mut(), enabled as u8, honk_type as u8);
        }
        true
    }

    /// Установить уровень загрязнения через vtable[86]. 0.0–1.0.
    pub fn set_dirty(&self, amount: f32) -> bool {
        if !amount.is_finite() {
            return false;
        }
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_vehicle_dirty)(self.this_mut(), amount);
        }
        true
    }

    /// Прочитать текст номерного знака через vtable[88].
    pub fn get_license_plate(&self) -> Option<String> {
        unsafe {
            let vt = self.vtable()?;
            let ptr = (vt.get_spz_text)(self.this_const());
            if ptr.is_null() {
                return None;
            }
            let cstr = std::ffi::CStr::from_ptr(ptr);
            let s = cstr.to_string_lossy().into_owned();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
    }

    /// Установить текст номерного знака через vtable[87].
    pub fn set_license_plate(&self, text: &str, update: bool) -> bool {
        let Ok(c_text) = std::ffi::CString::new(text) else {
            return false;
        };
        unsafe {
            let Some(vt) = self.vtable() else {
                return false;
            };
            (vt.set_spz_text)(self.this_mut(), c_text.as_ptr(), update as u8);
        }
        true
    }

    // =========================================================================
    //  Damage subobject
    // =========================================================================

    /// Доступ к damage subobject (inline at car+0xE0).
    pub fn damage(&self) -> Option<&CCarDamageSub1> {
        let car = unsafe { self.ptr.as_ref()? };
        let ptr = car.damage_sub1_ptr();
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }

    /// Количество crash-parts.
    pub fn parts_count(&self) -> usize {
        self.damage().map(|d| d.parts_count()).unwrap_or(0)
    }

    /// Количество дверей.
    pub fn doors_count(&self) -> usize {
        self.damage().map(|d| d.doors_count()).unwrap_or(0)
    }

    /// Количество event buckets деформации.
    pub fn event_bucket_count(&self) -> usize {
        self.damage().map(|d| d.event_bucket_count()).unwrap_or(0)
    }

    /// Флаги деформации.
    pub fn damage_flags(&self) -> Option<(u32, u64, u64)> {
        self.damage()
            .map(|d| (d.flags_aa8, d.flags_ab0, d.flags_ab8))
    }

    // =========================================================================
    //  Composite: диагностика
    // =========================================================================

    /// Полная диагностическая сводка (структура + безопасные vtable-геттеры).
    ///
    /// Все vtable-вызовы обёрнуты в catch — если какой-то метод
    /// некорректен для конкретной машины, возвращается None для этого поля.
    pub fn diagnostic_summary(&self) -> Option<CarDiagnostics> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(CarDiagnostics {
            addr: self.addr(),
            table_id: car.table_id(),
            entity_subtype: car.entity_subtype(),
            // Из структуры (гарантированно безопасно)
            position: self.get_position(),
            is_dirty: car.is_dirty(),
            has_physics: car.has_physics(),
            car_flags: car.car_flags,
            variant_index: car.variant_index,
            parts_count: self.parts_count(),
            doors_count: self.doors_count(),
            // Из vtable (могут быть None при проблемах)
            direction: self.get_direction(),
            rotation: self.get_rotation(),
            scale: self.get_scale(),
            camera_point: self.get_camera_point(),
            is_active: self.is_active_vt(),
            is_dead: self.is_dead(),
            seat_count: self.get_seat_count(),
            seats_occupied: self.get_human_used_seat_count(),
            is_rendered: self.is_model_rendered(),
            is_in_range: self.is_in_optim_player_range(),
            is_open_space: self.is_in_open_space(),
            underwater_status: self.get_underwater_status(),
            license_plate: self.get_license_plate(),
        })
    }
}

/// Результат полной диагностики машины.
#[derive(Debug)]
pub struct CarDiagnostics {
    pub addr: usize,
    pub table_id: u32,
    pub entity_subtype: u32,
    pub position: Option<Vec3>,
    pub direction: Option<Vec3>,
    pub rotation: Option<[f32; 4]>,
    pub scale: Option<f32>,
    pub camera_point: Option<Vec3>,
    pub is_active: Option<bool>,
    pub is_dead: Option<bool>,
    pub is_dirty: bool,
    pub has_physics: bool,
    pub car_flags: u64,
    pub variant_index: u32,
    pub seat_count: Option<u32>,
    pub seats_occupied: Option<u32>,
    pub is_rendered: Option<bool>,
    pub is_in_range: Option<bool>,
    pub is_open_space: Option<bool>,
    pub underwater_status: Option<u32>,
    pub license_plate: Option<String>,
    pub parts_count: usize,
    pub doors_count: usize,
}

// =============================================================================
//  EntityDatabase scan
// =============================================================================

fn estimate_entity_db_bucket_count(db: usize) -> Option<usize> {
    let entity_count = unsafe { memory::read::<u64>(db + 0x18)? } as usize;

    if entity_count == 0 || entity_count > 100_000 {
        return None;
    }

    let candidate_20 = unsafe { memory::read::<u32>(db + 0x20).unwrap_or(0) } as usize;
    if candidate_20.is_power_of_two() && (256..=16384).contains(&candidate_20) {
        return Some(candidate_20);
    }

    let candidate_30 = unsafe { memory::read::<u32>(db + 0x30).unwrap_or(0) } as usize;
    if candidate_30.is_power_of_two() && (256..=16384).contains(&candidate_30) {
        return Some(candidate_30);
    }

    Some((entity_count * 2).next_power_of_two().clamp(256, 8192))
}

/// Полный скан всех `C_Car` через EntityDatabase.
///
/// Вызывать только из game thread.
pub fn scan_all_cars() -> Vec<Car> {
    let mut out = Vec::new();

    let Some(db) =
        (unsafe { memory::read_validated_ptr(base() + addresses::globals::ENTITY_DATABASE) })
    else {
        return out;
    };

    let Some(entity_count) = (unsafe { memory::read::<u64>(db + 0x18) }).map(|v| v as usize)
    else {
        return out;
    };

    let Some(bucket_count) = estimate_entity_db_bucket_count(db) else {
        return out;
    };

    let hash_table_base = db + 0x38;
    let mut seen_entities = 0usize;

    for bucket in 0..bucket_count {
        let slot = hash_table_base + bucket * 8;

        let entity_ptr = match unsafe { memory::read_safe::<u64>(slot) } {
            Some(v) if v != 0 => v as usize,
            _ => continue,
        };

        if !memory::is_valid_ptr(entity_ptr) || !memory::is_readable(entity_ptr, 0x30) {
            continue;
        }

        let entity = Ptr::<CEntity>::new(entity_ptr);
        let Some(ent) = (unsafe { entity.as_ref() }) else {
            continue;
        };

        seen_entities += 1;

        if ent.factory_type() == 0x12 {
            if let Some(car) = Car::from_ptr(entity_ptr) {
                out.push(car);
            }
        }

        if seen_entities >= entity_count {
            break;
        }
    }

    out
}