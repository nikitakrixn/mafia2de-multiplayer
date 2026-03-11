//! Система entity/human сообщений движка.
//!
//! Движок рассылает сообщения через EntityMessageRegistry_Broadcast.
//! Каждое сообщение начинается с EntityMessageHeader (0x20 байт),
//! за которым идёт payload конкретного типа.
//!
//! Подтверждённые источники:
//! - M2DE_EntityMessageRegistry_Broadcast (0x1403A6DB0)
//! - M2DE_HumanEntity_HandleMessage (0x140DC2CC0)
//! - M2DE_HumanEntity_ProcessDeath (0x140DD2460)
//! - M2DE_Human_SendEnterVehicleMessage (0x140DA4C80)

use std::ffi::c_void;
use crate::macros::{assert_field_offsets, assert_layout};

/// Общий заголовок entity/human сообщения.
///
/// Все сообщения начинаются с этой структуры (0x20 байт).
/// Тип сообщения определяется по `message_id`.
/// Payload идёт сразу за заголовком начиная с +0x20.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EntityMessageHeader {
    /// VTable конкретного типа сообщения.
    /// Через vtable движок определяет тип и читает payload.
    pub vtable: *const c_void,              // +0x00

    /// Пока не расшифровано полностью.
    pub field_08: u32,                      // +0x08

    /// ID сущности-широковещателя.
    /// Если 0 при отправке — движок заполнит из [entity + 0x24].
    pub broadcaster_id: u32,                // +0x0C

    /// ID сущности, связанной с сообщением.
    /// Sender-path обычно пишет сюда [entity + 0x24].
    pub entity_id: u32,                     // +0x10

    /// Основной ID сообщения.
    /// Примеры: 0xD0010 (DAMAGE), 0xD0014 (DEATH),
    /// 0xD001B (ENTER_VEHICLE), 0xD001C (LEAVE_VEHICLE).
    pub message_id: u32,                    // +0x14

    /// Почти всегда 0xFFFFFFFF.
    pub sentinel: u32,                      // +0x18

    /// Флаг активности. Во всех sender-path ставится в 1.
    pub active_flag: u8,                    // +0x1C
    pub _pad_1d: [u8; 3],                   // +0x1D
}

impl EntityMessageHeader {
    /// Классификация по старшим битам message_id.
    /// 0x5 = low-level entity, 0xD = HUMAN, 0x12 = traffic/AI.
    pub fn message_family(&self) -> u32 {
        self.message_id >> 16
    }

    pub fn is_human_message(&self) -> bool {
        self.message_family() == 0xD
    }
}

/// Payload DAMAGE-сообщения (0xD0010).
///
/// Подтверждено по M2DE_HumanEntity_HandleMessage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DamageMessagePayload {
    /// Зона попадания.
    pub hit_area: u32,                      // +0x20
    /// Часть тела (0x10 = голова, 0x11..0x13 = другие зоны).
    pub body_part: u32,                     // +0x24
    /// Тип/ID оружия, нанёсшего урон.
    pub weapon_type: u32,                   // +0x28
    /// Количество урона.
    pub damage_amount: f32,                 // +0x2C
}

/// Полное DAMAGE-сообщение = заголовок + payload.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DamageMessage {
    pub header: EntityMessageHeader,
    pub payload: DamageMessagePayload,
}

/// Payload DEATH-сообщения (0xD0014).
///
/// Подтверждено по M2DE_HumanEntity_ProcessDeath.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeathMessagePayload {
    /// Позиция смерти X.
    pub pos_x: f32,                         // +0x20
    /// Позиция смерти Y.
    pub pos_y: f32,                         // +0x24
    /// Позиция Z (raw, без интерпретации).
    pub pos_z_raw: u32,                     // +0x28
    /// ID сущности-убийцы.
    pub killer_entity_id: u32,              // +0x2C
    /// Тип летального урона.
    pub damage_type: u32,                   // +0x30
    /// Часть тела, куда пришёл смертельный удар.
    pub body_part: u32,                     // +0x34
    /// ID оружия.
    pub weapon_id: u32,                     // +0x38
}

/// Полное DEATH-сообщение.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeathMessage {
    pub header: EntityMessageHeader,
    pub payload: DeathMessagePayload,
}

/// Payload для weapon draw/holster сообщений.
///
/// Менее жёстко подтверждённый layout, но weapon_entity_id
/// выглядит правдоподобно по runtime-дампам.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WeaponMessagePayload {
    /// ID сущности оружия.
    pub weapon_entity_id: u32,              // +0x20
}

/// Weapon message (для WEAPON_DRAW и WEAPON_HOLSTER).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WeaponMessage {
    pub header: EntityMessageHeader,
    pub payload: WeaponMessagePayload,
}

/// Минимальный payload для stance/transition сообщений.
/// Семантика пока частично подтверждена.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StanceMessagePayload {
    pub flag: u8,                           // +0x20
}

/// Полное stance message.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StanceMessage {
    pub header: EntityMessageHeader,
    pub payload: StanceMessagePayload,
}

// ═══════════════════════════════════════════════════════════════════
//  Compile-time проверки layout'ов
// ═══════════════════════════════════════════════════════════════════

assert_layout!(EntityMessageHeader, size = 0x20, {
    vtable         == 0x00,
    field_08       == 0x08,
    broadcaster_id == 0x0C,
    entity_id      == 0x10,
    message_id     == 0x14,
    sentinel       == 0x18,
    active_flag    == 0x1C,
});

assert_layout!(DamageMessagePayload, size = 0x10, {
    hit_area      == 0x00,
    body_part     == 0x04,
    weapon_type   == 0x08,
    damage_amount == 0x0C,
});

assert_field_offsets!(DamageMessage {
    payload == 0x20,
});

assert_layout!(DeathMessagePayload, size = 0x1C, {
    pos_x            == 0x00,
    pos_y            == 0x04,
    pos_z_raw        == 0x08,
    killer_entity_id == 0x0C,
    damage_type      == 0x10,
    body_part        == 0x14,
    weapon_id        == 0x18,
});

assert_field_offsets!(DeathMessage {
    payload == 0x20,
});

assert_field_offsets!(WeaponMessage {
    payload == 0x20,
});

assert_field_offsets!(StanceMessage {
    payload == 0x20,
});