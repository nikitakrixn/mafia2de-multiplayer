//! Низкоуровневая typed-обёртка над native entity pointer.
//!
//! Это НЕ Lua wrapper и НЕ high-level gameplay API.
//! Это безопасный слой поверх `usize`, который умеет:
//! - читать packed table_id
//! - определять FactoryType
//! - читать vtable / flags / name_hash
//! - брать позицию из frame node
//! - читать базовые human-only поля
//!
//! Удобен как единый primitive для:
//! - world dump
//! - entity inspector
//! - npc / player helpers
//! - runtime validation

use super::{
    base,
    entity_types::{EntityType, FactoryType},
    player::Vec3,
};
use crate::{addresses, memory};

/// Обёртка над native entity pointer (`C_Entity*`, `C_Human*`, `C_Car*`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityRef {
    ptr: usize,
}

impl EntityRef {
    /// Создать `EntityRef` из сырого указателя.
    ///
    /// Возвращает `None`, если указатель не похож на валидную сущность.
    pub fn from_ptr(ptr: usize) -> Option<Self> {
        if ptr == 0 || !memory::is_valid_ptr(ptr) {
            return None;
        }

        unsafe {
            let vtable = memory::read_ptr(ptr)?;
            if !memory::is_valid_ptr(vtable) {
                return None;
            }

            let table_id = memory::read_value::<u32>(ptr + addresses::fields::entity::TABLE_ID)?;
            if table_id == 0 || table_id == u32::MAX {
                return None;
            }
        }

        Some(Self { ptr })
    }

    /// Создать `EntityRef` из script wrapper pointer.
    pub fn from_wrapper_ptr(wrapper_ptr: usize) -> Option<Self> {
        if wrapper_ptr == 0 || !memory::is_valid_ptr(wrapper_ptr) {
            return None;
        }

        let native =
            unsafe { memory::read_ptr(wrapper_ptr + addresses::fields::script_wrapper::NATIVE)? };

        Self::from_ptr(native)
    }

    /// Сырой указатель на native entity.
    pub fn ptr(self) -> usize {
        self.ptr
    }

    /// Vtable pointer.
    pub fn vtable(self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr) }
    }

    /// Packed table_id.
    ///
    /// Формат:
    /// - low byte   = factory type
    /// - upper 24b  = instance id
    pub fn table_id(self) -> Option<u32> {
        unsafe { memory::read_value::<u32>(self.ptr + addresses::fields::entity::TABLE_ID) }
    }

    /// Factory type byte из packed table_id.
    pub fn factory_type_byte(self) -> Option<u8> {
        self.table_id().map(|tid| (tid & 0xFF) as u8)
    }

    /// FactoryType enum.
    pub fn factory_type(self) -> Option<FactoryType> {
        FactoryType::from_byte(self.factory_type_byte()?)
    }

    /// Instance id (upper 24 bits table_id).
    pub fn instance_id(self) -> Option<u32> {
        self.table_id().map(|tid| tid >> 8)
    }

    /// Entity flags (`entity + 0x28`).
    pub fn entity_flags(self) -> Option<u32> {
        unsafe { memory::read_value::<u32>(self.ptr + addresses::fields::entity::ENTITY_FLAGS) }
    }

    /// Name hash (`entity + 0x30`).
    pub fn name_hash(self) -> Option<u64> {
        unsafe { memory::read_value::<u64>(self.ptr + addresses::fields::entity::NAME_HASH) }
    }

    /// Frame node (`entity + 0x78`).
    pub fn frame_node(self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + addresses::fields::entity::FRAME_NODE) }
    }

    /// Owner (`entity + 0x80`).
    ///
    /// Для human:
    /// - NULL = пешком
    /// - vehicle ptr = в машине
    pub fn owner(self) -> Option<usize> {
        unsafe {
            let raw = memory::read_ptr_raw(self.ptr + addresses::fields::entity::OWNER)?;
            if raw == 0 { None } else { Some(raw) }
        }
    }

    /// Позиция через прямое чтение frame node.
    pub fn position(self) -> Option<Vec3> {
        let frame =
            unsafe { memory::read_ptr_raw(self.ptr + addresses::fields::entity::FRAME_NODE)? };
        if frame == 0 || !memory::is_valid_ptr(frame) {
            return None;
        }

        let x =
            unsafe { memory::read_value::<f32>(frame + addresses::fields::entity_frame::POS_X)? };
        let y =
            unsafe { memory::read_value::<f32>(frame + addresses::fields::entity_frame::POS_Y)? };
        let z =
            unsafe { memory::read_value::<f32>(frame + addresses::fields::entity_frame::POS_Z)? };

        if x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Vec3 { x, y, z })
        } else {
            None
        }
    }

    /// Lua-facing EntityType, если для данного factory type есть понятный mapping.
    pub fn lua_entity_type(self) -> Option<EntityType> {
        let ft = self.factory_type()?;
        EntityType::from_factory_type(ft as u8)
    }

    /// Является ли сущность human/player.
    pub fn is_humanoid(self) -> bool {
        matches!(self.factory_type(), Some(ft) if ft.is_humanoid())
    }

    /// Является ли сущность транспортом.
    pub fn is_vehicle(self) -> bool {
        matches!(self.factory_type(), Some(ft) if ft.is_vehicle())
    }

    /// Health (только для human/player).
    pub fn health(self) -> Option<f32> {
        if !self.is_humanoid() {
            return None;
        }
        unsafe { memory::read_value::<f32>(self.ptr + addresses::fields::player::CURRENT_HEALTH) }
    }

    /// Alive flag (только для human/player).
    pub fn is_alive(self) -> Option<bool> {
        if !self.is_humanoid() {
            return None;
        }
        unsafe {
            memory::read_value::<u8>(self.ptr + addresses::fields::player::IS_DEAD).map(|v| v == 0)
        }
    }

    /// Для humanoid: находится ли в транспорте.
    pub fn is_in_vehicle(self) -> Option<bool> {
        if !self.is_humanoid() {
            return None;
        }
        unsafe {
            memory::read_ptr_raw(self.ptr + addresses::fields::player::OWNER)
                .map(|owner| owner != 0)
        }
    }

    /// Короткая строка для логов.
    pub fn debug_label(self) -> String {
        let ft = self.factory_type_byte().unwrap_or(0);
        let tid = self.table_id().unwrap_or(0);
        let iid = self.instance_id().unwrap_or(0);
        let vt = self.vtable().unwrap_or(0);

        format!(
            "EntityRef(ptr=0x{:X}, type=0x{:02X}, table_id=0x{:08X}, instance={}, vtable=0x{:X})",
            self.ptr, ft, tid, iid, vt
        )
    }

    /// Проверка что vtable находится в `.rdata` модуля игры.
    pub fn has_probably_valid_vtable(self) -> bool {
        let Some(vtable) = self.vtable() else {
            return false;
        };
        let rva = vtable.wrapping_sub(base());
        (0x1800000..=0x1C50000).contains(&rva)
    }
}
