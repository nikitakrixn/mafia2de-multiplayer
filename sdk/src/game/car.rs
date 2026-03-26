//! Высокоуровневый API для работы с машинами.
//!
//! ## Два пути доступа к данным
//!
//! ```text
//! Ptr<CCar> -> as_ref() -> &CCar      — поля из structures/car.rs (compile-time checked)
//! Ptr<CCar> -> read_at::<T>(offset)   — поля базового класса CEntity (ещё не в CCar)
//! ```
//!
//! Когда поле перенесено в `structures::CCar` — обязательно переключиться
//! на доступ через `as_ref()`, чтобы compile-time ассерты защищали от дрифта.

use crate::addresses::fields::entity as entity_fields;
use crate::{addresses, memory};
use crate::memory::Ptr;
use crate::structures::CCar;

use super::base;
use crate::types::Vec3;

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
    /// Проверяет:
    /// - адрес в user-mode диапазоне
    /// - factory type byte == 0x12 (`C_Car`)
    pub fn from_ptr(addr: usize) -> Option<Self> {
        if addr == 0 || !memory::is_valid_ptr(addr) {
            return None;
        }

        let ptr = Ptr::<CCar>::new(addr);

        // table_id — поле базового класса CEntity (+0x24)
        let table_id = unsafe { ptr.read_at::<u32>(entity_fields::TABLE_ID)? };
        if (table_id & 0xFF) != 0x12 {
            return None;
        }

        Some(Self { ptr })
    }

    /// Типизированный указатель.
    pub fn ptr(&self) -> Ptr<CCar> {
        self.ptr
    }

    /// Сырой адрес (для логирования, сравнения).
    pub fn addr(&self) -> usize {
        self.ptr.addr()
    }

    // -------------------------------------------------------------------------
    //  Доступ через структуру CCar (compile-time проверенные смещения)
    // -------------------------------------------------------------------------

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
    ///
    /// Подтверждено: `CCar_SetPos` / `CCar_SetRotation` выставляют `car_flags |= 0x1000`.
    pub fn is_dirty(&self) -> bool {
        unsafe {
            self.ptr
                .as_ref()
                .map(|car| car.is_dirty())
                .unwrap_or(false)
        }
    }

    /// Variant index.
    pub fn get_variant_index(&self) -> Option<u32> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.variant_index)
    }

    /// Entity subtype (`0x36`, `0x37`, `0x3A` для разных `C_Car`).
    pub fn get_entity_subtype(&self) -> Option<u32> {
        let car = unsafe { self.ptr.as_ref()? };
        Some(car.entity_subtype)
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
    ///
    /// Подтверждено: slot[67] = `(end - begin) / 24`.
    pub fn record_count(&self) -> usize {
        unsafe {
            self.ptr
                .as_ref()
                .map(|car| car.record_count())
                .unwrap_or(0)
        }
    }

    // -------------------------------------------------------------------------
    //  Доступ через сырые смещения (поля базового класса CEntity)
    // -------------------------------------------------------------------------

    /// Packed `table_id` из базового класса `CEntity`.
    ///
    /// Младший байт = factory type, старшие 24 бита = instance id.
    pub fn get_table_id(&self) -> Option<u32> {
        unsafe { self.ptr.read_at::<u32>(entity_fields::TABLE_ID) }
    }
}

// =============================================================================
//  EntityDatabase scan
// =============================================================================

/// Оценивает количество бакетов в hash-таблице EntityDatabase.
///
/// EntityDatabase использует open-addressing. Bucket count обычно
/// степень двойки в диапазоне 256..16384.
fn estimate_entity_db_bucket_count(db: usize) -> Option<usize> {
    let entity_count = unsafe { memory::read::<u64>(db + 0x18)? } as usize;

    if entity_count == 0 || entity_count > 100_000 {
        return None;
    }

    // Пробуем прочитать bucket count из известных смещений
    let candidate_20 = unsafe { memory::read::<u32>(db + 0x20).unwrap_or(0) } as usize;
    if candidate_20.is_power_of_two() && (256..=16384).contains(&candidate_20) {
        return Some(candidate_20);
    }

    let candidate_30 = unsafe { memory::read::<u32>(db + 0x30).unwrap_or(0) } as usize;
    if candidate_30.is_power_of_two() && (256..=16384).contains(&candidate_30) {
        return Some(candidate_30);
    }

    // Fallback: оценка из количества entity
    Some((entity_count * 2).next_power_of_two().clamp(256, 8192))
}

/// Полный скан всех `C_Car` через EntityDatabase.
///
/// Проходит по open-addressing hash-таблице, проверяет factory type
/// каждой entity, собирает все `C_Car` (type = `0x12`).
///
/// ## ВАЖНО
///
/// Вызывать только из game thread — EntityDatabase не потокобезопасна.
///
/// ## Как работает EntityDatabase
///
/// ```text
/// g_EntityDatabase (singleton)
///   +0x18: entity_count (u64)
///   +0x38: hash_table[] (inline, NOT через pointer)
///           каждый slot = entity_ptr (u64), 0 = пустой
/// ```
pub fn scan_all_cars() -> Vec<Car> {
    let mut out = Vec::new();

    // Читаем синглтон EntityDatabase
    let Some(db) = (unsafe {
        memory::read_validated_ptr(base() + addresses::globals::ENTITY_DATABASE)
    }) else {
        return out;
    };

    // Количество entity в базе
    let Some(entity_count) =
        (unsafe { memory::read::<u64>(db + 0x18) }).map(|v| v as usize)
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

        // Безопасное чтение через VirtualQuery — slot может быть за пределами
        // committed-области если bucket_count оценён с запасом
        let entity_ptr = match unsafe { memory::read_safe::<u64>(slot) } {
            Some(v) if v != 0 => v as usize,
            _ => continue,
        };

        // Двойная проверка: адрес + первые 0x30 байт доступны
        if !memory::is_valid_ptr(entity_ptr) || !memory::is_readable(entity_ptr, 0x30) {
            continue;
        }

        // Читаем table_id через VirtualQuery (подозрительные указатели из hash table)
        let table_id =
            match unsafe { memory::read_safe::<u32>(entity_ptr + entity_fields::TABLE_ID) } {
                Some(v) => v,
                None => continue,
            };

        seen_entities += 1;

        // Factory type 0x12 = C_Car
        if (table_id & 0xFF) == 0x12 {
            if let Some(car) = Car::from_ptr(entity_ptr) {
                out.push(car);
            }
        }

        // Early stop: все entity из DB count уже просмотрены
        if seen_entities >= entity_count {
            break;
        }
    }

    out
}