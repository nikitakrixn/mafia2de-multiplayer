//! Базовые структуры системы сущностей — C_Entity, C_EntityPos, C_Actor.
//!
//! ```text
//! C_Entity (0x78 байт, VT: 0x14186CAC8)
//!   │
//!   ├─ C_EntityPos (нет новых полей, добавляет spatial pure virtuals 32-39)
//!   │    │
//!   │    ├─ C_Actor (0xA8 байт, VT: 0x14186D050)
//!   │    │    │
//!   │    │    ├─ I_Human2 (abstract, VT: 0x1418E2BD8)
//!   │    │    │    └─ C_Human2 / CHumanNPC (VT: 0x1418E5188, ft=0x0E)
//!   │    │    │         └─ C_Player2 / CPlayer (VT: 0x14184C060, ft=0x10)
//!   │    │    │
//!   │    │    ├─ C_ActorVehicle (VT: 0x1418CFE50, +seats/enter-leave)
//!   │    │    │    └─ C_Car (VT: 0x141850030, ft=0x12)
//!   │    │    │         + ue::game::vehicle::C_Vehicle (MI at +0xE0)
//!   │    │    │         + C_PhThingDeform (MI через C_Vehicle)
//!   │    │    │
//!   │    │    ├─ C_Tree, C_Item, C_Blocker, C_Wardrobe
//!   │    │    ├─ C_LightEntity, C_Pinup, C_DamageZone
//!   │    │    ├─ C_CleanEntity, C_StaticEntity, C_FrameWrapper
//!   │    │    └─ C_ActorDetector
//!   │    │
//!   │    └─ C_ActorDeform (VT: 0x14186D410, + C_PhThingDeform MI at +0xA8)
//!   │         ├─ C_CrashObject (ft=0x14)
//!   │         ├─ C_Door (ft=0x26)
//!   │         └─ C_Boat, C_Lift, C_Jukebox, C_Radio
//!   │
//!   ├─ C_EntityDummy
//!   │
//!   ├─ ue::C_ScriptEntity (ft=0x62, НЕ Actor!)
//!   │
//!   ├─ C_Sound (ft=0x29, НЕ Actor!)
//!   │
//!   └─ C_CarVehicle (ft=0x70, VT: 0x1418EAAC8)
//! ```

use crate::macros::assert_field_offsets;
use std::ffi::c_void;

// =============================================================================
//  C_Entity — корень иерархии (0x78 байт)
// =============================================================================

/// Базовый класс для всех сущностей движка.
///
/// **Размер: 0x78 байт** (подтверждено деструктором: `GlobalFree(this, 0x78)`).
///
/// ## Конструктор (`M2DE_CEntity_Construct`, 0x14039B710)
///
/// 1. Устанавливает vtable `C_Entity`
/// 2. Зануляет все поля
/// 3. Создаёт два RB-tree sentinel узла (подписки на сообщения)
/// 4. Генерирует `table_id = (instance_id << 8) | factory_type`
/// 5. Регистрирует сущность в `WorldEntityManager`
///
/// ## Vtable слоты
///
/// | Слот | Метод | Источник |
/// |:----:|:------|:---------|
/// | 0 | `~C_Entity()` | ScalarDeletingDestructor |
/// | 2 | `GetFrameNode()` | `return *(this+0x78)` |
/// | 3 | `Init(S_EntityInitProps*)` | Инициализация из SDS |
/// | 4 | `GameInit()` | Старт уровня |
/// | 5 | `GameDone()` | Конец уровня |
/// | 6 | `GameRestore()` | Восстановление после загрузки |
/// | 7 | `OnActivate()` | Активация в мире |
/// | 8 | `OnDeactivate()` | Деактивация |
/// | 9 | `IsActive()` | Проверка флага |
/// | 13 | `GameSaveDependencies()` | Сериализация RB-tree подписок |
/// | 17 | `EntityInvalidate()` | Инвалидация связей |
/// | 18 | `AddOnEvent()` | Создание 32B сообщения |
/// | 22 | `RecvMessage()` | Приём сообщений |
/// | 23 | `InvalidateRelation()` | Удаление подписок |
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntity {
    /// Указатель на виртуальную таблицу.
    pub vtable: *const c_void, // +0x00

    /// Расширения — heap-указатели.
    /// NULL у большинства типов. Ненулевые у Player и CarVehicle.
    pub ext_ptr_08: *mut c_void, // +0x08
    /// Расширение (см. `ext_ptr_08`).
    pub ext_ptr_10: *mut c_void, // +0x10
    /// Расширение (см. `ext_ptr_08`).
    pub ext_ptr_18: *mut c_void, // +0x18

    /// Флаг реентерабельности для итерации observer-списка.
    /// C_EntityPos записывает сюда `3` при конструировании.
    pub observer_guard: u8, // +0x20
    pub _pad_21: [u8; 3], // +0x21..+0x23

    /// Packed ID: `(instance_id << 8) | factory_type`.
    pub table_id: u32, // +0x24

    /// Флаги сущности.
    /// - bit 5 (0x20): активирована (`OnActivate` ORs это)
    /// - bit 7 (0x80): C_ActorDeform устанавливает в конструкторе
    /// - bits 17/18 (0x60000): стриминг
    pub entity_flags: u32, // +0x28
    pub _pad_2c: u32, // +0x2C

    /// FNV-1 64-bit хеш имени (0 для безымянных).
    pub name_hash: u64, // +0x30

    /// Ссылка на родительскую сущность.
    /// Записывается в vtable[3] `Init(S_EntityInitProps*)`.
    pub parent_ref: *mut c_void, // +0x38

    /// Корень RB-tree подписок на сообщения (sentinel, alloc 0x38).
    /// Используется в `GameSaveDependencies`[13], `InvalidateRelation`[23].
    pub message_tree_root: *mut c_void, // +0x40

    /// Количество записей в дереве сообщений.
    /// Player = 2, большинство = 0.
    pub message_tree_count: usize, // +0x48

    /// Корень RB-tree обратных ссылок (sentinel, alloc 0x30).
    pub reverse_ref_tree_root: *mut c_void, // +0x50

    pub _zero_58: usize, // +0x58

    /// Вектор observer/callback записей.
    pub observer_begin: *mut c_void, // +0x60
    /// Конец вектора observer-записей.
    pub observer_end: *mut c_void, // +0x68
    /// Конец выделенной памяти вектора.
    pub observer_cap: *mut c_void, // +0x70
}

