//! ScriptEntity family structures.
//!
//! Источники:
//! - `M2DE_CScriptEntity_Construct` (`0x14039BDE0`)
//!   - базовый top-level constructor
//!   - final type = `0x62`
//!   - alloc size = `0x90`
//!   - base vtable = `0x14186E170`
//!
//! - `M2DE_CScriptEntity_InitInPlace` (`0x14039BE40`) [strong]
//!   - инициализация уже выделенного блока памяти как ScriptEntity-like object
//!   - используется перед заменой vtable у child / derived paths
//!
//! - `M2DE_CDamageZone_Construct` (`0x140C0E8A0`)
//!   - outer entity type = `0x1E`
//!   - primary vtable = `0x1418D0A78`
//!   - выделяет child object размером `0xA0`
//!   - child vtable = `0x1418D05D8`
//!
//! - `M2DE_CTelephoneReg_Construct` (`0x140C0E9F0`)
//!   - outer entity type = `0x20`
//!   - primary vtable = `0x1418D0D80`
//!   - embedded child начинается по `outer + 0x100`
//!   - child vtable = `0x1418D0C58`
//!
//! - phonecalls child path:
//!   - `0x140E1A570`
//!   - alloc `0x90`, base init, child vtable `0x1418EAFF8`
//!
//! - direct police-script child path (Sub5):
//!   - `M2DE_CScriptEntitySub5_CreateInstance` (`0x140EBFD00`)
//!   - `M2DE_CScriptEntitySub5_Construct` (`0x1400B3B50`)
//!   - `M2DE_VT_CScriptEntity_Sub5` (`0x14184B230`)
//!   - Lua observed:
//!       - `AddPoliceman(self, guid_a, guid_b, number, vec3)`
//!       - `RemovePoliceman(self, guid)`
//!
//! Runtime:
//! - `factory_type = 0x62` -> 200 entities in FreeRide
//!
//! ВАЖНО:
//! не все связанные vtable принадлежат standalone top-level entity.
//! Часть используются как child script objects внутри outer world entities.

use std::ffi::c_void;

use crate::macros::assert_field_offsets;

// =============================================================================
//  C_ScriptEntity — базовый top-level ScriptEntity (size = 0x90)
// =============================================================================

/// Базовый `C_ScriptEntity`.
///
/// Confirmed:
/// - top-level constructor: `M2DE_CScriptEntity_Construct`
/// - final type = `0x62`
/// - alloc size = `0x90`
/// - base vtable = `0x14186E170`
///
/// Base ctor делает:
/// - `BaseEntity_Construct`
/// - `type = 0x62`
/// - `entity_flags |= 0x40`
/// - `+0x78 <- edx`
/// - `+0x7C <- r8d`
/// - `+0x80 <- -1`
/// - `+0x88 <- 0`
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptEntity {
    /// Primary vtable.
    pub vtable: *const c_void, // +0x00

    /// Базовый `C_Entity` head до script-specific tail.
    ///
    /// Здесь уже лежат:
    /// - state flags
    /// - packed table_id
    /// - entity_flags
    /// - name_hash
    ///
    /// Но точная разбивка всего диапазона `+0x08..+0x77`
    /// пока не завершена, поэтому храним raw-блок.
    pub _entity_head_08: [u8; 0x70], // +0x08..+0x77

    /// Script slot / entry id.
    ///
    /// Base ctor пишет сюда `EDX`.
    /// В direct police child path используется для доступа к:
    /// `scripts[this+0x78]`.
    pub script_entry_id: u32, // +0x78

    /// Script context index / selector.
    ///
    /// Base ctor пишет сюда `R8D`.
    /// В direct police child path низкий байт читается как:
    /// `movzx ecx, byte ptr [this+7Ch]`.
    pub script_context_index: u32, // +0x7C

    /// Дополнительный code/state field.
    ///
    /// Base ctor инициализирует `-1`.
    /// В add/init path участвует как дополнительный аргумент/состояние.
    pub aux_code_or_state: i32, // +0x80
    pub _pad_84: u32, // +0x84

    /// Provider/list-like pointer.
    ///
    /// Base ctor зануляет.
    /// В direct police child path используется как pointer на provider/list,
    /// из которого перебираются script-related записи.
    pub script_provider_or_list: *mut c_void, // +0x88
}

assert_field_offsets!(CScriptEntity {
    vtable                == 0x00,
    script_entry_id       == 0x78,
    script_context_index  == 0x7C,
    aux_code_or_state     == 0x80,
    script_provider_or_list == 0x88,
});

