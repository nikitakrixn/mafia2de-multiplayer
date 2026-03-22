//! Структуры игрока и NPC (CHuman / CHumanNPC / CPlayer).
//!
//! Все поля восстановлены из IDA Pro + runtime проверок.

use super::{CEntity, Inventory};
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

/// CHuman — базовый класс для всех гуманоидов (NPC и Player).
///
/// Размер: до +0x260 (где заканчиваются общие поля).
/// Конструктор: `M2DE_CHuman_BaseConstructor` (0x140D730B0).
/// Vtable: abstract (с _purecall).
///
/// Наследуется:
/// - CHumanNPC (0x0E) — NPC, размер ~0x338
/// - CPlayer (0x10) — Player, размер 0x520+
///
/// Подтверждённые поля:
/// - Health/damage (+0x148-0x163)
/// - Components (+0xA8-0x120)
/// - Smart ptr slots (+0x1C0-0x23F)
/// - Self-ref (+0x190)
/// - Physics provider (+0x258)
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHuman {
    /// База C_Entity (+0x00..+0x77)
    pub base: CEntity, // +0x000

    /// C_Actor overlay
    pub frame_node: *mut c_void, // +0x078
    pub owner: *mut c_void, // +0x080

    /// Для Human/Player обычно 0, но оставляем как raw actor extension.
    pub actor_field_88: usize, // +0x088
    pub actor_field_90: usize, // +0x090
    pub actor_field_98: usize, // +0x098

    pub entity_subtype: u32, // +0x0A0
    pub _pad_0A4: u32, // +0x0A4

    /// Human components
    pub ai_params: *mut c_void, // +0x0A8
    pub external_component_b0: *mut c_void, // +0x0B0
    pub component_b8: *mut c_void, // +0x0B8
    pub ai_nav: *mut c_void, // +0x0C0
    pub component_c8: *mut c_void, // +0x0C8
    pub transform_sync: *mut c_void, // +0x0D0
    pub optional_component: *mut c_void, // +0x0D8
    pub component_e0: *mut c_void, // +0x0E0
    pub inventory: *mut Inventory, // +0x0E8
    pub property_accessor: *mut c_void, // +0x0F0
    pub behavior: *mut c_void, // +0x0F8
    pub component_100: *mut c_void, // +0x100
    pub weapon_state: *mut c_void, // +0x108
    pub component_110: *mut c_void, // +0x110
    pub component_118: *mut c_void, // +0x118
    pub component_120: *mut c_void, // +0x120
    pub _unk_128: [u8; 0x20], // +0x128..+0x147

    /// Health / damage
    pub current_health: f32, // +0x148
    pub field_14c: f32, // +0x14C
    pub nonplayer_damage_mult: f32, // +0x150
    pub nonplayer_damage_dist: f32, // +0x154
    pub _pad_158: [u8; 0x08], // +0x158..+0x15F

    pub invulnerability: u8, // +0x160
    pub is_dead: u8, // +0x161
    pub demigod: u8, // +0x162
    pub unknown_flag_163: u8, // +0x163
    pub _pad_164: [u8; 0x1C], // +0x164..+0x17F

    pub body_damage_multipliers: *mut f32, // +0x180
    pub body_zone_count: u32, // +0x188
    pub _unk_18c: u32, // +0x18C

    pub self_ref: *mut CHuman, // +0x190

    pub _unk_198: u32, // +0x198
    pub _unk_19c: u32, // +0x19C

    pub _pad_1A0: [u8; 0x20], // +0x1A0..+0x1BF

    /// 8 smart ptr slots по 16 байт — пока raw.
    pub smart_ptr_slots: [[u8; 16]; 8], // +0x1C0..+0x23F

    pub _pad_240: [u8; 0x18], // +0x240..+0x257

    pub physics_provider: *mut c_void, // +0x258
}

assert_field_offsets!(CHuman {
    base                       == 0x000,
    frame_node                 == 0x078,
    owner                      == 0x080,
    actor_field_88             == 0x088,
    entity_subtype             == 0x0A0,
    ai_params                  == 0x0A8,
    external_component_b0      == 0x0B0,
    component_b8               == 0x0B8,
    ai_nav                     == 0x0C0,
    component_c8               == 0x0C8,
    transform_sync             == 0x0D0,
    optional_component         == 0x0D8,
    component_e0               == 0x0E0,
    inventory                  == 0x0E8,
    property_accessor          == 0x0F0,
    behavior                   == 0x0F8,
    component_100              == 0x100,
    weapon_state               == 0x108,
    component_110              == 0x110,
    component_118              == 0x118,
    component_120              == 0x120,
    current_health             == 0x148,
    field_14c                  == 0x14C,
    nonplayer_damage_mult      == 0x150,
    nonplayer_damage_dist      == 0x154,
    invulnerability            == 0x160,
    is_dead                    == 0x161,
    demigod                    == 0x162,
    unknown_flag_163           == 0x163,
    body_damage_multipliers    == 0x180,
    body_zone_count            == 0x188,
    self_ref                   == 0x190,
    physics_provider           == 0x258,
});

impl CHuman {
    /// factory type byte из packed table_id.
    pub fn factory_type(&self) -> u8 {
        self.base.factory_type()
    }

    /// packed table_id.
    pub fn table_id(&self) -> u32 {
        self.base.table_id
    }

    /// entity flags.
    pub fn entity_flags(&self) -> u32 {
        self.base.entity_flags
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

    pub fn has_valid_self_ref(&self) -> bool {
        let self_ptr = self as *const Self as *mut CHuman;
        self.self_ref == self_ptr
    }
}
