//! Низкоуровневая typed-обёртка над native entity pointer.
//!
//! Это **не** Lua wrapper и **не** high-level gameplay API.
//! `EntityRef` — маленький primitive для:
//! - world dump
//! - entity inspector
//! - runtime validation
//! - перехода от сырого `usize` к типизированному доступу
//!
//! ## Подход Internal DLL
//!
//! Для подтверждённых layout'ов используем `repr(C)` структуры:
//! - [`CEntity`] для базовой части сущности
//! - [`CHuman`] для humanoid-полей
//!
//! Для неподтверждённых / неоформленных layout'ов (например frame node)
//! используем offsets из `addresses::fields` и функции из `memory.rs`.
//!
//! Это и есть правильный компромисс для internal SDK:
//! - confirmed layouts -> структуры
//! - provisional layouts -> fields + read_value/read_ptr

use super::{
    base,
    entity_types::{EntityType, FactoryType},
};
use crate::memory::Ptr;
use crate::structures::{CEntity, CHuman};
use crate::types::Vec3;
use crate::{addresses, memory};

use std::hash::{Hash, Hasher};

/// Typed-обёртка над native entity pointer (`C_Entity*`, `C_Human*`, `C_Car*`, ...).
///
/// Хранит уже провалидированный `usize`-адрес.
/// Дешёвый в копировании, поэтому подходит для массового использования
/// в сканере мира, инспекторе сущностей и devtools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityRef {
    ptr: Ptr<CEntity>,
}

impl Hash for EntityRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.addr().hash(state);
    }
}

impl EntityRef {
    // =============================================================================
    //  Создание
    // =============================================================================

    /// Создать `EntityRef` из сырого указателя с базовой валидацией.
    ///
    /// Проверяет:
    /// - адрес в user-mode диапазоне
    /// - наличие валидного `vtable`
    /// - `table_id != 0`
    /// - `table_id != 0xFFFF_FFFF`
    ///
    /// Это не доказывает на 100%, что перед нами корректный `CEntity`,
    /// но хорошо отсекает NULL, мусор и stale pointers.
    pub fn from_ptr(ptr: usize) -> Option<Self> {
        if ptr == 0 || !memory::is_valid_ptr(ptr) {
            return None;
        }

        let entity_ptr = Ptr::<CEntity>::new(ptr);

        unsafe {
            let entity = entity_ptr.as_ref()?;

            if !memory::is_valid_ptr(entity.vtable as usize) {
                return None;
            }

            if entity.table_id == 0 || entity.table_id == u32::MAX {
                return None;
            }
        }

        Some(Self { ptr: entity_ptr })
    }

    /// Создать `EntityRef` из script wrapper pointer.
    ///
    /// Читает native entity из `wrapper + 0x10`,
    /// затем валидирует его через [`from_ptr`](Self::from_ptr).
    pub fn from_wrapper_ptr(wrapper_ptr: usize) -> Option<Self> {
        if wrapper_ptr == 0 || !memory::is_valid_ptr(wrapper_ptr) {
            return None;
        }

        let native =
            unsafe { memory::read_ptr(wrapper_ptr + addresses::fields::script_wrapper::NATIVE)? };

        Self::from_ptr(native)
    }

    // =============================================================================
    //  Внутренние typed casts
    // =============================================================================

    /// Typed reference на базовую часть [`CEntity`].
    ///
    /// # Safety
    ///
    /// Объект должен быть жив. Для runtime инспекции в пределах одного кадра
    /// это обычно безопасно.
    #[inline]
    unsafe fn entity(&self) -> Option<&CEntity> {
        unsafe { self.ptr.as_ref() }
    }

    /// Typed reference на [`CHuman`].
    ///
    /// # Safety
    ///
    /// `self.ptr` должен указывать на объект с layout `CHuman`.
    #[inline]
    unsafe fn human(&self) -> Option<&CHuman> {
        if !self.is_humanoid() {
            return None;
        }

        if !self.ptr.is_valid() {
            return None;
        }

        Some(unsafe { &*(self.ptr.raw() as *const CHuman) })
    }

    // =============================================================================
    //  Базовые аксессоры
    // =============================================================================

    /// Сырой native pointer.
    #[inline]
    pub fn ptr(self) -> usize {
        self.ptr.addr()
    }

    pub fn typed_ptr(self) -> Ptr<CEntity> {
        self.ptr
    }

    /// Primary vtable pointer.
    pub fn vtable(self) -> Option<usize> {
        unsafe { self.entity().map(|e| e.vtable as usize) }
    }

    /// Packed `table_id = (instance_id << 8) | factory_type_byte`.
    pub fn table_id(self) -> Option<u32> {
        unsafe { self.entity().map(|e| e.table_id) }
    }

    /// Factory type byte — младший байт `table_id`.
    pub fn factory_type_byte(self) -> Option<u8> {
        self.table_id().map(|tid| (tid & 0xFF) as u8)
    }

    /// Typed factory type.
    pub fn factory_type(self) -> Option<FactoryType> {
        FactoryType::from_byte(self.factory_type_byte()?)
    }

    /// Instance id — старшие 24 бита `table_id`.
    pub fn instance_id(self) -> Option<u32> {
        self.table_id().map(|tid| tid >> 8)
    }

