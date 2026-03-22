//! Структуры игрока и NPC (CHuman / CHumanNPC / CPlayer).
//!
//! Иерархия наследования:
//! ```
//! CEntity (0x78)
//!   └─ CActor (+0x78)
//!       └─ CHuman (+0xA8)
//!           ├─ CHumanNPC (~0x338) — factory type 0x0E
//!           └─ CPlayer (0x520+)   — factory type 0x10
//! ```
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


// =============================================================================
//  CHumanNPC — NPC гуманоид
// =============================================================================

/// CHumanNPC — конкретный класс для NPC.
///
/// Размер: ~0x338 байт (точный размер требует уточнения).
/// Конструктор: `M2DE_CHumanNPC_Constructor` (0x140D712E0).
/// Vtable: 0x1418E5188.
/// Factory type: 0x0E.
///
/// Отличия от Player:
/// - Нет ext_ptr_1/2/3 (всегда NULL)
/// - Нет Player-specific полей от +0x338
/// - Другая vtable (меньше методов)
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHumanNPC {
    /// Базовый CHuman.
    pub base: CHuman, // +0x000..+0x25F

    /// NPC-specific поля от +0x260 до ~0x338.
    /// Пока не полностью разобраны, оставляем как raw.
    pub _npc_data: [u8; 0xD8], // +0x260..+0x337
}

impl CHumanNPC {
    pub fn as_human(&self) -> &CHuman {
        &self.base
    }

    pub fn as_human_mut(&mut self) -> &mut CHuman {
        &mut self.base
    }

    /// Проверка что это действительно NPC (factory type = 0x0E).
    pub fn is_npc(&self) -> bool {
        self.base.factory_type() == 0x0E
    }
}

// =============================================================================
//  CPlayer — игрок
// =============================================================================

/// CPlayer — класс игрока.
///
/// Размер: минимум 0x520 байт (возможно больше).
/// Конструктор: `M2DE_CPlayerEntity_Constructor` (0x1400B9160).
/// Vtable: 0x14184C060.
/// Factory type: 0x10.
///
/// Отличия от NPC:
/// - ext_ptr_1/2/3 установлены (heap pointers)
/// - entity_subtype = 6
/// - Множество Player-specific полей от +0x338
/// - Расширенная vtable с дополнительными методами
///
/// Player-specific поля (подтверждены vtable):
/// - +0x338: death_position (Vec3)
/// - +0x344: death_type (i32)
/// - +0x3D8: mode_3d8 (u8)
/// - +0x430: state_code_430 (u32)
/// - +0x464: field_464 (u32)
/// - +0x490: state_flags_490 (u32 bitfield)
#[repr(C)]
#[allow(non_snake_case)]
pub struct CPlayer {
    /// Базовый CHuman.
    pub base: CHuman, // +0x000..+0x25F

    /// Player-specific данные от +0x260 до +0x337.
    pub _player_data_260: [u8; 0xD8], // +0x260..+0x337

    /// Death position (Vec3).
    pub death_position: [f32; 3], // +0x338

    /// Death type / death mode.
    pub death_type: i32, // +0x344

    /// Неизвестные поля между death_type и mode_3d8.
    pub _unk_348: [u8; 0x90], // +0x348..+0x3D7

    /// Mode/state byte.
    /// Проверяется vtable[94]: `IsMode3D8_Not3Or4` (значение != 3 && != 4).
    pub mode_3d8: u8, // +0x3D8

    /// Padding до state_code_430.
    pub _pad_3d9: [u8; 0x57], // +0x3D9..+0x42F

    /// State code dword.
    /// Проверяется vtable[83]: `IsStateCode430_Equal10` (значение == 10).
    pub state_code_430: u32, // +0x430

    /// Padding до field_464.
    pub _pad_434: [u8; 0x30], // +0x434..+0x463

    /// Dword-like bool/state field.
    /// Проверяется vtable[102]: `IsField464_Equal1` (значение == 1).
    pub field_464: u32, // +0x464

