//! Структуры гуманоидов — CHuman, CHumanNPC, CPlayer.
//!
//! Иерархия:
//! ```text
//! CEntity (0x78)
//!   └─ CActor (0xA8)
//!       └─ CHuman (+0xA8)
//!           ├─ CHumanNPC (~0x338) — тип 0x0E
//!           └─ CPlayer (0x520+)   — тип 0x10
//! ```

use super::entity::CActor;
use super::Inventory;
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

// =============================================================================
//  CHuman — базовый класс всех гуманоидов
// =============================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct CHuman {
    /// CEntity + CActor base.
    pub actor: CActor, // +0x000..+0x0A7

    // Компоненты (начинаются с +0xA8)

    pub ai_params: *mut c_void,              // +0x0A8
    pub external_component_b0: *mut c_void,  // +0x0B0
    pub component_b8: *mut c_void,           // +0x0B8
    pub ai_nav: *mut c_void,                 // +0x0C0
    pub component_c8: *mut c_void,           // +0x0C8
    pub transform_sync: *mut c_void,         // +0x0D0
    pub optional_component: *mut c_void,     // +0x0D8
    pub component_e0: *mut c_void,           // +0x0E0
    pub inventory: *mut Inventory,           // +0x0E8
    pub property_accessor: *mut c_void,      // +0x0F0
    pub behavior: *mut c_void,               // +0x0F8
    pub component_100: *mut c_void,          // +0x100
    pub weapon_state: *mut c_void,           // +0x108
    pub component_110: *mut c_void,          // +0x110
    pub component_118: *mut c_void,          // +0x118
    pub component_120: *mut c_void,          // +0x120

    pub _unk_128: [u8; 0x20], // +0x128..+0x147

    // Здоровье и урон

    pub current_health: f32,           // +0x148
    pub field_14c: f32,                // +0x14C
    pub nonplayer_damage_mult: f32,    // +0x150
    pub nonplayer_damage_dist: f32,    // +0x154
    pub damage_param_158: f32,         // +0x158
    pub damage_scale_factor: f32,      // +0x15C
    pub invulnerability: u8,           // +0x160
    pub is_dead: u8,                   // +0x161
    pub demigod: u8,                   // +0x162
    pub unknown_flag_163: u8,          // +0x163
    pub _pad_164: [u8; 4],            // +0x164..+0x167

    // Модель

    pub model_descriptor: *mut c_void, // +0x168
    pub saved_frame_ptr: *mut c_void,  // +0x170
    pub saved_streaming_flag: u8,      // +0x178
    pub _pad_179: [u8; 7],            // +0x179..+0x17F

    // Зоны повреждений

    pub body_damage_multipliers: *mut f32, // +0x180
    pub body_zone_count: u32,              // +0x188
    pub _unk_18c: u32,                     // +0x18C
    pub self_ref: *mut CHuman,             // +0x190
    pub _unk_198: u32,                     // +0x198
    pub _unk_19c: u32,                     // +0x19C
    pub _pad_1A0: [u8; 0x20],            // +0x1A0..+0x1BF

    pub smart_ptr_slots: [[u8; 16]; 8], // +0x1C0..+0x23F
    pub _pad_240: [u8; 0x18],          // +0x240..+0x257

    pub physics_provider: *mut c_void, // +0x258

    pub _pad_260: [u8; 0x18],            // +0x260..+0x277
    pub helper_object_278: *mut c_void,  // +0x278
    pub _pad_280: [u8; 0x14],            // +0x280..+0x293
    pub movement_speed_target: f32,      // +0x294
    pub movement_speed_current: f32,     // +0x298
    pub _pad_29C: [u8; 0x5C],           // +0x29C..+0x2F7
    pub model_overlay_active: u8,        // +0x2F8
    pub _pad_2F9: [u8; 7],              // +0x2F9..+0x2FF
    pub model_overlay_state: usize,      // +0x300
    pub frame_detach_flag: u8,           // +0x308
    pub _pad_309: [u8; 7],              // +0x309..+0x30F
    pub collision_body: *mut c_void,     // +0x310
}

assert_field_offsets!(CHuman {
    actor                      == 0x000,
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
    damage_param_158           == 0x158,
    damage_scale_factor        == 0x15C,
    invulnerability            == 0x160,
    is_dead                    == 0x161,
    demigod                    == 0x162,
    model_descriptor           == 0x168,
    saved_frame_ptr            == 0x170,
    body_damage_multipliers    == 0x180,
    body_zone_count            == 0x188,
    self_ref                   == 0x190,
    physics_provider           == 0x258,
    helper_object_278          == 0x278,
    movement_speed_target      == 0x294,
    movement_speed_current     == 0x298,
    model_overlay_active       == 0x2F8,
    model_overlay_state        == 0x300,
    frame_detach_flag          == 0x308,
    collision_body             == 0x310,
});