assert_field_offsets!(CEntity {
    vtable               == 0x00,
    ext_ptr_08           == 0x08,
    ext_ptr_10           == 0x10,
    ext_ptr_18           == 0x18,
    observer_guard       == 0x20,
    table_id             == 0x24,
    entity_flags         == 0x28,
    name_hash            == 0x30,
    parent_ref           == 0x38,
    message_tree_root    == 0x40,
    message_tree_count   == 0x48,
    reverse_ref_tree_root == 0x50,
    _zero_58             == 0x58,
    observer_begin       == 0x60,
    observer_end         == 0x68,
    observer_cap         == 0x70,
});

const _: () = {
    assert!(std::mem::size_of::<CEntity>() == 0x78);
};

impl CEntity {
    /// Тип фабрики из младшего байта `table_id`.
    #[inline]
    pub fn factory_type(&self) -> u8 {
        (self.table_id & 0xFF) as u8
    }

    /// Индекс экземпляра из старших 24 бит `table_id`.
    #[inline]
    pub fn instance_index(&self) -> u32 {
        self.table_id >> 8
    }

    /// Проверяет флаг активации (bit 5, устанавливается в `OnActivate`).
    #[inline]
    pub fn is_activated(&self) -> bool {
        (self.entity_flags & 0x20) != 0
    }

    /// Проверяет флаги стриминга (bits 17/18).
    #[inline]
    pub fn has_streaming_bits(&self) -> bool {
        (self.entity_flags & 0x60000) != 0
    }
}

// =============================================================================
//  C_Actor — пространственное представление (0xA8 байт)
// =============================================================================

/// Слой Actor — добавляет к C_Entity пространственное представление.
///
/// **Размер: 0xA8 байт**.
///
/// ## Иерархия
///
/// ```text
/// C_Entity -> C_EntityPos (pure virtuals) -> C_Actor (реализация)
/// ```
///
/// C_EntityPos не добавляет полей — только объявляет SetPos/GetPos/SetDir
/// и т.д. как pure virtual. C_Actor реализует их через `frame_node`.
///
/// ## Конструктор (`M2DE_CActor_Construct`, 0x14039A7E0)
///
/// 1. Вызывает `CEntity_Construct()`
/// 2. Устанавливает vtable `C_Actor`
/// 3. Зануляет `frame_node`, `owner`, компонентные поля
/// 4. Устанавливает `factory_type = 5` (промежуточный тип Actor)
///
/// ## Vtable слоты поверх C_Entity
///
/// | Слот | Метод | Описание |
/// |:----:|:------|:---------|
/// | 4 | `GameInit()` | C_Actor override |
/// | 5 | `GameDone()` | C_Actor override — высокоуровневая деактивация |
/// | 7 | `OnActivate()` | C_Actor override |
/// | 8 | `OnDeactivate()` | C_Actor override |
/// | 31 | `Update(float)` | Базовый тик актора |
/// | 32 | `SetPos(C_Vector&)` | Запись в frame_node |
/// | 33 | `SetDir(C_Vector&)` | Направление через frame |
/// | 34 | `SetRot(C_Quat&)` | Кватернион через frame |
/// | 35 | `SetScale(float)` | Масштабирование 3x3 матрицы |
/// | 36–39 | `GetPos/Dir/Rot/Scale` | Чтение из frame_node |
/// | 41 | `GameSavePRS()` | Сериализация позиции/вращения/масштаба |
/// | 42 | `GameLoadPRS()` | Десериализация, вызывает SetPos+SetRot |
/// | 43 | `GetCameraPoint()` | Базовая реализация |
/// | 44 | `SetFrame(C_Frame*)` | Замена frame_node (+0x78) |
/// | 47 | `IsDead()` | Базовый — всегда false |
#[repr(C)]
#[allow(non_snake_case)]
pub struct CActor {
    /// Базовая сущность.
    pub base: CEntity, // +0x00..+0x77

