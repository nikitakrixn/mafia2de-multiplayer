//! Структура игрока (C_Human / C_Player).
//!
//! Все поля восстановлены из IDA Pro + runtime проверок.
//! Процент уверенности указан в комментариях к каждому полю.

use std::ffi::c_void;
use super::Inventory;
use crate::macros::assert_field_offsets;

/// Игрок (C_Human / C_Player).
///
/// Движок использует C_Human для всех humanoid-сущностей.
/// Для локального игрока (entity_type=0x10) указатель
/// берётся из GameManager+0x180.
///
/// vtable: 0x14184C060 (игрок), 0x1418E5188 (NPC)
///
/// Ключевые подтверждённые поля:
/// - `frame_node` (+0x78) — transform node с матрицей (позиция мира)
/// - `owner` (+0x80) — NULL пешком, vehicle* в машине
/// - `inventory` (+0xE8) — инвентарь с оружием и деньгами
/// - `current_health` (+0x148) — текущее здоровье (float)
/// - `invulnerability` (+0x160) — флаг неуязвимости
/// - `is_dead` (+0x161) — флаг смерти
/// - `demigod` (+0x162) — флаг полубога
/// - `self_ref` (+0x190) — указатель на самого себя (валидация)
#[repr(C)]
pub struct CHuman {
    // === Базовый C_Entity ===

    /// Указатель на vtable
    pub vtable: *const c_void,                   // +0x000
    /// Связи в графе сцены (linked list).
    _scene_graph: [u8; 0x1C],                    // +0x008..+0x024
    /// Тип сущности: 0x10=игрок, 0x0E=NPC, 0x12=физ. тело
    pub entity_type: u8,                         // +0x024
    _pad_025: [u8; 0x03],                        // +0x025
    /// Флаги сущности (бит 15 меняется при смене состояния)
    pub entity_flags: u32,                       // +0x028
    _pad_02C: [u8; 0x0C],                        // +0x02C
    /// GUID сущности (16 байт)
    pub guid: [u8; 16],                          // +0x038
    _entity_pad: [u8; 0x30],                     // +0x048..+0x078

    // === C_Actor ===

    /// Frame/transform node — хранит мировую позицию.
    /// Координаты: frame+0x64 (X), frame+0x74 (Y), frame+0x84 (Z)
    pub frame_node: *mut c_void,                 // +0x078
    /// Владелец: NULL пешком, vehicle* в машине
    pub owner: *mut c_void,                      // +0x080

    // === Указатели на компоненты (inline внутри C_Human) ===

    /// Пустые слоты (всегда 0)
    _slots_088: [u8; 0x18],                      // +0x088..+0x0A0
    /// Подтип сущности (= 6 для человека)
    pub entity_subtype: u32,                     // +0x0A0
    _pad_0A4: u32,                               // +0x0A4
    _component_0A8: *mut c_void,                 // +0x0A8
    _entity_id_hash: u32,                        // +0x0B0
    _pad_0B4: u32,                               // +0x0B4
    _component_0B8: *mut c_void,                 // +0x0B8
    /// AI/навигационный компонент (vtbl 0x1418E3290)
    pub ai_nav: *mut c_void,                     // +0x0C0
    _component_0C8: *mut c_void,                 // +0x0C8
    /// Компонент синхронизации позиции (vtbl 0x1418E33A8)
    pub transform_sync: *mut c_void,             // +0x0D0
    /// Опциональный компонент, создаётся по запросу (msg 0xD0059)
    pub optional_component: *mut c_void,         // +0x0D8
    _component_0E0: *mut c_void,                 // +0x0E0
    /// Инвентарь (оружие, деньги, предметы). vtbl 0x1418E3090
    pub inventory: *mut Inventory,               // +0x0E8
    /// Property accessor — первый qword = back-ref на self (НЕ vtable!)
    pub property_accessor: *mut c_void,          // +0x0F0
    /// Behavior компонент — сюда пересылаются сообщения. vtbl 0x1418E37A8
    pub behavior: *mut c_void,                   // +0x0F8
    _component_100: *mut c_void,                 // +0x100
    /// Компонент состояния оружия. *(this + 0x2B0) → ID оружия
    pub weapon_state: *mut c_void,               // +0x108
    _components_110: [u8; 0x18],                 // +0x110..+0x128
    _pad_128: [u8; 0x18],                        // +0x128..+0x140

    // === Здоровье и урон ===

