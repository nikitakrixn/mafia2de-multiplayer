use std::ffi::c_void;

/// Общий заголовок entity/human сообщения движка.
///
/// Это базовая часть, которая встречается у всех сообщений.
/// Конкретные типы сообщений добавляют свой payload начиная с `+0x20`.
///
/// Подтверждено:
/// - `message_id` читается из `+0x14` в `M2DE_EntityMessageRegistry_Broadcast`
/// - `+0x0C` автозаполняется broadcaster'ом, если было 0
/// - `+0x1C` ставится в `1` при конструировании сообщения
///
/// Источники:
/// - `M2DE_EntityMessageRegistry_Broadcast` (`0x1403A6DB0`)
/// - `M2DE_HumanEntity_HandleMessage` (`0x140DC2CC0`)
/// - `M2DE_HumanEntity_ProcessDeath` (`0x140DD2460`)
/// - `M2DE_Human_SendEnterVehicleMessage` (`0x140DA4C80`)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EntityMessageHeader {
    /// VTable конкретного типа сообщения.
    ///
    /// По vtable движок получает тип сообщения и данные payload
    /// через виртуальные методы.
    pub vtable: *const c_void, // +0x00

    /// Пока не до конца расшифровано.
    ///
    /// Иногда совпадает с другими значениями сообщения,
    /// поэтому оставляем нейтральное имя.
    pub field_08: u32, // +0x08

    /// ID сущности-широковещателя.
    ///
    /// В `M2DE_EntityMessageRegistry_Broadcast`:
    /// если это поле равно 0, движок записывает сюда `[entity + 0x24]`.
    pub broadcaster_id: u32, // +0x0C

    /// ID сущности, связанной с сообщением.
    ///
    /// Почти всегда sender-path явно пишет сюда `[entity + 0x24]`.
    /// Пока используем нейтральное имя `entity_id`, без слишком смелых выводов.
    pub entity_id: u32, // +0x10

    /// Основной ID сообщения.
    ///
    /// Примеры:
    /// - `0xD0010` = DAMAGE
    /// - `0xD0014` = DEATH
    /// - `0xD001B` = ENTER_VEHICLE
    /// - `0xD001C` = LEAVE_VEHICLE
    pub message_id: u32, // +0x14

    /// Почти всегда `0xFFFFFFFF`.
    pub sentinel: u32, // +0x18

    /// Флаг активности/валидности сообщения.
    ///
    /// Во всех подтверждённых sender-path ставится в `1`.
    pub active_flag: u8, // +0x1C
    pub _pad_1d: [u8; 3], // +0x1D
}

impl EntityMessageHeader {
    /// Упрощённая классификация по "старшим" битам id.
    ///
    /// Практически это совпадает с event_type family:
    /// - `0x5`  -> low-level entity messages
    /// - `0xD`  -> HUMAN messages
    /// - `0x12` -> traffic/AI related
    pub fn message_family(&self) -> u32 {
        self.message_id >> 16
    }

    pub fn is_human_message(&self) -> bool {
        self.message_family() == 0xD
    }
}

/// Payload DAMAGE-сообщения (`0xD0010`).
///
/// Подтверждено по `M2DE_HumanEntity_HandleMessage`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DamageMessagePayload {
    /// Зона попадания / hit area.
    pub hit_area: u32, // +0x20

    /// Часть тела.
    ///
    /// По текущей гипотезе:
    /// - `0x10` = голова
    /// - `0x11..0x13` = другие зоны
    pub body_part: u32, // +0x24

    /// Тип/ID оружия.
    pub weapon_type: u32, // +0x28

    /// Количество урона.
    pub damage_amount: f32, // +0x2C
}

/// Полное DAMAGE-сообщение.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DamageMessage {
    pub header: EntityMessageHeader,
    pub payload: DamageMessagePayload,
}

/// Payload DEATH-сообщения (`0xD0014`).
///
/// Подтверждено по `M2DE_HumanEntity_ProcessDeath`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeathMessagePayload {
    pub pos_x: f32, // +0x20
    pub pos_y: f32, // +0x24

    /// Z оставляем как raw, пока без лишней интерпретации.
    pub pos_z_raw: u32, // +0x28

    /// ID сущности-убийцы.
    pub killer_entity_id: u32, // +0x2C

    /// Тип летального урона.
    pub damage_type: u32, // +0x30

    /// Часть тела, куда пришёл смертельный удар.
    pub body_part: u32, // +0x34

    /// ID оружия.
    pub weapon_id: u32, // +0x38
}

/// Полное DEATH-сообщение.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeathMessage {
    pub header: EntityMessageHeader,
    pub payload: DeathMessagePayload,
}

/// Payload weapon draw/holster сообщений.
///
/// Это уже менее жёстко подтверждённый layout, но поле `weapon_entity_id`
/// выглядит достаточно правдоподобно для практики.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WeaponMessagePayload {
    pub weapon_entity_id: u32, // +0x20
}

/// Полное weapon message.
///
/// Используется для:
/// - `WEAPON_DRAW`
/// - `WEAPON_HOLSTER`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WeaponMessage {
    pub header: EntityMessageHeader,
    pub payload: WeaponMessagePayload,
}

/// Минимальный payload для stance/transition сообщений.
///
/// Семантика пока частично подтверждена, поэтому layout делаем минимальным.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StanceMessagePayload {
    pub flag: u8, // +0x20
}

/// Полное stance message.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StanceMessage {
    pub header: EntityMessageHeader,
    pub payload: StanceMessagePayload,
}

const _: () = {
    // Заголовок
    assert!(std::mem::size_of::<EntityMessageHeader>() == 0x20);
    assert!(std::mem::offset_of!(EntityMessageHeader, vtable) == 0x00);
    assert!(std::mem::offset_of!(EntityMessageHeader, field_08) == 0x08);
    assert!(std::mem::offset_of!(EntityMessageHeader, broadcaster_id) == 0x0C);
    assert!(std::mem::offset_of!(EntityMessageHeader, entity_id) == 0x10);
    assert!(std::mem::offset_of!(EntityMessageHeader, message_id) == 0x14);
    assert!(std::mem::offset_of!(EntityMessageHeader, sentinel) == 0x18);
    assert!(std::mem::offset_of!(EntityMessageHeader, active_flag) == 0x1C);

    // DAMAGE
    assert!(std::mem::size_of::<DamageMessagePayload>() == 0x10);
    assert!(std::mem::offset_of!(DamageMessage, payload) == 0x20);

    // DEATH
    assert!(std::mem::size_of::<DeathMessagePayload>() == 0x1C);
    assert!(std::mem::offset_of!(DeathMessage, payload) == 0x20);

    // Weapon / stance — только минимальные sanity checks
    assert!(std::mem::offset_of!(WeaponMessage, payload) == 0x20);
    assert!(std::mem::offset_of!(StanceMessage, payload) == 0x20);
};