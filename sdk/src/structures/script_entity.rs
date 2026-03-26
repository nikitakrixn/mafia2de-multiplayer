//! ScriptEntity family structures.
//!
//! `CScriptEntity` теперь встраивает `CEntity` вместо raw blob.
//! `CPoliceScriptChild` удалён — идентичный layout, используй `CScriptEntity`.

use std::ffi::c_void;
use crate::macros::assert_field_offsets;
use super::entity::CEntity;

// =============================================================================
//  C_ScriptEntity (size = 0x90)
// =============================================================================

/// Базовый `C_ScriptEntity`.
///
/// - factory type = `0x62`
/// - alloc size = `0x90`
/// - base vtable = `0x14186E170`
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptEntity {
    /// Полная базовая сущность.
    pub base: CEntity, // +0x00..+0x77

    /// Script slot / entry id.
    pub script_entry_id: u32, // +0x78

    /// Script context index / selector.
    pub script_context_index: u32, // +0x7C

    /// Дополнительный code/state field. Init: `-1`.
    pub aux_code_or_state: i32, // +0x80
    pub _pad_84: u32,           // +0x84

    /// Provider/list pointer. Init: `NULL`.
    pub script_provider_or_list: *mut c_void, // +0x88
}

assert_field_offsets!(CScriptEntity {
    base                    == 0x00,
    script_entry_id         == 0x78,
    script_context_index    == 0x7C,
    aux_code_or_state       == 0x80,
    script_provider_or_list == 0x88,
});

const _: () = {
    assert!(std::mem::size_of::<CScriptEntity>() == 0x90);
    assert!(std::mem::offset_of!(CScriptEntity, base.table_id) == 0x24);
    assert!(std::mem::offset_of!(CScriptEntity, base.entity_flags) == 0x28);
};

impl CScriptEntity {
    pub fn factory_type(&self) -> u8 { self.base.factory_type() }
    pub fn table_id(&self) -> u32 { self.base.table_id }
    pub fn entity_flags(&self) -> u32 { self.base.entity_flags }
}

// =============================================================================
//  C_ScriptEntityChildEx (size = 0xA0)
// =============================================================================

/// Расширенный child script-object.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptEntityChildEx {
    pub base: CScriptEntity,       // +0x00..+0x8F
    pub field_90: *mut c_void,     // +0x90
    pub field_98: *mut c_void,     // +0x98
}

assert_field_offsets!(CScriptEntityChildEx {
    base     == 0x00,
    field_90 == 0x90,
    field_98 == 0x98,
});