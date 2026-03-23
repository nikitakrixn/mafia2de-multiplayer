//! Структуры системы сущностей — базовые классы и инфраструктура.

use std::ffi::c_void;

use crate::macros::assert_field_offsets;

// =============================================================================
//  C_Entity — корень иерархии всех сущностей (0x78 байт)
// =============================================================================

/// Базовый класс для всех сущностей движка.
///
/// Размер: **0x78 байт**. Все остальные типы сущностей наследуются от него.
///
/// ## Структура vtable
///
/// Таблица виртуальных методов содержит две логические секции:
/// - Слоты [3-16]  — управление жизненным циклом (активация, сообщения)
/// - Слоты [32-48] — пространственный интерфейс (позиция, поворот, масштаб)
///
/// В базовом `C_Entity` пространственные методы являются заглушками.
/// `C_Actor` переопределяет их реальными реализациями через `frame_node`.
///
/// ## Инициализация
///
/// При создании обнуляются поля +0x08..+0x70, аллоцируются два
/// RB-дерева (+0x40, +0x50), генерируется уникальный `table_id`.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntity {
    /// Указатель на таблицу виртуальных методов.
    pub vtable: *const c_void, // +0x00

    /// Расширенные указатели на дополнительные подсистемы.
    /// Ненулевые только у Player и C_CarVehicle.
    pub ext_ptr_1: usize, // +0x08
    pub ext_ptr_2: usize, // +0x10
    pub ext_ptr_3: usize, // +0x18

    /// Байт состояния сущности.
    pub state_flags: u8, // +0x20
    pub _gap_21: [u8; 3], // +0x21..+0x23

    /// Упакованный идентификатор: `(instance_id << 8) | factory_type`.
    ///
    /// Младший байт — тип сущности:
    /// - `0x0E` = HumanNPC
    /// - `0x10` = Player
    /// - `0x12` = Car (статичная)
    /// - `0x70` = CarVehicle (управляемая)
    ///
    /// Старшие 24 бита — уникальный номер экземпляра.
    pub table_id: u32, // +0x24

    /// Флаги сущности (битовое поле).
    ///
    /// - bit 5 (0x20):    активирована
    /// - bit 17 (0x20000): streaming state 1
    /// - bit 18 (0x40000): streaming state 2
    pub entity_flags: u32, // +0x28
    pub _gap_2c: u32, // +0x2C

    /// FNV-1 64-bit хеш имени сущности. Ноль для безымянных.
    pub name_hash: u64, // +0x30

    /// Ссылка на родительский контейнер. NULL у автономных сущностей.
    pub parent_ref: usize, // +0x38

    /// Корень первого RB-дерева (иерархия сущностей).
    /// Sentinel самоссылается: left=self, right=self, parent=self.
    pub tree_1_root: usize, // +0x40

    /// Количество записей в первом дереве.
    /// Обычно 0, у Player равно 2.
    pub tree_1_count: usize, // +0x48

    /// Корень второго RB-дерева (подписки на сообщения).
    pub tree_2_root: usize, // +0x50

    /// Зарезервировано, всегда 0.
    pub _zero_58: usize, // +0x58

    /// Очередь входящих сообщений (`std::vector<EntityMessage*>`).
    ///
    /// Стандартный layout вектора: begin / end / capacity.
    /// В большинстве случаев пуста — сообщения обрабатываются
    /// и сбрасываются каждый кадр.
    pub pending_msg_begin: usize, // +0x60
    pub pending_msg_end: usize,   // +0x68
    pub pending_msg_cap: usize,   // +0x70
    // Итого: 0x78 байт. Поля C_Actor начинаются с +0x78.
}

assert_field_offsets!(CEntity {
    vtable            == 0x00,
    state_flags       == 0x20,
    table_id          == 0x24,
    entity_flags      == 0x28,
    name_hash         == 0x30,
    parent_ref        == 0x38,
    tree_1_root       == 0x40,
    tree_2_root       == 0x50,
    pending_msg_begin == 0x60,
    pending_msg_end   == 0x68,
    pending_msg_cap   == 0x70,
});

impl CEntity {
    /// Тип сущности — младший байт `table_id`.
    pub fn factory_type(&self) -> u8 {
        (self.table_id & 0xFF) as u8
    }

    /// Уникальный номер экземпляра — старшие 24 бита `table_id`.
    pub fn instance_index(&self) -> u32 {
        self.table_id >> 8
    }

    /// Проверка флага активации (bit 5).
    pub fn is_activated(&self) -> bool {
        (self.entity_flags & 0x20) != 0
    }

    /// Проверка наличия streaming state.
    pub fn has_streaming_flag(&self) -> bool {
        (self.entity_flags & 0x60000) != 0
    }
}

// =============================================================================
//  C_Actor — расширяет C_Entity трансформацией и owner'ом
// =============================================================================

