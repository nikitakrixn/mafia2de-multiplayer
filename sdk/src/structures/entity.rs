//! Структуры системы сущностей — CEntity, CActor и инфраструктура.

use std::ffi::c_void;
use crate::macros::assert_field_offsets;

// =============================================================================
//  CEntity — корень иерархии (0x78 байт)
// =============================================================================

/// Базовый класс для всех сущностей движка.
///
/// Размер: **0x78 байт**. Все остальные типы наследуются от него.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntity {
    /// Указатель на vtable. Конкретный тип зависит от подкласса.
    /// Для Player/Human кастить в `*const CPlayerVTable`.
    pub vtable: *const c_void, // +0x00

    pub ext_ptr_1: usize, // +0x08
    pub ext_ptr_2: usize, // +0x10
    pub ext_ptr_3: usize, // +0x18

    pub state_flags: u8, // +0x20
    pub _gap_21: [u8; 3], // +0x21..+0x23

    /// Упакованный идентификатор: `(instance_id << 8) | factory_type`.
    pub table_id: u32, // +0x24

    /// Флаги сущности (bit 5 = активирована, etc.).
    pub entity_flags: u32, // +0x28
    pub _gap_2c: u32, // +0x2C

    /// FNV-1 64-bit хеш имени. 0 для безымянных.
    pub name_hash: u64, // +0x30

    pub parent_ref: usize, // +0x38
    pub tree_1_root: usize, // +0x40
    pub tree_1_count: usize, // +0x48
    pub tree_2_root: usize, // +0x50
    pub _zero_58: usize, // +0x58

    pub pending_msg_begin: usize, // +0x60
    pub pending_msg_end: usize,   // +0x68
    pub pending_msg_cap: usize,   // +0x70
}

assert_field_offsets!(CEntity {
    vtable            == 0x00,
    state_flags       == 0x20,
    table_id          == 0x24,
    entity_flags      == 0x28,
    name_hash         == 0x30,
    parent_ref        == 0x38,
    tree_1_root       == 0x40,
    tree_2_root       == 0x50,
    pending_msg_begin == 0x60,
    pending_msg_end   == 0x68,
    pending_msg_cap   == 0x70,
});

impl CEntity {
    pub fn factory_type(&self) -> u8 {
        (self.table_id & 0xFF) as u8
    }

    pub fn instance_index(&self) -> u32 {
        self.table_id >> 8
    }

    pub fn is_activated(&self) -> bool {
        (self.entity_flags & 0x20) != 0
    }

    pub fn has_streaming_flag(&self) -> bool {
        (self.entity_flags & 0x60000) != 0
    }
}

// =============================================================================
//  CActor — расширяет CEntity трансформацией и owner'ом (0xA8 байт)
// =============================================================================

/// Слой Actor — добавляет к CEntity frame node, owner и subtype.
///
/// Позиция читается из frame_node:
/// ```text
/// frame + 0x64 = X, frame + 0x74 = Y, frame + 0x84 = Z
/// ```
#[repr(C)]
#[allow(non_snake_case)]
pub struct CActor {
    /// Базовая сущность.
    pub base: CEntity, // +0x00

    /// Узел трансформации в мировом пространстве.
    pub frame_node: *mut c_void, // +0x78

    /// Владелец/контейнер. NULL = пешком, ненулевой = в транспорте.
    pub owner: *mut c_void, // +0x80

    /// Расширенные компоненты. У гуманоидов обычно 0, у транспорта ≠ 0.
    pub component_88: usize, // +0x88
    pub component_90: usize, // +0x90
    pub component_98: usize, // +0x98

    /// Подтип сущности: Player=6, CarVehicle=3.
    pub entity_subtype: u32, // +0xA0
    pub _pad_a4: u32,        // +0xA4
}

assert_field_offsets!(CActor {
    base           == 0x00,
    frame_node     == 0x78,
    owner          == 0x80,
    component_88   == 0x88,
    component_90   == 0x90,
    component_98   == 0x98,
    entity_subtype == 0xA0,
});

const _: () = assert!(std::mem::size_of::<CActor>() == 0xA8);

// =============================================================================
//  C_EntityGuid — уникальный идентификатор сущности
// =============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CEntityGuid {
    pub guid: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntityDBRecord {
    pub _unk_00: [u8; 0x24],
    pub table_id: u32, // +0x24
    pub flags: u32,    // +0x28
    pub _unk_2c: u32,  // +0x2C
    pub name_hash: u64, // +0x30
}

impl CEntityDBRecord {
    pub fn factory_type(&self) -> u8 { (self.table_id & 0xFF) as u8 }
    pub fn instance_index(&self) -> u32 { self.table_id >> 8 }
    pub fn has_script_wrapper(&self) -> bool { (self.flags & 0x20) != 0 }
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptWrapper {
    pub vtable: *const c_void,
    pub refcount: i32,
    pub _pad_0c: i32,
    pub native_entity: *mut c_void, // +0x10
    pub observer: *mut c_void,      // +0x18
}

#[repr(C)]
pub struct CScriptWrapperManager {
    pub vtable: *const c_void,
    pub hash_cache_begin: *mut u8,
    pub hash_cache_end: *mut u8,
    pub hash_cache_sentinel: *mut u8,
    pub _unk_20: *mut c_void,
    pub id_cache_begin: *mut u8,
    pub id_cache_end: *mut u8,
    pub id_cache_capacity: *mut u8,
}

#[repr(C)]
pub struct CWrapperFactory {
    pub vtable: *const c_void,
    pub type_id_ptr: *const u32,
    pub create_fn: *const c_void,
}

#[repr(C)]
pub struct CServiceIdentity {
    pub vtable: *const c_void,
    pub name_hash: u32,
    pub module_id: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CTypeDescriptor {
    pub next: *mut CTypeDescriptor,
    pub type_id: u32,
    pub _pad_0c: u32,
    pub name_hash: u64,
    pub create_fn: *const c_void,
    pub parse_fn: *const c_void,
    pub aligned_size: u32,
    pub _pad_2c: u32,
}