    /// Сущность крепления к транспорту (set/clear по msg 0x50002)
    pub vehicle_mount: *mut c_void,              // +0x140
    /// Текущее здоровье. 720.0 = полное (нормальная сложность)
    pub current_health: f32,                     // +0x148
    /// NPC: максимум здоровья. Игрок: множитель типов урона 12/15/22
    pub npc_healthmax_or_type_mult: f32,         // +0x14C
    /// Множитель урона от NPC (Lua: *100 = проценты). Default 1.0
    pub nonplayer_damage_mult: f32,              // +0x150
    /// Пороговая дистанция для снижения урона. Default 5.0
    pub nonplayer_damage_dist: f32,              // +0x154
    _pad_158: [u8; 0x08],                        // +0x158..+0x160

    // === Флаги состояния ===

    /// Неуязвимость. 0=обычный, 1=неуязвим (урон полностью пропускается)
    pub invulnerability: u8,                     // +0x160
    /// Флаг смерти. IsDeath() = return *(this + 0x161)
    pub is_dead: u8,                             // +0x161
    /// Режим полубога. Здоровье не падает ниже 1.0 при уроне
    pub demigod: u8,                             // +0x162
    _pad_163: [u8; 0x1D],                        // +0x163..+0x180

    // === Урон по частям тела ===

    /// Массив множителей урона: [4]=голова, [5]=торс, [6]=руки, [7]=ноги.
    pub body_damage_multipliers: *mut f32,       // +0x180
    _pad_188: [u8; 0x08],                        // +0x188

    // === Ссылка на себя ===

    /// Указатель на самого себя. Валидация: self_ref == this.
    pub self_ref: *mut CHuman,                   // +0x190

    // === NPC-специфичные поля (за пределами +0x190) ===

    /// `+0x198` → int32 entity_table_index (выделяется при создании)
    pub entity_table_index: i32,                 // +0x198
    /// `+0x19C` → int32 entity_table_slot
    pub entity_table_slot: i32,                  // +0x19C

    _pad_1A0: [u8; 0x20],                       // +0x1A0..+0x1C0

    /// `+0x1C0` → 8 smart pointer слотов (16 bytes каждый).
    /// IDs: 1,2,3,4,5,6,7,-1. Назначение неизвестно (AI states?).
    pub smart_ptr_slots: [[u8; 16]; 8],          // +0x1C0..+0x240

    _npc_pad_240: [u8; 0x18],                   // +0x240..+0x258
}

assert_field_offsets!(CHuman {
    vtable                     == 0x000,
    entity_type                == 0x024,
    entity_flags               == 0x028,
    guid                       == 0x038,
    frame_node                 == 0x078,
    owner                      == 0x080,
    entity_subtype             == 0x0A0,
    ai_nav                     == 0x0C0,
    transform_sync             == 0x0D0,
    optional_component         == 0x0D8,
    inventory                  == 0x0E8,
    property_accessor          == 0x0F0,
    behavior                   == 0x0F8,
    weapon_state               == 0x108,
    vehicle_mount              == 0x140,
    current_health             == 0x148,
    npc_healthmax_or_type_mult == 0x14C,
    nonplayer_damage_mult      == 0x150,
    nonplayer_damage_dist      == 0x154,
    invulnerability            == 0x160,
    is_dead                    == 0x161,
    demigod                    == 0x162,
    body_damage_multipliers    == 0x180,
    self_ref                   == 0x190,
});

impl CHuman {
    /// Текущее здоровье.
    pub fn health(&self) -> f32 {
        self.current_health
    }

    /// Жив ли персонаж.
    pub fn is_alive(&self) -> bool {
        self.is_dead == 0
    }

    /// Проверка флага неуязвимости.
    pub fn is_invulnerable(&self) -> bool {
        self.invulnerability != 0
    }

    /// Проверка режима полубога.
    pub fn is_demigod(&self) -> bool {
        self.demigod != 0
    }

    /// Находится ли в транспорте.
    pub fn is_in_vehicle(&self) -> bool {
        !self.owner.is_null()
    }

    /// Является ли это игроком (а не NPC).
    pub fn is_player(&self) -> bool {
        self.entity_type == 0x10
    }

    /// Валидация указателя через self_ref.
    pub fn is_valid_ptr(&self) -> bool {
        let self_ptr = self as *const Self;
        self.self_ref == self_ptr as *mut CHuman
    }
}