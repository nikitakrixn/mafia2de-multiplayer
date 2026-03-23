//! Структуры гуманоидов — CHuman, CHumanNPC, CPlayer.
//!
//! Иерархия наследования:
//! ```
//! CEntity (0x78)
//!   └─ CActor (+0x78)
//!       └─ CHuman (+0xA8)
//!           ├─ CHumanNPC (~0x338) — тип 0x0E
//!           └─ CPlayer (0x520+)   — тип 0x10
//! ```

use super::{CEntity, Inventory};
use crate::macros::assert_field_offsets;
use std::ffi::c_void;

// =============================================================================
//  CHuman — базовый класс всех гуманоидов
// =============================================================================

/// Базовый класс для NPC и игрока. Содержит общие компоненты,
/// здоровье, физику и параметры модели.
///
/// Размер общей части: до +0x318.
/// Наследуется: CHumanNPC (0x0E), CPlayer (0x10).
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHuman {
    // C_Entity base
    pub base: CEntity, // +0x000

    // C_Actor overlay

    /// Узел трансформации в мировом пространстве.
    pub frame_node: *mut c_void, // +0x078

    /// Транспортное средство, в котором находится персонаж. NULL = пешком.
    pub owner: *mut c_void, // +0x080

    /// Расширенные поля слоя Actor. У гуманоидов обычно 0.
    pub actor_field_88: usize, // +0x088
    pub actor_field_90: usize, // +0x090
    pub actor_field_98: usize, // +0x098

    /// Подтип сущности. У Player = 6.
    pub entity_subtype: u32, // +0x0A0
    pub _pad_0A4: u32, // +0x0A4

    // Компоненты

    /// Параметры ИИ (агрессивность и т.д.). У Player обычно NULL.
    pub ai_params: *mut c_void, // +0x0A8

    /// Внешний компонент (отдельная аллокация, не из общего блока).
    pub external_component_b0: *mut c_void, // +0x0B0

    pub component_b8: *mut c_void, // +0x0B8

    /// Компонент навигации ИИ.
    pub ai_nav: *mut c_void, // +0x0C0

    pub component_c8: *mut c_void, // +0x0C8

    /// Компонент синхронизации трансформации.
    pub transform_sync: *mut c_void, // +0x0D0

    /// Опциональный компонент. Может быть NULL.
    pub optional_component: *mut c_void, // +0x0D8

    pub component_e0: *mut c_void, // +0x0E0

    /// Инвентарь персонажа.
    pub inventory: *mut Inventory, // +0x0E8

    /// Компонент доступа к свойствам.
    pub property_accessor: *mut c_void, // +0x0F0

    /// Компонент поведения.
    pub behavior: *mut c_void, // +0x0F8

    pub component_100: *mut c_void, // +0x100

    /// Компонент состояния оружия.
    pub weapon_state: *mut c_void, // +0x108

    pub component_110: *mut c_void, // +0x110
    pub component_118: *mut c_void, // +0x118
    pub component_120: *mut c_void, // +0x120

    pub _unk_128: [u8; 0x20], // +0x128..+0x147

    // Здоровье и урон

    /// Текущее здоровье. Начальное значение: 210.0.
    pub current_health: f32, // +0x148

    /// Максимальное здоровье (используется у NPC).
    pub field_14c: f32, // +0x14C

    /// Множитель входящего урона от не-игровых источников.
    pub nonplayer_damage_mult: f32, // +0x150

    /// Дистанционный порог для урона от не-игровых источников.
    pub nonplayer_damage_dist: f32, // +0x154

    /// Неизвестный параметр урона/физики.
    pub damage_param_158: f32, // +0x158

    /// Масштабный коэффициент урона.
    pub damage_scale_factor: f32, // +0x15C

    /// Флаг неуязвимости. Ненулевой = урон не применяется.
    pub invulnerability: u8, // +0x160

    /// Флаг смерти. Ненулевой = персонаж мёртв.
    pub is_dead: u8, // +0x161

    /// Режим полубога — персонаж получает урон, но не умирает.
    pub demigod: u8, // +0x162

    /// Вспомогательный флаг состояния. У живого персонажа = 1.
    pub unknown_flag_163: u8, // +0x163

    pub _pad_164: [u8; 4], // +0x164..+0x167

    // Модель и трансформация

    /// Дескриптор внешнего вида персонажа.
    ///
    /// Структура дескриптора:
    /// - +0x00: u32 — идентификатор текущего облика
    /// - +0x0C: u32 — маска флагов совместимости
    /// - +0x18: данные модели
    /// - +0xA8: строка с именем модели (null-terminated)
    pub model_descriptor: *mut c_void, // +0x168

    /// Сохранённый указатель на frame node при смене модели.
    pub saved_frame_ptr: *mut c_void, // +0x170

    /// Сохранённый флаг стриминга при смене модели.
    pub saved_streaming_flag: u8, // +0x178
    pub _pad_179: [u8; 7], // +0x179..+0x17F

    // Зоны повреждений

    /// Массив множителей урона по зонам тела.
    pub body_damage_multipliers: *mut f32, // +0x180

    /// Количество зон тела. Обычно 12.
    pub body_zone_count: u32, // +0x188

    pub _unk_18c: u32, // +0x18C

    /// Самоссылка (this). Используется для проверки валидности объекта.
    pub self_ref: *mut CHuman, // +0x190

    pub _unk_198: u32, // +0x198
    pub _unk_19c: u32, // +0x19C

    pub _pad_1A0: [u8; 0x20], // +0x1A0..+0x1BF

    // Smart pointer slots

    /// 8 слотов умных указателей по 16 байт каждый.
    pub smart_ptr_slots: [[u8; 16]; 8], // +0x1C0..+0x23F

    pub _pad_240: [u8; 0x18], // +0x240..+0x257

    // Физика

    /// Провайдер физики персонажа.
    pub physics_provider: *mut c_void, // +0x258

    // Расширенные поля (общие для NPC и Player) 

    pub _pad_260: [u8; 0x18], // +0x260..+0x277

    /// Вспомогательный объект (32 байта). Создаётся лениво при первом обращении.
    pub helper_object_278: *mut c_void, // +0x278

    pub _pad_280: [u8; 0x14], // +0x280..+0x293

    /// Целевая скорость движения.
    pub movement_speed_target: f32, // +0x294

    /// Текущая скорость движения.
    pub movement_speed_current: f32, // +0x298

    pub _pad_29C: [u8; 0x5C], // +0x29C..+0x2F7

    /// Флаг активности наложения модели.
    pub model_overlay_active: u8, // +0x2F8
    pub _pad_2F9: [u8; 7], // +0x2F9..+0x2FF

    /// Состояние наложения модели.
    pub model_overlay_state: usize, // +0x300

    /// Флаг отсоединения frame node при смене модели.
    pub frame_detach_flag: u8, // +0x308
    pub _pad_309: [u8; 7], // +0x309..+0x30F

    /// Указатель на физическое тело коллизии. Создаётся лениво.
    /// NULL = коллизия отключена. Bit 2 по смещению +2 = активна.
    pub collision_body: *mut c_void, // +0x310
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

impl CHuman {
    /// Тип сущности — младший байт `table_id`.
    pub fn factory_type(&self) -> u8 {
        self.base.factory_type()
    }

    /// Упакованный идентификатор типа и экземпляра.
    pub fn table_id(&self) -> u32 {
        self.base.table_id
    }

    /// Флаги сущности.
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

/// NPC-гуманоид. Тип сущности 0x0E. Размер ~0x338 байт.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CHumanNPC {
    pub base: CHuman, // +0x000
    /// NPC-специфичные поля (не разобраны).
    pub _npc_data: [u8; 0x20], // +0x318..+0x337
}

impl CHumanNPC {
    pub fn as_human(&self) -> &CHuman {
        &self.base
    }

    pub fn as_human_mut(&mut self) -> &mut CHuman {
        &mut self.base
    }

    pub fn is_npc(&self) -> bool {
        self.base.factory_type() == 0x0E
    }
}

// =============================================================================
//  CPlayerSub45C — вложенный объект состояния игрока
// =============================================================================

/// Вложенный объект состояния игрока по смещению +0x45C.
///
/// Хранит текущий код действия и подсостояние.
/// Значения `state`:
/// - 0 = ожидание
/// - 1 = pending
/// - 2 = активно A
/// - 3 = активно B
/// - 4 = отложено
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPlayerSub45C {
    /// Основной код действия.
    pub code_a: u32, // +0x00

    /// Вспомогательный код действия.
    pub code_b: u32, // +0x04

    /// Подсостояние.
    pub state: u32, // +0x08
}

// =============================================================================
//  CPlayer — игрок
// =============================================================================

/// Класс игрока. Тип сущности 0x10. Размер минимум 0x520 байт.
///
/// Отличия от NPC:
/// - `ext_ptr_1/2/3` установлены (дополнительные подсистемы)
/// - `entity_subtype` = 6
/// - Расширенный набор полей состояния от +0x318
#[repr(C)]
#[allow(non_snake_case)]
pub struct CPlayer {
    pub base: CHuman, // +0x000

    pub _player_data_318: [u8; 0x10], // +0x318..+0x327

    /// Сентинел отслеживания позиции. -FLT_MAX = не используется.
    pub pos_tracking_sentinel: f32, // +0x328
    pub _pad_32C: [u8; 4], // +0x32C..+0x32F

    /// Данные отслеживания позиции.
    pub pos_tracking_data: [u8; 8], // +0x330..+0x337

    /// Позиция смерти / точка возрождения.
    pub death_position: [f32; 3], // +0x338

    /// Тип смерти.
    pub death_type: i32, // +0x344

    pub _unk_348: [u8; 4], // +0x348..+0x34B

    /// Флаги. Биты 0-1 сбрасываются при десериализации.
    pub flags_34C: u32, // +0x34C

    pub _unk_350: [u8; 0x88], // +0x350..+0x3D7

    /// Флаги состояния игрока.
    ///
    /// Младший байт: значения 3 и 4 имеют особое значение.
    /// Старшие биты: флаги режимов (например, 0x40000).
    pub state_flags_3d8: u32, // +0x3D8

    pub _pad_3dc: [u8; 0x4C], // +0x3DC..+0x427

    /// Указатель вспомогательного объекта.
    pub field_428: usize, // +0x428

    /// Код текущего состояния. Значение 10 имеет особое значение.
    pub state_code_430: u32, // +0x430

    pub _pad_434: [u8; 4], // +0x434..+0x437

    /// Маска/профиль состояния. Загружается по имени.
    pub state_mask_438: u32, // +0x438

    pub _pad_43C: [u8; 0x20], // +0x43C..+0x45B

    /// Вложенный объект состояния.
    pub sub45c: CPlayerSub45C, // +0x45C

    pub _pad_468: [u8; 0x28], // +0x468..+0x48F

    /// Битовое поле флагов состояния.
    ///
    /// - биты [1..3]:  группа A
    /// - биты [4..6]:  группа B
    /// - биты [7..13]: группа C
    /// - бит  [14]:    флаг D
    /// - бит  [15]:    флаг E
    pub state_flags_490: u32, // +0x490

    pub _pad_494: [u8; 0x24], // +0x494..+0x4B7

    pub _player_tail: [u8; 0x68], // +0x4B8..+0x51F
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

impl CPlayer {
    pub fn as_human(&self) -> &CHuman {
        &self.base
    }

    pub fn as_human_mut(&mut self) -> &mut CHuman {
        &mut self.base
    }

    pub fn is_player(&self) -> bool {
        self.base.factory_type() == 0x10
    }

    pub fn is_player_subtype(&self) -> bool {
        self.base.entity_subtype == 6
    }

    /// Проверка: младший байт `state_flags_3d8` не равен 3 и не равен 4.
    pub fn is_state_3d8_lowbyte_not_3_or_4(&self) -> bool {
        let low = (self.state_flags_3d8 & 0xFF) as u8;
        low != 3 && low != 4
    }

    pub fn has_state_flag_3d8(&self, mask: u32) -> bool {
        (self.state_flags_3d8 & mask) != 0
    }

    /// Проверка: `state_code_430` равен 10.
    pub fn is_state_code_430_equal_10(&self) -> bool {
        self.state_code_430 == 10
    }

    /// Проверка: `sub45c.state` равен 1.
    pub fn is_sub45c_state_equal_1(&self) -> bool {
        self.sub45c.state == 1
    }

    /// Проверка: `sub45c.state` равен 2 или 3.
    pub fn is_sub45c_state_2_or_3(&self) -> bool {
        (self.sub45c.state.wrapping_sub(2)) <= 1
    }

    /// Проверка: `sub45c.state` равен 4.
    pub fn is_sub45c_state_equal_4(&self) -> bool {
        self.sub45c.state == 4
    }

    pub fn get_state_flags_bits_1_3(&self) -> u32 {
        use crate::addresses::constants::player_state_flags_490::MASK_BITS_1_3;
        (self.state_flags_490 & MASK_BITS_1_3) >> 1
    }

    pub fn get_state_flags_bits_4_6(&self) -> u32 {
        use crate::addresses::constants::player_state_flags_490::MASK_BITS_4_6;
        (self.state_flags_490 & MASK_BITS_4_6) >> 4
    }

    pub fn get_state_flags_bits_7_13(&self) -> u32 {
        use crate::addresses::constants::player_state_flags_490::MASK_BITS_7_13;
        (self.state_flags_490 & MASK_BITS_7_13) >> 7
    }

    pub fn is_state_flag_bit_14_set(&self) -> bool {
        use crate::addresses::constants::player_state_flags_490::BIT_14;
        (self.state_flags_490 & BIT_14) != 0
    }

    pub fn is_state_flag_bit_15_set(&self) -> bool {
        use crate::addresses::constants::player_state_flags_490::BIT_15;
        (self.state_flags_490 & BIT_15) != 0
    }
}