/// Поля слоя `C_Actor` — начинаются с +0x78 от начала сущности.
///
/// Actor добавляет к базовой сущности:
/// - `frame_node` (+0x78) — узел трансформации (позиция, поворот в мире)
/// - `owner` (+0x80) — NULL = пешком, ненулевой = внутри транспорта
/// - `component_88/90/98` — расширенные компоненты (у C_Car, C_CarVehicle)
/// - `entity_subtype` (+0xA0) — подтип: Player=6, CarVehicle=3
///
/// Позиция читается из frame_node:
/// ```text
/// frame + 0x64 = X (float)
/// frame + 0x74 = Y (float)
/// frame + 0x84 = Z (float)
/// ```
#[repr(C)]
#[allow(non_snake_case)]
pub struct CActorFields {
    /// Узел трансформации в мировом пространстве.
    ///
    /// Позиция: `frame+0x64` (X), `frame+0x74` (Y), `frame+0x84` (Z).
    /// Направление: `frame+0x5C/0x6C/0x7C` (forward vector).
    pub frame_node: *mut c_void, // +0x78

    /// Владелец/контейнер. NULL = пешком, ненулевой = в транспорте.
    pub owner: *mut c_void, // +0x80

    /// Расширенные компоненты Actor. NULL у гуманоидов, ненулевые у транспорта.
    pub component_88: usize, // +0x88
    pub component_90: usize, // +0x90
    pub component_98: usize, // +0x98

    /// Подтип сущности. Player=6, CarVehicle=3, Car=varies.
    pub entity_subtype: u32, // +0xA0
    pub _pad_a4: u32, // +0xA4
}

// =============================================================================
//  C_EntityGuid — уникальный идентификатор сущности
// =============================================================================

/// GUID сущности для скриптовой системы.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CEntityGuid {
    pub guid: u32,
}

// =============================================================================
//  Запись в EntityDatabase
// =============================================================================

/// Запись сущности в глобальной базе данных.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntityDBRecord {
    pub _unk_00: [u8; 0x24],
    /// Упакованный ID: `(instance_index << 8) | factory_type_byte`.
    pub table_id: u32, // +0x24
    /// Флаги. Bit 5 (0x20) = есть script wrapper.
    pub flags: u32, // +0x28
    pub _unk_2c: u32, // +0x2C
    /// FNV-1 64-bit хеш имени сущности.
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

// =============================================================================
//  Script Wrapper — скриптовый handle на нативную сущность
// =============================================================================

/// Скриптовый wrapper — handle для доступа к нативной сущности из Lua.
///
/// `wrapper+0x10` — нативный указатель на сущность.
/// `wrapper+0x18` — observer-объект, кеширующий состояние.
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptWrapper {
    pub vtable: *const c_void, // +0x00
    pub refcount: i32,         // +0x08
    pub _pad_0c: i32,          // +0x0C
    /// Нативный указатель на сущность (C_Human*, C_Car* и т.д.).
    pub native_entity: *mut c_void, // +0x10
    /// Observer-объект (кеширует состояние сущности).
    pub observer: *mut c_void, // +0x18
}

/// Менеджер script wrapper'ов — двойной сортированный кеш.
///
/// Кеш по хешу (16 байт/запись): `{ u64 fnv1_hash, *mut CScriptWrapper }`
/// Кеш по table_id (16 байт/запись): `{ u32 table_id, u32 pad, *mut CScriptWrapper }`
#[repr(C)]
pub struct CScriptWrapperManager {
    pub vtable: *const c_void,        // +0x00
    pub hash_cache_begin: *mut u8,    // +0x08
    pub hash_cache_end: *mut u8,      // +0x10
    pub hash_cache_sentinel: *mut u8, // +0x18
    pub _unk_20: *mut c_void,         // +0x20
    pub id_cache_begin: *mut u8,      // +0x28
    pub id_cache_end: *mut u8,        // +0x30
    pub id_cache_capacity: *mut u8,   // +0x38
}

/// Фабрика wrapper'ов — создаёт типизированный CScriptWrapper по типу сущности.
#[repr(C)]
pub struct CWrapperFactory {
    pub vtable: *const c_void,    // +0x00
    pub type_id_ptr: *const u32,  // +0x08
    pub create_fn: *const c_void, // +0x10
}

// =============================================================================
//  Service Identity — регистрация модуля
// =============================================================================

/// Идентификатор сервиса в Service Locator.
///
/// Хеш: FNV-1 32-bit (seed=0x811C9DC5, prime=0x01000193).
#[repr(C)]
pub struct CServiceIdentity {
    pub vtable: *const c_void, // +0x00
    pub name_hash: u32,        // +0x08
    pub module_id: u32,        // +0x0C
}

// =============================================================================
//  TypeRegistry — создание нативных сущностей из SDS
// =============================================================================

/// Дескриптор типа для создания сущности из SDS-ресурсов.
///
/// Хеш имени использует FNV-1 64-bit с seed=0:
/// ```c
/// for (i = 0LL; *name; ) {
///     i = byte ^ (0x100000001B3LL * i);
/// }
/// ```
#[repr(C)]
#[allow(non_snake_case)]
pub struct CTypeDescriptor {
    pub next: *mut CTypeDescriptor, // +0x00
    pub type_id: u32,               // +0x08
    pub _pad_0c: u32,               // +0x0C
    pub name_hash: u64,             // +0x10
    pub create_fn: *const c_void,   // +0x18
    pub parse_fn: *const c_void,    // +0x20
    pub aligned_size: u32,          // +0x28
    pub _pad_2c: u32,               // +0x2C
}
