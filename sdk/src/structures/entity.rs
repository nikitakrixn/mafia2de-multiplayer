//! Entity system structures — infrastructure + base classes.
//!
//! Sources:
//! - M2DE_EntityManager_FindByName (inline FNV-1 + binary search)
//! - M2DE_EntityManager_CreateScriptWrapper (factory dispatch)
//! - M2DE_ScriptWrapperMgr_GetOrCreateWrapper (cache layout)
//! - M2DE_CHuman_BaseConstructor (component allocation, health init)
//! - M2DE_CHumanNPC_Constructor (self_ref, entity_table)
//! - M2DE_CPlayerEntity_Constructor (type=0x10, player vtable)
//! - M2DE_Register_CleanEntity_TypeDescriptor (TypeRegistry system)
//! - Runtime: Joe/Henry -> entity_type=0x0E, health=1000.0

use std::ffi::c_void;

use crate::macros::assert_field_offsets;

/// C_Entity — корень иерархии сущностей.
///
/// ВАЖНО: head до `+0x78` пока ещё восстановлен не полностью.
/// Поэтому здесь используется консервативный layout:
/// - подтверждённые поля выделены явно
/// - остальное оставлено raw/padding
///
/// Подтверждено:
/// - `+0x20` state_flags
/// - `+0x24` packed table_id
/// - `+0x28` entity_flags
/// - `+0x30` name_hash
/// - actor fields начинаются с `+0x78`
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntity {
    /// Primary vtable.
    pub vtable: *const c_void, // +0x00

    /// Неполностью доревершенный head C_Entity.
    pub _unknown_08: [u8; 0x18], // +0x08..+0x1F

    /// State/alive flags byte.
    pub state_flags: u8, // +0x20
    pub _pad_21: [u8; 3], // +0x21

    /// Packed table_id: `(instance_id << 8) | factory_type`.
    pub table_id: u32, // +0x24

    /// Entity flags.
    pub entity_flags: u32, // +0x28
    pub _pad_2C: u32, // +0x2C

    /// FNV-1 64-bit name hash.
    pub name_hash: u64, // +0x30

    /// Raw entity head tail.
    ///
    /// Сюда попадает область до actor layer (`+0x78`).
    pub _unknown_38: [u8; 0x40], // +0x38..+0x77
}

assert_field_offsets!(CEntity {
    vtable       == 0x00,
    state_flags  == 0x20,
    table_id     == 0x24,
    entity_flags == 0x28,
    name_hash    == 0x30,
});

impl CEntity {
    /// Native factory type byte (low byte of table_id).
    pub fn factory_type(&self) -> u8 {
        (self.table_id & 0xFF) as u8
    }

    /// Upper 24-bit instance id.
    pub fn instance_index(&self) -> u32 {
        self.table_id >> 8
    }
}

// =============================================================================
//  C_Actor — extends C_Entity with transform and owner (44 bytes)
// =============================================================================

/// C_Actor layer — adds frame node, owner, and sub-type.
///
/// Initialized by M2DE_ActorEntity_Construct (0x14039A7E0).
/// Vtable: off_14186D050 (M2DE_vtbl_CActor).
/// Zeroes +0x78..+0xA0, sets alive flags at +0x24.
///
/// Fields start at offset +0x78 from entity base.
/// Use with CEntity: `entity_ptr + sizeof(CEntity)` for actor fields.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CActorFields {
    /// Frame/transform node — world position.
    /// Position: frame+0x64 (X), frame+0x74 (Y), frame+0x84 (Z)
    pub frame_node: *mut c_void, // +0x78

    /// Owner entity. NULL=on foot, vehicle*=in car.
    pub owner: *mut c_void, // +0x80

    pub _unknown_88: u64, // +0x88  zeroed
    pub _unknown_90: u64, // +0x90  zeroed
    pub _unknown_98: u64, // +0x98  zeroed

    /// Entity sub-type (set after construction).
    pub entity_subtype: u32, // +0xA0
    pub _pad_A4: u32, // +0xA4
}

// =============================================================================
//  Entity GUID
// =============================================================================

/// Entity GUID — уникальный идентификатор сущности.
///
/// Lua: `C_EntityGuid`. Format: `"C_EntityGuid: %u"`.
/// Подтверждено из `M2DE_LuaW_WrappersList_GetEntityByGUID`:
/// ```c
/// M2DE_FormatString("C_EntityGuid: %u", *ThisObject);
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CEntityGuid {
    pub guid: u32,
}

// =============================================================================
//  Entity Database
// =============================================================================