    /// Узел трансформации в сцене (`ue::sys::core::C_Frame*`).
    ///
    /// Содержит матрицу 4×3 трансформации. Позиция: frame+0x64/0x74/0x84.
    /// Заменяется через vtable[44] `SetFrame(C_Frame*)`.
    pub frame_node: *mut c_void, // +0x78

    /// Владелец / контейнер / транспорт.
    ///
    /// - `NULL` = пешком
    /// - `vehicle_ptr` = сидит в машине
    ///
    /// Записывается через vtable[45] `SetOwner(C_Entity*)`.
    /// При записи broadcast'ит EnterVehicle msg 0xD0004.
    pub owner: *mut c_void, // +0x80

    /// Компонентные указатели (зависят от типа наследника).
    ///
    /// - Чистый C_Actor: все NULL
    /// - C_Car / C_CarVehicle: heap-указатели
    /// - I_Human2: часть компонентного блока
    pub component_88: *mut c_void, // +0x88
    /// Компонентный указатель.
    pub component_90: *mut c_void, // +0x90
    /// Компонентный указатель.
    pub component_98: *mut c_void, // +0x98

    /// Подтип сущности. Устанавливается после конструирования.
    ///
    /// | Значение | Тип |
    /// |:--------:|:----|
    /// | 6 | Player |
    /// | 3 | CarVehicle |
    /// | 0x36–0x3A | C_Car (различные кузова) |
    pub entity_subtype: u32, // +0xA0
    pub _pad_a4: u32, // +0xA4
}

assert_field_offsets!(CActor {
    base           == 0x00,
    frame_node     == 0x78,
    owner          == 0x80,
    component_88   == 0x88,
    component_90   == 0x90,
    component_98   == 0x98,
    entity_subtype == 0xA0,
});

const _: () = {
    assert!(std::mem::size_of::<CActor>() == 0xA8);
};

// =============================================================================
//  Вспомогательные структуры
// =============================================================================

/// Уникальный идентификатор сущности.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CEntityGuid {
    pub guid: u32,
}

/// Запись в EntityDatabase.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntityDBRecord {
    pub _unk_00: [u8; 0x24],
    pub table_id: u32,  // +0x24
    pub flags: u32,     // +0x28
    pub _unk_2c: u32,   // +0x2C
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

/// Обёртка скриптовой системы над нативной сущностью.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptWrapper {
    pub vtable: *const c_void,
    pub refcount: i32,
    pub _pad_0c: i32,
    /// Указатель на нативную сущность (подтверждено runtime).
    pub native_entity: *mut c_void, // +0x10
    /// Объект-наблюдатель.
    pub observer: *mut c_void, // +0x18
}

/// Менеджер скриптовых обёрток (кеш по хешу и по ID).
#[repr(C)]
pub struct CScriptWrapperManager {
    pub vtable: *const c_void,
    pub hash_cache_begin: *mut u8,
    pub hash_cache_end: *mut u8,
    pub hash_cache_sentinel: *mut u8,
    pub _unk_20: *mut c_void,
    pub id_cache_begin: *mut u8,
    pub id_cache_end: *mut u8,
    pub id_cache_capacity: *mut u8,
}

/// Фабрика обёрток (запись в RB-tree по factory_type).
#[repr(C)]
pub struct CWrapperFactory {
    pub vtable: *const c_void,
    pub type_id_ptr: *const u32,
    pub create_fn: *const c_void,
}

/// Идентификатор сервиса/модуля.
#[repr(C)]
pub struct CServiceIdentity {
    pub vtable: *const c_void,
    pub name_hash: u32,
    pub module_id: u32,
}

/// Дескриптор типа в TypeRegistry (linked list).
#[repr(C)]
#[allow(non_snake_case)]
pub struct CTypeDescriptor {
    pub next: *mut CTypeDescriptor,
    pub type_id: u32,
    pub _pad_0c: u32,
    pub name_hash: u64,
    pub create_fn: *const c_void,
    pub parse_fn: *const c_void,
    pub aligned_size: u32,
    pub _pad_2c: u32,
}