impl CScriptEntity {
    /// Factory type byte из packed table_id (`entity + 0x24`).
    pub fn factory_type(&self) -> u8 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x24) as *const u8)
        }
    }

    /// Packed table_id.
    pub fn table_id(&self) -> u32 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x24) as *const u32)
        }
    }

    /// Entity flags (`entity + 0x28`).
    pub fn entity_flags(&self) -> u32 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x28) as *const u32)
        }
    }
}

// =============================================================================
//  C_PoliceScriptChild — direct Sub5 police-script child path (size = 0x90)
// =============================================================================

/// Direct police-script child path (рабочее reverse-имя).
///
/// Это тот же `0x90`-байтный layout, что и `CScriptEntity`,
/// но с уже более узкой и подтверждённой семантикой:
///
/// - `InitAndAddPoliceman`
/// - `CallRemovePolicemanByGuid`
///
/// Lua-side observed behavior:
/// - `AddPoliceman(self, guid_a, guid_b, number, vec3)`
/// - `RemovePoliceman(self, guid)`
///
/// ВАЖНО:
/// formal engine class name пока не подтверждён на 100%.
/// Это именно рабочее reverse-имя.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CPoliceScriptChild {
    /// Primary vtable.
    pub vtable: *const c_void, // +0x00

    /// Базовый entity head.
    pub _entity_head_08: [u8; 0x70], // +0x08..+0x77

    /// Индекс/entry id внутри `scripts[...]`.
    ///
    /// Remove-path:
    /// - `scripts[this+0x78]["RemovePoliceman"]`
    ///
    /// Add-path:
    /// - `scripts[this+0x78]["AddPoliceman"]`
    pub script_entry_id: u32, // +0x78

    /// Script-context selector / script machine selector.
    pub script_context_index: u32, // +0x7C

    /// Дополнительный code/state-like field.
    pub aux_code_or_state: i32, // +0x80
    pub _pad_84: u32, // +0x84

    /// Pointer на provider/list/registry object.
    ///
    /// В heavy add/init path используется как source контейнера script entries.
    pub script_provider_or_list: *mut c_void, // +0x88
}

assert_field_offsets!(CPoliceScriptChild {
    vtable                == 0x00,
    script_entry_id       == 0x78,
    script_context_index  == 0x7C,
    aux_code_or_state     == 0x80,
    script_provider_or_list == 0x88,
});

impl CPoliceScriptChild {
    pub fn factory_type(&self) -> u8 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x24) as *const u8)
        }
    }

    pub fn table_id(&self) -> u32 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x24) as *const u32)
        }
    }

    pub fn entity_flags(&self) -> u32 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x28) as *const u32)
        }
    }
}

// =============================================================================
//  C_ScriptEntityChildEx — child object pattern (size = 0xA0)
// =============================================================================

/// Расширенный child script-object паттерн размером `0xA0`.
///
/// Этот layout подтверждён как минимум для:
/// - child object внутри `C_DamageZone`
/// - embedded child внутри `C_TelephoneReg`
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptEntityChildEx {
    pub base: CScriptEntity, // +0x00 .. +0x8F

    /// Дополнительное child-поле.
    pub field_90: *mut c_void, // +0x90

    /// Дополнительное child-поле / helper ptr slot.
    pub field_98: *mut c_void, // +0x98
}

assert_field_offsets!(CScriptEntityChildEx {
    base     == 0x00,
    field_90 == 0x90,
    field_98 == 0x98,
});

// =============================================================================
//  Documentation block
// =============================================================================

/// `C_ScriptEntity` family summary:
///
/// ```text
/// Base:
///   C_ScriptEntity
///     ctor  = 0x14039BDE0
///     size  = 0x90
///     type  = 0x62
///     vtbl  = 0x14186E170
///
/// Child / related vtable variants:
///   DamageZone child:
///     vtbl = 0x1418D05D8
///     alloc size = 0xA0
///     created in C_DamageZone ctor
///
///   TelephoneReg child:
///     vtbl = 0x1418D0C58
///     embedded at outer + 0x100
///     used in C_TelephoneReg ctor
///
///   PhoneCalls child:
///     vtbl = 0x1418EAFF8
///     path loads "/scripts/common/phonecalls.lua"
///
///   Direct police-script child path:
///     vtbl = 0x14184B230
///     direct alloc size = 0x90
///     observed Lua behavior:
///       AddPoliceman(self, guid_a, guid_b, number, vec3)
///       RemovePoliceman(self, guid)
/// ```
pub const _SCRIPT_ENTITY_FAMILY_DOC: () = ();