/// Entity database record — запись в глобальной БД (M2DE_g_EntityDatabase).
///
/// table_id — упакованный формат:
///   Bits [7:0]  = factory type byte (для wrapper factory dispatch)
///   Bits [31:8] = instance index
///
/// Подтверждено из:
/// - FindByName: `mov r9d, [rax+24h]` (table_id as dword)
/// - CreateScriptWrapper: `movzx ebx, byte ptr [rdx+24h]` (low byte = type)
/// - GetOrCreateWrapper: `mov rcx, [rax+30h]` (name_hash)
/// - Runtime scan: factory_type == entity_type at native+0x24 for ALL entities
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntityDBRecord {
    pub _unknown_00: [u8; 0x24],
    /// Packed: (instance_index << 8) | factory_type_byte.
    pub table_id: u32, // +0x24
    /// Bit 5 (0x20) = has_script_wrapper / spawnable.
    pub flags: u32, // +0x28
    pub _unknown_2C: u32, // +0x2C
    /// FNV-1 64-bit hash of entity name.
    pub name_hash: u64, // +0x30
}

impl CEntityDBRecord {
    pub fn factory_type(&self) -> u8 {
        (self.table_id & 0xFF) as u8
    }
    pub fn instance_index(&self) -> u32 {
        self.table_id >> 8
    }
    pub fn has_script_wrapper(&self) -> bool {
        (self.flags & 0x20) != 0
    }
}

// =============================================================================
//  Script Wrapper System
// =============================================================================

/// Script wrapper — Lua-accessible handle to native entity.
///
/// `wrapper+0x10` IS the native entity pointer (runtime confirmed).
/// For Joe/Henry: reads entity_type=0x0E, health correctly.
///
/// Created by M2DE_EntityManager_CreateScriptWrapper:
///   1. factory_type = db_record.table_id & 0xFF
///   2. factory = M2DE_g_WrapperFactoryMap[factory_type]
///   3. wrapper = factory->Create() via vtable[+0x10]
///   4. wrapper+0x10 = native entity ptr
///   5. wrapper+0x18 = observer (264 bytes from DB record)
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptWrapper {
    pub vtable: *const c_void, // +0x00
    pub refcount: i32,         // +0x08
    pub _pad_0C: i32,          // +0x0C
    /// Native entity pointer (C_Human*, C_Car*, etc.). Runtime confirmed.
    pub native_entity: *mut c_void, // +0x10
    /// Observer (8 bytes alloc, vtable off_141919A78, 264 bytes from DB).
    pub observer: *mut c_void, // +0x18
}

/// Script Wrapper Manager — dual sorted cache for O(log n) lookup.
///
/// Global: M2DE_g_ScriptWrapperManager (0x1431360F8, double indirection).
/// Binary search in both caches.
///
/// Hash cache entry (16 bytes): { u64 fnv1_hash, *mut CScriptWrapper }
/// ID cache entry (16 bytes): { u32 table_id, u32 pad, *mut CScriptWrapper }
#[repr(C)]
pub struct CScriptWrapperManager {
    pub vtable: *const c_void,        // +0x00
    pub hash_cache_begin: *mut u8,    // +0x08
    pub hash_cache_end: *mut u8,      // +0x10
    pub hash_cache_sentinel: *mut u8, // +0x18
    pub _unknown_0x20: *mut c_void,   // +0x20
    pub id_cache_begin: *mut u8,      // +0x28
    pub id_cache_end: *mut u8,        // +0x30
    pub id_cache_capacity: *mut u8,   // +0x38
}

/// Wrapper Factory — creates typed C_ScriptWrapper for entity.
///
/// 36 factories registered in M2DE_g_WrapperFactoryMap (RB-tree).
/// All share vtable off_141918858.
///
/// Create function allocates wrapper with type-specific vtable:
/// ```c
/// result = GlobalAlloc(32);  // or larger for some types
/// result[0] = WRAPPER_VTABLE; // unique per entity type
/// result[1] = 1;              // refcount
/// result[2] = 0;              // native_entity (filled later)
/// result[3] = 0;              // observer (filled later)
/// ```
#[repr(C)]
pub struct CWrapperFactory {
    pub vtable: *const c_void,    // +0x00
    pub type_id_ptr: *const u32,  // +0x08
    pub create_fn: *const c_void, // +0x10
}
// =============================================================================
//  Service Identity
// =============================================================================

/// Service Identity — module registration in Service Locator.
///
/// Used by 49 module types (E_ModuleId 0-48).
/// Hash: FNV-1 32-bit (seed=0x811C9DC5, prime=0x01000193).
///
/// Confirmed from C_ServiceIdentity::Init (0x1404444F0):
/// ```asm
/// mov [rdi+8], ebx    ; FNV-1 hash
/// mov [rdi+0Ch], esi   ; module_id
/// ```
#[repr(C)]
pub struct CServiceIdentity {
    pub vtable: *const c_void, // +0x00
    pub name_hash: u32,        // +0x08  FNV-1 32-bit
    pub module_id: u32,        // +0x0C  E_ModuleId
}

