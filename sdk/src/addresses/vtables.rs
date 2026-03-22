//! Адреса виртуальных таблиц
//!
//! Источники:
//! - runtime EntityDatabase scan (`2415` entities, FreeRide)
//! - IDA Pro static analysis
//! - constructor vtable assignments
//! - xref анализ к vtable
//!
//! ВАЖНО:
//! - это именно RVA, а не абсолютные адреса
//! - для получения VA нужно: `module_base + RVA`
//! - часть vtable относится к top-level entity
//! - часть — к child/subobject path
//! - спорные/старые адреса явно помечены как `*_UNUSED` или `*_RTTI_LIKE`

pub mod entity {
    // =========================================================================
    //  Базовая иерархия Entity
    // =========================================================================

    /// Базовая vtable `C_Entity`.
    ///
    /// Корень иерархии сущностей.
    /// Constructor: `M2DE_BaseEntity_Construct` (`0x14039B710`)
    pub const BASE: usize = 0x186_CAC8;

    /// Vtable `C_Actor`.
    ///
    /// Расширяет `C_Entity` полями трансформации и owner.
    /// Constructor: `M2DE_ActorEntity_Construct` (`0x14039A7E0`)
    pub const ACTOR: usize = 0x186_D050;

    /// Абстрактная actor-vtable с `_purecall` в ctor-цепочке.
    ///
    /// Используется в `CHuman_BaseConstructor` до финальной замены на
    /// NPC/player-specific vtable.
    pub const ACTOR_ABSTRACT: usize = 0x186_CF50;

    // =========================================================================
    //  Инфраструктура entity system
    // =========================================================================

    /// Vtable `WorldEntityManager` / `EntityDatabase`.
    ///
    /// Один и тот же объект доступен через:
    /// - `g_WorldEntityManager`
    /// - `g_EntityDatabase`
    ///
    /// Runtime подтверждение:
    /// - `count` по `+0x18`
    /// - open-addressing array по `+0x38`
    pub const WORLD_ENTITY_MANAGER: usize = 0x186_F360;

    /// Vtable модуля `C_EntityList`.
    /// Module ID = `ENTITY_LIST (9)`.
    pub const ENTITY_LIST: usize = 0x186_F410;

    /// Общая vtable для wrapper factory объектов.
    pub const WRAPPER_FACTORY: usize = 0x191_8858;

    /// Vtable `C_ServiceIdentity`.
    pub const SERVICE_IDENTITY: usize = 0x184_A8F0;

    /// Vtable observer-объекта script wrapper.
    pub const ENTITY_OBSERVER: usize = 0x191_9A78;
}

pub mod human {
    /// Vtable `C_HumanNPC`.
    ///
    /// Runtime:
    /// - factory type = `0x0E`
    /// - count = `86`
    ///
    /// Constructor:
    /// - `M2DE_CHumanNPC_Constructor` (`0x140D712E0`)
    pub const NPC: usize = 0x18E_5188;

    /// Vtable `C_Player`.
    ///
    /// Runtime:
    /// - factory type = `0x10`
    /// - count = `1`
    ///
    /// Constructor:
    /// - `M2DE_CPlayerEntity_Constructor` (`0x1400B9160`)
    pub const PLAYER: usize = 0x184_C060;

    /// Vtable Lua-wrapper'а `C_WrapperPlayer`.
    pub const WRAPPER_PLAYER: usize = 0x1C3_3648;

    /// Старый адрес рядом со строкой `C_Human`.
    ///
    /// Не использовать как runtime primary vtable.
    pub const HUMAN_RTTI_LIKE_UNUSED: usize = 0x18E_94E0;
}

pub mod car {
    // =========================================================================
    //  Основные runtime vehicle class paths
    // =========================================================================

    /// Vtable `C_Car`.
    ///
    /// Это parked/static car path:
    /// - factory type = `0x12`
    /// - runtime count = `41`
    /// - constructor = `M2DE_CCar_Construct` (`0x1400EE6C0`)
    /// - destructor  = `0x1400F4610`
    /// - create-instance alloc size = `0x1258` (`4696`)
    pub const CAR: usize = 0x185_0030;

    /// Vtable `C_CarVehicle`.
    ///
    /// Это drivable/active vehicle path:
    /// - factory type = `0x70`
    /// - runtime count = `1`
    /// - constructor = `M2DE_CCarVehicle_Construct` (`0x140DF3360`)
    /// - likely destructor slot0 = `0x140DF9020`
    /// - create-instance alloc size = `0x2F0` (`752`)
    ///
    /// ВАЖНО:
    /// `C_Car` и `C_CarVehicle` — разные runtime class/vtable.
    pub const CAR_VEHICLE: usize = 0x18E_AAC8;

    // =========================================================================
    //  Промежуточные vtable в ctor-цепочке `C_Car`
    // =========================================================================

    /// Промежуточная vtable в ctor-цепочке `C_Car`.
    pub const CAR_INTERMEDIATE_1: usize = 0x185_04C0;