// Nested offset checks (Rust 1.82+)
const _: () = {
    assert!(std::mem::offset_of!(CHuman, actor.base.vtable) == 0x00);
    assert!(std::mem::offset_of!(CHuman, actor.base.table_id) == 0x24);
    assert!(std::mem::offset_of!(CHuman, actor.frame_node) == 0x78);
    assert!(std::mem::offset_of!(CHuman, actor.owner) == 0x80);
    assert!(std::mem::offset_of!(CHuman, actor.entity_subtype) == 0xA0);
};

impl CHuman {
    // Делегаты к CEntity (через actor.base)

    pub fn factory_type(&self) -> u8 {
        self.actor.base.factory_type()
    }

    pub fn table_id(&self) -> u32 {
        self.actor.base.table_id
    }

    pub fn entity_flags(&self) -> u32 {
        self.actor.base.entity_flags
    }

    // Состояние

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
        !self.actor.owner.is_null()
    }

    pub fn is_player(&self) -> bool {
        self.factory_type() == 0x10
    }

    pub fn has_valid_self_ref(&self) -> bool {
        let self_ptr = self as *const Self as *mut CHuman;
        self.self_ref == self_ptr
    }
}

// =============================================================================
//  CHumanNPC
// =============================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct CHumanNPC {
    pub base: CHuman,
    pub _npc_data: [u8; 0x20],
}

impl CHumanNPC {
    pub fn as_human(&self) -> &CHuman { &self.base }
    pub fn as_human_mut(&mut self) -> &mut CHuman { &mut self.base }
    pub fn is_npc(&self) -> bool { self.base.factory_type() == 0x0E }
}

// =============================================================================
//  CPlayerSub45C
// =============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPlayerSub45C {
    pub code_a: u32, // +0x00
    pub code_b: u32, // +0x04
    pub state: u32,  // +0x08
}

// =============================================================================
//  CPlayer
// =============================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct CPlayer {
    pub base: CHuman, // +0x000

    pub _player_data_318: [u8; 0x10],
    pub pos_tracking_sentinel: f32, // +0x328
    pub _pad_32C: [u8; 4],
    pub pos_tracking_data: [u8; 8], // +0x330..+0x337
    pub death_position: [f32; 3],   // +0x338
    pub death_type: i32,            // +0x344
    pub _unk_348: [u8; 4],
    pub flags_34C: u32,             // +0x34C
    pub _unk_350: [u8; 0x88],
    pub state_flags_3d8: u32,       // +0x3D8
    pub _pad_3dc: [u8; 0x4C],
    pub field_428: usize,           // +0x428
    pub state_code_430: u32,        // +0x430
    pub _pad_434: [u8; 4],
    pub state_mask_438: u32,        // +0x438
    pub _pad_43C: [u8; 0x20],
    pub sub45c: CPlayerSub45C,      // +0x45C
    pub _pad_468: [u8; 0x28],
    pub state_flags_490: u32,       // +0x490
    pub _pad_494: [u8; 0x24],
    pub _player_tail: [u8; 0x68],   // +0x4B8..+0x51F
}

assert_field_offsets!(CPlayer {
    base                  == 0x000,
    pos_tracking_sentinel == 0x328,
    death_position        == 0x338,
    death_type            == 0x344,
    flags_34C             == 0x34C,
    state_flags_3d8       == 0x3D8,
    field_428             == 0x428,
    state_code_430        == 0x430,
    state_mask_438        == 0x438,
    sub45c                == 0x45C,
    state_flags_490       == 0x490,
});

// Nested offset checks через всю иерархию
const _: () = {
    assert!(std::mem::offset_of!(CPlayer, base.actor.base.table_id) == 0x24);
    assert!(std::mem::offset_of!(CPlayer, base.actor.frame_node) == 0x78);
    assert!(std::mem::offset_of!(CPlayer, base.actor.owner) == 0x80);
    assert!(std::mem::offset_of!(CPlayer, base.current_health) == 0x148);
    assert!(std::mem::offset_of!(CPlayer, base.is_dead) == 0x161);
    assert!(std::mem::offset_of!(CPlayer, base.physics_provider) == 0x258);
};

impl CPlayer {
    pub fn as_human(&self) -> &CHuman { &self.base }
    pub fn as_human_mut(&mut self) -> &mut CHuman { &mut self.base }
    pub fn is_player(&self) -> bool { self.base.factory_type() == 0x10 }
    pub fn is_player_subtype(&self) -> bool { self.base.actor.entity_subtype == 6 }
}