    /// Entity flags (`entity + 0x28`).
    pub fn entity_flags(self) -> Option<u32> {
        unsafe { self.entity().map(|e| e.entity_flags) }
    }

    /// FNV-1 64-bit name hash (`entity + 0x30`).
    ///
    /// Ноль обычно означает безымянную сущность.
    pub fn name_hash(self) -> Option<u64> {
        unsafe { self.entity().map(|e| e.name_hash) }
    }

    // =============================================================================
    //  Actor-like поля
    // =============================================================================

    /// `frame node` pointer (`entity + 0x78`).
    ///
    /// Это уже слой `C_Actor`, а не `CEntity`, поэтому пока читаем
    /// через offset. Отдельной полной `repr(C)` структуры frame node
    /// у нас ещё нет.
    pub fn frame_node(self) -> Option<usize> {
        let raw = unsafe {
            self.ptr
                .read_at::<usize>(addresses::fields::entity::FRAME_NODE)?
        };
        memory::is_valid_ptr(raw).then_some(raw)
    }

    /// Owner / container pointer (`entity + 0x80`).
    ///
    /// Для humanoid:
    /// - `None` -> пешком
    /// - `Some(ptr)` -> внутри транспорта
    ///
    /// Для generic `EntityRef` читается через offset,
    /// потому что это уже actor-layer, а не чистый `CEntity`.
    pub fn owner(self) -> Option<usize> {
        let raw = unsafe {
            self.ptr
                .read_at::<usize>(addresses::fields::entity::OWNER)?
        };
        if raw == 0 {
            None
        } else {
            memory::is_valid_ptr(raw).then_some(raw)
        }
    }

    /// Позиция entity через `frame node`.
    ///
    /// Читает:
    /// - `frame + 0x64` -> X
    /// - `frame + 0x74` -> Y
    /// - `frame + 0x84` -> Z
    ///
    /// Возвращает `None`, если:
    /// - `frame_node == NULL`
    /// - координаты не finite
    pub fn position(self) -> Option<Vec3> {
        use crate::addresses::fields::entity_frame;

        let frame = self.frame_node()?;

        let x = unsafe { memory::read_value::<f32>(frame + entity_frame::POS_X)? };
        let y = unsafe { memory::read_value::<f32>(frame + entity_frame::POS_Y)? };
        let z = unsafe { memory::read_value::<f32>(frame + entity_frame::POS_Z)? };

        let pos = Vec3 { x, y, z };
        if pos.is_finite() { Some(pos) } else { None }
    }

    // =============================================================================
    //  Типизация
    // =============================================================================

    /// Lua-facing тип сущности, если для данного factory type есть mapping.
    pub fn lua_entity_type(self) -> Option<EntityType> {
        let ft = self.factory_type()?;
        EntityType::from_factory_type(ft as u8)
    }

    /// Является ли сущность humanoid (`HumanNpc` или `Player`).
    pub fn is_humanoid(self) -> bool {
        matches!(self.factory_type(), Some(ft) if ft.is_humanoid())
    }

    /// Является ли сущность транспортом.
    pub fn is_vehicle(self) -> bool {
        matches!(self.factory_type(), Some(ft) if ft.is_vehicle())
    }

    // =============================================================================
    //  Humanoid-only поля (через CHuman)
    // =============================================================================

    /// Текущее здоровье. Только для humanoid.
    ///
    /// Читается через подтверждённый layout [`CHuman`],
    /// а не через `fields::player::CURRENT_HEALTH`.
    pub fn health(self) -> Option<f32> {
        if !self.is_humanoid() {
            return None;
        }

        unsafe { self.human().map(|h| h.current_health) }
    }

    /// Жив ли персонаж. Только для humanoid.
    ///
    /// Использует `CHuman.is_dead`.
    pub fn is_alive(self) -> Option<bool> {
        if !self.is_humanoid() {
            return None;
        }

        unsafe { self.human().map(|h| h.is_dead == 0) }
    }

    /// Находится ли humanoid в транспорте.
    ///
    /// Использует `CHuman.owner`, а не raw offset.
    pub fn is_in_vehicle(self) -> Option<bool> {
        if !self.is_humanoid() {
            return None;
        }

        unsafe { self.human().map(|h| !h.actor.owner.is_null()) }
    }

    // =============================================================================
    //  Диагностика
    // =============================================================================

    /// Короткая строка для логов.
    pub fn debug_label(self) -> String {
        let ft = self.factory_type_byte().unwrap_or(0);
        let tid = self.table_id().unwrap_or(0);
        let iid = self.instance_id().unwrap_or(0);
        let vt = self.vtable().unwrap_or(0);

        format!(
            "EntityRef(ptr=0x{:X}, type=0x{:02X}, tid=0x{:08X}, inst={}, vt=0x{:X})",
            self.ptr.addr(),
            ft,
            tid,
            iid,
            vt
        )
    }

    /// Проверка что vtable находится в `.rdata` секции модуля игры.
    ///
    /// Это эвристика: useful для отлова мусорных / stale pointers.
    pub fn has_probably_valid_vtable(self) -> bool {
        let Some(vtable) = self.vtable() else {
            return false;
        };

        let rva = vtable.wrapping_sub(base());
        (0x1800000..=0x1C50000).contains(&rva)
    }
}