    /// Padding до state_flags_490.
    pub _pad_468: [u8; 0x28], // +0x468..+0x48F

    /// Player state flags bitfield.
    ///
    /// Управляется через 5 vtable методов:
    /// - vtable[96]: SetClearMaskedBits1_3 (биты 1-3)
    /// - vtable[97]: SetFieldBits4_6 (биты 4-6)
    /// - vtable[98]: SetClearMaskedBits7_13 (биты 7-13)
    /// - vtable[99]: SetBit14 (бит 14)
    /// - vtable[100]: SetBit15 (бит 15)
    ///
    /// См. `sdk::addresses::constants::player_state_flags_490` для масок.
    pub state_flags_490: u32, // +0x490

    /// Остальные Player-specific поля.
    /// Размер неизвестен, минимум до +0x520.
    pub _player_data_494: [u8; 0x8C], // +0x494..+0x51F
}

assert_field_offsets!(CPlayer {
    base             == 0x000,
    death_position   == 0x338,
    death_type       == 0x344,
    mode_3d8         == 0x3D8,
    state_code_430   == 0x430,
    field_464        == 0x464,
    state_flags_490  == 0x490,
});

impl CPlayer {
    pub fn as_human(&self) -> &CHuman {
        &self.base
    }

    pub fn as_human_mut(&mut self) -> &mut CHuman {
        &mut self.base
    }

    /// Проверка что это действительно Player (factory type = 0x10).
    pub fn is_player(&self) -> bool {
        self.base.factory_type() == 0x10
    }

    /// Проверка entity_subtype (должен быть 6 для Player).
    pub fn is_player_subtype(&self) -> bool {
        self.base.entity_subtype == 6
    }

    /// Проверка mode_3d8 (не равен 3 и не равен 4).
    ///
    /// Соответствует vtable[94]: `M2DE_CPlayer_IsMode3D8_Not3Or4`.
    pub fn is_mode_3d8_not_3_or_4(&self) -> bool {
        self.mode_3d8 != 3 && self.mode_3d8 != 4
    }

    /// Проверка state_code_430 (равен 10).
    ///
    /// Соответствует vtable[83]: `M2DE_CPlayer_IsStateCode430_Equal10`.
    pub fn is_state_code_430_equal_10(&self) -> bool {
        self.state_code_430 == 10
    }

    /// Проверка field_464 (равен 1).
    ///
    /// Соответствует vtable[102]: `M2DE_CPlayer_IsField464_Equal1`.
    pub fn is_field_464_equal_1(&self) -> bool {
        self.field_464 == 1
    }

    /// Получить биты [1..3] из state_flags_490.
    pub fn get_state_flags_bits_1_3(&self) -> u32 {
        use crate::addresses::constants::player_state_flags_490::MASK_BITS_1_3;
        (self.state_flags_490 & MASK_BITS_1_3) >> 1
    }

    /// Получить биты [4..6] из state_flags_490.
    pub fn get_state_flags_bits_4_6(&self) -> u32 {
        use crate::addresses::constants::player_state_flags_490::MASK_BITS_4_6;
        (self.state_flags_490 & MASK_BITS_4_6) >> 4
    }

    /// Получить биты [7..13] из state_flags_490.
    pub fn get_state_flags_bits_7_13(&self) -> u32 {
        use crate::addresses::constants::player_state_flags_490::MASK_BITS_7_13;
        (self.state_flags_490 & MASK_BITS_7_13) >> 7
    }

    /// Проверить бит 14 в state_flags_490.
    pub fn is_state_flag_bit_14_set(&self) -> bool {
        use crate::addresses::constants::player_state_flags_490::BIT_14;
        (self.state_flags_490 & BIT_14) != 0
    }

    /// Проверить бит 15 в state_flags_490.
    pub fn is_state_flag_bit_15_set(&self) -> bool {
        use crate::addresses::constants::player_state_flags_490::BIT_15;
        (self.state_flags_490 & BIT_15) != 0
    }
}