// =============================================================================
//  Type Registry (native entity creation from SDS)
// =============================================================================

/// Type Descriptor — registered in global linked list for entity creation.
///
/// 49 types registered via M2DE_TypeRegistry_RegisterDescriptor.
/// Global: M2DE_g_TypeRegistry (0x141CAE228).
///
/// IMPORTANT: Name hash uses FNV-1 64-bit with **seed=0** (not standard seed!):
/// ```c
/// for (i = 0; *name; ) {       // seed = 0!
///     i = byte ^ (0x100000001B3 * i);
/// }
/// ```
///
/// Example (C_CleanEntity):
///   typeId = computed from alignment, nameHash = fnv1_64_seed0("C_CleanEntity")
///
/// Confirmed from M2DE_Register_CleanEntity_TypeDescriptor (0x14006AE30).
#[repr(C)]
#[allow(non_snake_case)]
pub struct CTypeDescriptor {
    pub next: *mut CTypeDescriptor, // +0x00
    pub type_id: u32,               // +0x08
    pub _pad_0C: u32,               // +0x0C
    pub name_hash: u64,             // +0x10  FNV-1 64-bit seed=0
    pub create_fn: *const c_void,   // +0x18
    pub parse_fn: *const c_void,    // +0x20
    pub aligned_size: u32,          // +0x28
    pub _pad_2C: u32,               // +0x2C
}

// =============================================================================
//  Constructor Chain (documented, not struct)
// =============================================================================

/// Constructor chain for C_Human entities (confirmed from IDA):
///
/// ```text
/// 1. M2DE_BaseEntity_Construct
///    - Sets initial vtable
///    - Initializes C_Entity fields
///
/// 2. M2DE_ActorEntity_Construct (sub called from step 3)
///    - Sets C_Actor vtable = off_14186D050
///    - Zeros frame_node (+0x78), owner (+0x80), and nearby ptrs
///    - Sets alive flags at +0x24: bits 0+2 (value 5)
///
/// 3. M2DE_CHuman_BaseConstructor (0x140D730B0)
///    - Sets vtable = M2DE_vtbl_CActor_Abstract (with _purecall entries)
///    - Allocates 2648 bytes (0xA58) for ALL inline components
///    - Assigns component pointers to entity offsets +0xA8..+0x120
///    - Sets initial values:
///        +0x148 = 210.0f (health, NOT 200.0!)
///        +0x14C = 210.0f (npc_healthmax)
///        +0x150 = 1.0f   (nonplayer_damage_mult)
///        +0x154 = 5.0f   (nonplayer_damage_dist)
///        +0x160 = 0       (invulnerability + is_dead)
///        +0x162 = 0       (demigod)
///
/// 4. M2DE_CHumanNPC_Constructor (0x140D712E0)
///    - Sets vtable = 0x1418E5188 (NPC vtable)
///    - Sets type = 0x0E via M2DE_Entity_SetTypeID
///    - Sets entity_flags |= 0x40 at +0x28
///    - Initializes self_ref (+0x190) = this
///    - Initializes 8 smart ptr slots (+0x1C0..+0x238, IDs 1-7 and -1)
///    - Registers in global entity table
///
/// 5. M2DE_CPlayerEntity_Constructor (0x1400B9160)  [Player only]
///    - Overwrites vtable = 0x14184C060 (player vtable)
///    - Sets type = 0x10 via M2DE_Entity_SetTypeID
///    - Initializes player-specific fields from +0x338
///    - Total player entity size: ~0x530+ bytes
/// ```
///
/// Component allocation (2648 bytes, single block):
/// ```text
/// +0xA8  -> AI params block (78+ bytes with float arrays)
/// +0xB8  -> unknown component
/// +0xC0  -> AI navigation (vtable from sub_140D723E0)
/// +0xC8  -> component (vtable off_1418E3330)
/// +0xD0  -> TransformSync (vtable M2DE_vtbl_TransformSyncComponent)
/// +0xD8  -> NULL (optional, filled on demand)
/// +0xE0  -> component (vtable off_1418E3308)
/// +0xE8  -> Inventory (vtable M2DE_vtbl_HumanInventory)
/// +0xF0  -> PropertyAccessor (back-ref at comp+0x170 -> entity)
/// +0xF8  -> Behavior (sub_140D71AD0 result)
/// +0x100 -> component block
/// +0x108 -> weapon state (sub_1400B8FD0 result)
/// +0x110 -> component (sub_1400B8F90 result)
/// +0x118 -> component (vtable off_1418E33D0)
/// +0x120 -> component (vtable off_1418E3358)
/// ```
pub const _CONSTRUCTOR_CHAIN_DOC: () = ();