    /// Промежуточная vtable в ctor-цепочке `C_Car`.
    pub const CAR_INTERMEDIATE_2: usize = 0x185_04E0;

    /// Промежуточная vtable в ctor-цепочке `C_Car`.
    pub const CAR_INTERMEDIATE_3: usize = 0x185_04F0;

    /// Поздняя vtable / helper-object path в ctor-цепочке `C_Car`.
    pub const CAR_LATE: usize = 0x184_FE68;

    // =========================================================================
    //  Multiple inheritance sub-vtables у `C_CarVehicle`
    // =========================================================================

    /// Sub-vtable `C_CarVehicle` по смещению `+0xA8`.
    pub const CAR_VEHICLE_SUB1: usize = 0x18E_AC60;

    /// Sub-vtable `C_CarVehicle` по смещению `+0xB0`.
    pub const CAR_VEHICLE_SUB2: usize = 0x18E_AC98;

    /// Sub-vtable `C_CarVehicle` по смещению `+0xB8`.
    pub const CAR_VEHICLE_SUB3: usize = 0x18E_ACC8;

    /// Sub-vtable `C_CarVehicle` по смещению `+0xC0`.
    pub const CAR_VEHICLE_SUB4: usize = 0x18E_ACD8;

    // =========================================================================
    //  Старый ошибочный адрес
    // =========================================================================

    /// Старый адрес рядом со строкой `C_Car`.
    ///
    /// По xref не используется как runtime primary vtable.
    /// Оставлено только как напоминание о старой ошибочной гипотезе.
    pub const CAR_RTTI_LIKE_UNUSED: usize = 0x18E_8908;
}

pub mod traffic {
    /// Vtable `C_TrafficCar`.
    pub const TRAFFIC_CAR: usize = 0x18D_1EC8;

    /// Vtable `C_TrafficHuman`.
    pub const TRAFFIC_HUMAN: usize = 0x18D_2320;

    /// Vtable `C_TrafficTrain`.
    pub const TRAFFIC_TRAIN: usize = 0x18D_2090;
}

pub mod world_objects {
    /// Vtable `C_CrashObject`.
    pub const CRASH_OBJECT: usize = 0x18E_8D00;

    /// Vtable `C_DamageZone` (outer world entity).
    ///
    /// Runtime:
    /// - type = `0x1E`
    /// - count = `109`
    /// - outer size = `0xD8`
    ///
    /// Constructor:
    /// - `M2DE_CDamageZone_Construct` (`0x140C0E8A0`)
    ///
    /// ВАЖНО:
    /// это primary outer vtable.
    /// Child script-object внутри него использует:
    /// - `script_entity::DAMAGE_ZONE_CHILD`
    pub const DAMAGE_ZONE: usize = 0x18D_0A78;

    /// Vtable `C_Item`.
    pub const ITEM: usize = 0x18E_9C38;

    /// Vtable `C_Wardrobe`.
    pub const WARDROBE: usize = 0x190_9A60;

    /// Vtable `C_Door`.
    pub const DOOR: usize = 0x185_1BD0;

    /// Vtable `Tree`.
    pub const TREE: usize = 0x18E_9700;

    /// Vtable `C_Grenade`.
    pub const GRENADE: usize = 0x190_BB00;

    /// Vtable `C_StaticWeapon`.
    pub const STATIC_WEAPON: usize = 0x190_C720;

    /// Vtable `StaticEntity`.
    pub const STATIC_ENTITY: usize = 0x18D_0F40;

    /// Vtable `C_Pinup`.
    pub const PINUP: usize = 0x18E_B198;

    /// Vtable `C_DummyDoor`.
    pub const DUMMY_DOOR: usize = 0x190_D2C0;

    /// Vtable `C_Blocker`.
    pub const BLOCKER: usize = 0x187_39A0;

    /// Vtable `C_ActorDetector`.
    pub const ACTOR_DETECTOR: usize = 0x187_3AB0;

    /// Vtable `C_CleanEntity`.
    pub const CLEAN_ENTITY: usize = 0x18D_34B0;
}

pub mod sound {
    /// Vtable `C_Sound`.
    pub const SOUND: usize = 0x18E_89D0;

    /// Vtable `C_SoundMixer`.
    pub const SOUND_MIXER: usize = 0x18E_B868;
}

pub mod action_points {
    /// Vtable `C_ActionPointBase`.
    pub const BASE: usize = 0x18C_FC20;

    /// Vtable `C_ActionPointCrossing`.
    pub const CROSSING: usize = 0x18E_9568;

    /// Vtable `C_ActionPointSideWalk`.
    pub const SIDEWALK: usize = 0x18E_A8B8;

    /// Vtable `C_ActionPointScript`.
    pub const SCRIPT: usize = 0x190_CD70;

    /// Vtable `C_ActionPointSearch`.
    pub const SEARCH: usize = 0x18E_A470;
}

pub mod visual {
    /// Vtable `FrameWrapper`.
    pub const FRAME_WRAPPER: usize = 0x18D_3D58;

    /// Vtable `C_StaticParticle`.
    pub const STATIC_PARTICLE: usize = 0x18E_9A38;

