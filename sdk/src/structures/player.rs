//! Структура игрока (C_Human / C_Player).
//!
//! Все поля восстановлены из IDA Pro + runtime проверок.
//! Процент уверенности указан в комментариях к каждому полю.

use super::Inventory;
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

/// C_Human / C_Player.
///
/// Базовый head `C_Entity` до `+0x78` пока не восстановлен полностью,
/// поэтому он представлен raw-блоком.
///
/// Подтверждённые полезные поля:
/// - `+0x78` frame node
/// - `+0x80` owner
/// - `+0xA8` ai params
/// - `+0xC0` ai nav
/// - `+0xD0` transform sync
/// - `+0xE8` inventory
/// - `+0xF0` property accessor
/// - `+0xF8` behavior
/// - `+0x108` weapon state
/// - `+0x148` current health
/// - `+0x160` invulnerability
/// - `+0x161` is_dead
/// - `+0x162` demigod
/// - `+0x180` body damage multipliers
/// - `+0x190` self_ref
/// - `+0x258` physics provider
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHuman {
    /// Primary vtable.
    pub vtable: *const c_void, // +0x000

    /// Неполностью доревершенный entity head.
    pub _entity_head_08: [u8; 0x70], // +0x008..+0x077

    /// Frame / transform node.
    pub frame_node: *mut c_void, // +0x078

    /// Owner entity. NULL = on foot, vehicle ptr = in car.
    pub owner: *mut c_void, // +0x080

    /// Unknown actor padding.
    pub _actor_pad_088: [u8; 0x18], // +0x088..+0x09F

    /// Actor subtype.
    pub entity_subtype: u32, // +0x0A0
    pub _pad_0A4: u32, // +0x0A4

    /// AI params / config block.
    pub ai_params: *mut c_void, // +0x0A8

    /// Unknown region between ai_params and ai_nav.
    pub _unk_0B0: [u8; 0x10], // +0x0B0..+0x0BF

    /// AI navigation component.
    pub ai_nav: *mut c_void, // +0x0C0
    pub component_c8: *mut c_void,       // +0x0C8
    pub transform_sync: *mut c_void,     // +0x0D0
    pub optional_component: *mut c_void, // +0x0D8
    pub component_e0: *mut c_void,       // +0x0E0

    /// Inventory pointer.
    pub inventory: *mut Inventory, // +0x0E8

    /// Property accessor / control-like component.
    pub property_accessor: *mut c_void, // +0x0F0

    /// Behavior component.
    pub behavior: *mut c_void, // +0x0F8

    pub component_100: *mut c_void, // +0x100

    /// Weapon state component.
    pub weapon_state: *mut c_void, // +0x108

    /// Unknown region before health.
    pub _unk_110: [u8; 0x38], // +0x110..+0x147

    /// Current health.
    pub current_health: f32, // +0x148

    /// Для NPC — обычно healthmax.
    /// Для player часть семантики может отличаться.
    pub npc_healthmax_or_type_mult: f32, // +0x14C

    /// Damage multiplier from non-player sources.
    pub nonplayer_damage_mult: f32, // +0x150

    /// Distance threshold for damage falloff.
    pub nonplayer_damage_dist: f32, // +0x154

    pub _pad_158: [u8; 0x08], // +0x158..+0x15F

    /// Invulnerability flag.
    pub invulnerability: u8, // +0x160

    /// Is dead flag.
    pub is_dead: u8, // +0x161

    /// Demigod flag.
    pub demigod: u8, // +0x162

    pub _pad_163: [u8; 0x1D], // +0x163..+0x17F

    /// Body damage multipliers pointer.
    pub body_damage_multipliers: *mut f32, // +0x180

    pub _pad_188: [u8; 0x08], // +0x188..+0x18F

    /// Self pointer.
    pub self_ref: *mut CHuman, // +0x190

    /// NPC-specific indices / table refs.
    pub entity_table_index: i32, // +0x198
    pub entity_table_slot: i32, // +0x19C

    pub _pad_1A0: [u8; 0x20], // +0x1A0..+0x1BF

    /// 8 smart pointer slots.
    pub smart_ptr_slots: [[u8; 16]; 8], // +0x1C0..+0x23F

    pub _pad_240: [u8; 0x18], // +0x240..+0x257

    /// Physics provider.
    pub physics_provider: *mut c_void, // +0x258
}

assert_field_offsets!(CHuman {
    vtable                     == 0x000,
    frame_node                 == 0x078,
    owner                      == 0x080,
    entity_subtype             == 0x0A0,
    ai_params                  == 0x0A8,
    ai_nav                     == 0x0C0,
    transform_sync             == 0x0D0,
    inventory                  == 0x0E8,
    property_accessor          == 0x0F0,
    behavior                   == 0x0F8,
    weapon_state               == 0x108,
    current_health             == 0x148,
    npc_healthmax_or_type_mult == 0x14C,
    nonplayer_damage_mult      == 0x150,
    nonplayer_damage_dist      == 0x154,
    invulnerability            == 0x160,
    is_dead                    == 0x161,
    demigod                    == 0x162,
    body_damage_multipliers    == 0x180,
    self_ref                   == 0x190,
    physics_provider           == 0x258,
});

impl CHuman {
    /// factory type byte из packed table_id по offset +0x24.
    pub fn factory_type(&self) -> u8 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x24) as *const u8)
        }
    }

    /// packed table_id по offset +0x24.
    pub fn table_id(&self) -> u32 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x24) as *const u32)
        }
    }

    /// entity flags по offset +0x28.
    pub fn entity_flags(&self) -> u32 {
        unsafe {
            let base = self as *const Self as *const u8;
            *(base.add(0x28) as *const u32)
        }
    }

    pub fn health(&self) -> f32 {
        self.current_health
    }

    pub fn is_alive(&self) -> bool {
        self.is_dead == 0
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invulnerability != 0
    }

    pub fn is_demigod(&self) -> bool {
        self.demigod != 0
    }

    pub fn is_in_vehicle(&self) -> bool {
        !self.owner.is_null()
    }

    pub fn is_player(&self) -> bool {
        self.factory_type() == 0x10
    }

    pub fn is_valid_ptr(&self) -> bool {
        let self_ptr = self as *const Self as *mut CHuman;
        self.self_ref == self_ptr
    }
}