    /// Vtable `LightEntity`.
    pub const LIGHT_ENTITY: usize = 0x18E_84F0;
}

pub mod special {
    /// Vtable `Telephone` (entity type `0x5F`).
    pub const TELEPHONE: usize = 0x185_29D0;

    /// Vtable `TelephoneReg` (outer registration entity type `0x20`).
    ///
    /// Runtime:
    /// - count = `1`
    /// - outer size = `0x1A0`
    ///
    /// Constructor:
    /// - `M2DE_CTelephoneReg_Construct` (`0x140C0E9F0`)
    ///
    /// ВАЖНО:
    /// это primary outer vtable.
    /// Embedded script child использует:
    /// - `script_entity::TELEPHONE_REG_CHILD`
    pub const TELEPHONE_REG: usize = 0x18D_0D80;

    /// Vtable `C_CutsceneEnt`.
    pub const CUTSCENE_ENT: usize = 0x185_19C8;
}

pub mod script_entity {
    /// Базовая vtable `C_ScriptEntity`.
    ///
    /// Constructor:
    /// - `M2DE_CScriptEntity_Construct` (`0x14039BDE0`)
    ///
    /// Runtime:
    /// - factory type `0x62`
    /// - base alloc size `0x90`
    /// - family total count = `200`
    pub const BASE: usize = 0x186_E170;

    /// Child script-object внутри `C_DamageZone`.
    ///
    /// Это НЕ primary vtable `C_DamageZone`.
    /// Child alloc size = `0xA0`.
    pub const DAMAGE_ZONE_CHILD: usize = 0x18D_05D8;

    /// Helper-object vtable внутри `C_DamageZone`.
    ///
    /// Alloc size = `0x28`
    /// Helper ptr сохраняется в `outer + 0xA0`.
    pub const DAMAGE_ZONE_HELPER_OBJECT: usize = 0x18D_0BD8;

    /// Embedded child script-object внутри `C_TelephoneReg`.
    ///
    /// Child base начинается по `outer + 0x100`.
    pub const TELEPHONE_REG_CHILD: usize = 0x18D_0C58;

    /// Helper-object vtable внутри `C_TelephoneReg`.
    ///
    /// Alloc size = `0x28`
    /// Helper ptr сохраняется в `outer + 0x198`.
    pub const TELEPHONE_REG_HELPER_OBJECT: usize = 0x18D_0EE0;

    /// Script child / host path, связанный с `/scripts/common/phonecalls.lua`.
    ///
    /// Child alloc size = `0x90`.
    /// Child сохраняется в `owner + 0x08`.
    pub const PHONECALLS_CHILD: usize = 0x18E_AFF8;

    /// Direct police-script child path.
    ///
    /// Runtime/IDA:
    /// - create-instance alloc size = `0x90`
    /// - direct ctor path confirmed
    /// - observed Lua behavior:
    ///   - `AddPoliceman(self, guid_a, guid_b, number, vec3)`
    ///   - `RemovePoliceman(self, guid)`
    ///
    /// Formal engine class name пока не закреплено на 100%,
    /// поэтому это всё ещё reverse-name path.
    pub const SUB5: usize = 0x184_B230;
}

pub mod car_manager {
    /// Основная vtable car manager path.
    pub const MAIN: usize = 0x186_F708;

    /// Альтернативная vtable car manager path.
    pub const ALT: usize = 0x186_F778;
}

pub mod garage {
    /// Vtable `C_GarageManager`.
    pub const GARAGE_MANAGER: usize = 0x190_CD38;

    /// Vtable family `C_Garage`.
    pub const GARAGE_1: usize = 0x190_CD78;
    pub const GARAGE_2: usize = 0x190_CF28;
    pub const GARAGE_3: usize = 0x190_CF78;
    pub const GARAGE_4: usize = 0x190_CFC8;
    pub const GARAGE_5: usize = 0x190_D2C0;

    /// Vtable `VehicleWrapper`.
    pub const VEHICLE_WRAPPER: usize = 0x190_D480;
}

pub mod callbacks {
    /// Vtable `GameCallbackManager`.
    pub const GAME_CALLBACK_MANAGER: usize = 0x186_D208;
}

pub mod misc {
    /// Vtable `CrashObjMgr`.
    pub const CRASH_OBJ_MGR: usize = 0x18E_9198;
}

pub mod render_device {
    /// Самая базовая abstract render-device vtable.
    pub const ABSTRACT: usize = 0x189_AF80;

    /// Промежуточная vtable в ctor-цепочке render device.
    pub const MID: usize = 0x18A_2F80;

    /// Базовая vtable `C_RenderDevice`.
    pub const BASE: usize = 0x18A_3310;

    /// Финальная vtable `M2DE_C_RenderDeviceD3D11`.
    pub const D3D11: usize = 0x18A_38C0;

    /// Временная vtable state-tracker object.
    pub const STATE_TRACKER_INITIAL: usize = 0x18A_2870;

    /// Финальная vtable state-tracker object.
    pub const STATE_TRACKER_FINAL: usize = 0x18A_2960;
}
