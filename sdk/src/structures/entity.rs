//! Структуры системы сущностей — базовые классы и инфраструктура.
//!
//! Источники подтверждения:
//! - M2DE_BaseEntity_Construct (0x14039B710) — конструктор, инициализация полей
//! - Деструктор vtable[0] (0x14039F310) — `GlobalFree(this, 0x78)` = размер 0x78
//! - M2DE_ActorEntity_Construct (0x14039A7E0) — Actor overlay
//! - M2DE_CHuman_BaseConstructor (0x140D730B0) — компоненты Human
//! - M2DE_Entity_SetTypeID (0x1403B99F0) — packed table_id
//! - Runtime: сканирование 2415 entity в FreeRide
//!
//! КЛЮЧЕВОЕ ОТКРЫТИЕ — двойной интерфейс в vtable:
//!   Слоты [3-16]  = управление entity (activate/deactivate/messages)
//!   Слоты [32-48] = пространственный интерфейс (SetPos/GetPos/SetRot...)
//!   В C_Entity обе секции одинаковы (stubs).
//!   В C_Actor вторая секция работает через frame_node (+0x78).

use std::ffi::c_void;

use crate::macros::assert_field_offsets;

// =============================================================================
//  C_Entity — корень иерархии всех сущностей (0x78 байт)
// =============================================================================

/// `C_Entity` — базовый класс для ВСЕХ сущностей Mafia II: DE.
///
/// Размер: **0x78 байт** (120 decimal). Подтверждено деструктором:
/// ```asm
/// mov edx, 78h     ; размер для GlobalFree
/// call M2DE_GlobalFree
/// ```
///
/// Vtable: `M2DE_VT_CEntity` (0x14186CAC8), ~110 виртуальных методов.
/// Конструктор: `M2DE_BaseEntity_Construct` (0x14039B710).
///
/// ## Двойной интерфейс в vtable
///
/// Vtable содержит **дублированную секцию**:
/// - Слоты [3-16] = первичный интерфейс управления entity
/// - Слоты [32-48] = пространственный/трансформный интерфейс
///
/// В базовом `C_Entity` обе секции содержат одинаковые stubs.
/// `C_Actor` переопределяет вторую секцию реальными реализациями
/// через `frame_node` (+0x78).
///
/// ## Ключевые виртуальные методы (подтверждённые)
///
/// | Слот | Имя | Уверенность |
/// |------|------|-------------|
/// | [0] | ScalarDeletingDestructor | 100% |
/// | [2] | GetFrameNode | 95% (Actor) |
/// | [3] | SetParentRef | 85% |
/// | [4] | Activate | 90% |
/// | [5] | Deactivate | 90% |
/// | [13] | ProcessMessage_Internal | 80% |
/// | [16] | LoadFromStream | 85% |
/// | [22] | HandleMessage | 95% |
/// | [23] | UnregisterMessages | 85% |
/// | [32] | SetPos | 100% |
/// | [36] | GetPos | 100% |
/// | [44] | SetFrameNode | 90% |
/// | [47] | IsDead | 100% |
/// | [50] | Update | 95% |
/// | [82] | ApplyDamage | 95% |
///
/// ## Конструктор инициализирует
///
/// ```text
/// +0x00 = vtable M2DE_VT_CEntity
/// +0x08..+0x18 = NULL (три qword'а)
/// +0x20 = 0 (state_flags byte)
/// +0x24 = 0 → потом packed table_id
/// +0x28 = entity_flags (биты streaming)
/// +0x30 = 0 (name_hash)
/// +0x38 = 0 (parent_ref)
/// +0x40 = alloc(0x38) → RB-tree sentinel 1 (hierarchy)
/// +0x48 = 0
/// +0x50 = alloc(0x30) → RB-tree sentinel 2 (subscriptions)
/// +0x58..+0x70 = 0
/// ```
///
/// RB-tree sentinel инициализируется как self-linked:
/// ```asm
/// mov [rax], rax       ; left = self
/// mov [rax+8], rax     ; right = self
/// mov [rax+10h], rax   ; parent = self
/// mov word ptr [rax+18h], 101h  ; чёрный + is_sentinel
/// ```
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntity {
    /// Указатель на таблицу виртуальных методов.
    pub vtable: *const c_void, // +0x00

    /// Player-only: три указателя на heap-объекты (шаг ~0x20/0x40).
    /// NULL для всех остальных типов entity.
    /// Устанавливается CPlayerEntity_Constructor.
    /// Runtime подтверждено: только Player имеет эти поля != NULL.
    pub player_data_08: usize, // +0x08
    pub player_data_10: usize, // +0x10
    pub player_data_18: usize, // +0x18

    /// Байт состояния (обнулён в конструкторе).
    pub state_flags: u8, // +0x20
    /// НЕ инициализируется конструктором — содержит мусор от аллокатора.
    /// Runtime: Sound имеет ASCII "urce" (от "resource").
    pub _gap_21: [u8; 3], // +0x21..+0x23

    /// Упакованный идентификатор: `(instance_id << 8) | factory_type`.
    ///
    /// Младший байт = factory type:
    /// - `0x0E` = HumanNPC
    /// - `0x10` = Player
    /// - `0x12` = Car (статичная)
    /// - `0x70` = CarVehicle (управляемая)
    /// - и т.д.
    ///
    /// Старшие 24 бита = instance_id (глобальный счётчик).
    pub table_id: u32, // +0x24

    /// Флаги entity (битовое поле).
    ///
    /// - bit 5 (0x20): активирована
    /// - bit 17 (0x20000): streaming state 1
    /// - bit 18 (0x40000): streaming state 2
    pub entity_flags: u32, // +0x28
    /// НЕ инициализируется конструктором — содержит мусор от аллокатора.
    pub _gap_2c: u32, // +0x2C

    /// FNV-1 64-bit хеш имени entity (0 для безымянных entity).
    /// Runtime: HumanNPC, LightEntity, ScriptEntity имеют name_hash=0.
    pub name_hash: u64, // +0x30

    /// Ссылка на parent/container.
    ///
    /// Устанавливается через vtable[3] (`SetParentRef`),
    /// при этом уведомляется WorldEntityManager.
    /// Runtime: NULL для Sound, ScriptEntity.
    pub parent_ref: usize, // +0x38

    /// RB-дерево 1 — корень sentinel'а (иерархия entity).
    ///
    /// Аллоцируется 0x38 байт. Sentinel самоссылается:
    /// `[0]=self, [8]=self, [10h]=self, word[18h]=0x0101`.
    pub tree_1_root: usize, // +0x40

    /// Количество записей в дереве 1.
    /// Runtime: 0 для большинства entity, 2 для Player.
    pub tree_1_count: usize, // +0x48

    /// RB-дерево 2 — корень sentinel'а (подписки на сообщения).
    ///
    /// Аллоцируется 0x30 байт. Тот же self-linked паттерн.
    pub tree_2_root: usize, // +0x50

    /// Всегда 0 в runtime (подтверждено дампом всех типов).
    pub _zero_58: usize, // +0x58
    pub _zero_60: usize, // +0x60
    pub _zero_68: usize, // +0x68
    pub _zero_70: usize, // +0x70
    // Итого: 0x78 байт. Поля C_Actor начинаются с +0x78.
}

assert_field_offsets!(CEntity {
    vtable       == 0x00,
    state_flags  == 0x20,
    table_id     == 0x24,
    entity_flags == 0x28,
    name_hash    == 0x30,
    parent_ref   == 0x38,
    tree_1_root  == 0x40,
    tree_2_root  == 0x50,
    _zero_68     == 0x68,
    _zero_70     == 0x70,
});

impl CEntity {
    /// Factory type byte — младший байт packed table_id.
    pub fn factory_type(&self) -> u8 {
        (self.table_id & 0xFF) as u8
    }

    /// Instance ID — старшие 24 бита packed table_id.
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

/// Поля `C_Actor` — начинаются с +0x78 от начала entity.
///
/// Конструктор: `M2DE_ActorEntity_Construct` (0x14039A7E0).
/// Vtable: `M2DE_VT_CActor` (0x14186D050).
///
/// Actor добавляет:
/// - `frame_node` (+0x78) — указатель на трансформ/позицию в мире
/// - `owner` (+0x80) — NULL = на ногах, vehicle* = в машине
///
/// Позиция читается из frame_node:
/// ```text
/// frame + 0x64 = X (float)
/// frame + 0x74 = Y (float)
/// frame + 0x84 = Z (float)
/// ```
///
/// Ключевые vtable методы Actor (вторая секция, слоты 32-48):
/// - [32] SetPos — `M2DE_CActor_SetPos_ViaFrame`
/// - [33] SetRotation — через frame_node
/// - [34] SetScale — через frame_node
/// - [35] SetDir — через frame_node
/// - [36] GetPos — `M2DE_CActor_GetPos_ViaFrame`
/// - [39] GetBoundRadius — `frame+0x68` (float)
/// - [44] SetFrameNode — простая замена указателя +0x78
#[repr(C)]
#[allow(non_snake_case)]
pub struct CActorFields {
    /// Указатель на frame/transform node.
    ///
    /// Позиция: `frame+0x64` (X), `frame+0x74` (Y), `frame+0x84` (Z).
    /// Направление: `frame+0x34/0x44/0x54` (forward vector).
    pub frame_node: *mut c_void, // +0x78

    /// Владелец/контейнер. NULL = на ногах, vehicle* = в транспорте.
    pub owner: *mut c_void, // +0x80

    /// Неизвестно (обнулено в конструкторе).
    pub _unk_88: u64, // +0x88

    /// Неизвестно (обнулено). Actor::OnStateUpdate читает +0x90.
    pub _unk_90: u64, // +0x90

    /// Неизвестно (обнулено).
    pub _unk_98: u64, // +0x98

    /// Подтип entity (устанавливается после конструирования).
    pub entity_subtype: u32, // +0xA0
    pub _pad_a4: u32, // +0xA4
}

// =============================================================================
//  C_EntityGuid — уникальный идентификатор сущности
// =============================================================================

/// GUID сущности для Lua-скриптов.
///
/// Lua: `C_EntityGuid`. Формат: `"%u"`.
/// Подтверждено из `M2DE_LuaW_WrappersList_GetEntityByGUID`:
/// ```c
/// M2DE_FormatString("C_EntityGuid: %u", *ThisObject);
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CEntityGuid {
    pub guid: u32,
}

// =============================================================================
//  Запись в EntityDatabase
// =============================================================================

/// Запись entity в глобальной БД (`M2DE_g_EntityDatabase`).
///
/// Подтверждено из:
/// - FindByName: `mov r9d, [rax+24h]` (table_id)
/// - CreateScriptWrapper: `movzx ebx, byte ptr [rdx+24h]` (factory type)
/// - GetOrCreateWrapper: `mov rcx, [rax+30h]` (name_hash)
/// - Runtime: factory_type совпадает для всех 2415 entity
#[repr(C)]
#[allow(non_snake_case)]
pub struct CEntityDBRecord {
    pub _unk_00: [u8; 0x24],
    /// Упакованный ID: `(instance_index << 8) | factory_type_byte`.
    pub table_id: u32, // +0x24
    /// Bit 5 (0x20) = has_script_wrapper / spawnable.
    pub flags: u32, // +0x28
    pub _unk_2c: u32, // +0x2C
    /// FNV-1 64-bit хеш имени entity.
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
//  Script Wrapper — Lua-handle на native entity
// =============================================================================

/// Script wrapper — Lua-доступный handle на нативную entity.
///
/// `wrapper+0x10` — это **нативный указатель** (runtime подтверждено).
/// Для Joe/Henry: корректно читает entity_type=0x0E, health.
///
/// Создание через `M2DE_EntityManager_CreateScriptWrapper`:
/// 1. `factory_type = db_record.table_id & 0xFF`
/// 2. `factory = WrapperFactoryMap[factory_type]`
/// 3. `wrapper = factory->Create()` через vtable[+0x10]
/// 4. `wrapper+0x10 = native entity ptr`
/// 5. `wrapper+0x18 = observer (264 байта из DB record)`
#[repr(C)]
#[allow(non_snake_case)]
pub struct CScriptWrapper {
    pub vtable: *const c_void, // +0x00
    pub refcount: i32,         // +0x08
    pub _pad_0c: i32,          // +0x0C
    /// Нативный указатель на entity (C_Human*, C_Car* и т.д.). Подтверждено runtime.
    pub native_entity: *mut c_void, // +0x10
    /// Observer-объект (кеширует состояние entity).
    pub observer: *mut c_void, // +0x18
}

/// Менеджер script wrapper'ов — двойной сортированный кеш для O(log n) поиска.
///
/// Глобал: `M2DE_g_ScriptWrapperManager` (0x1431360F8).
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

/// Фабрика wrapper'ов — создаёт типизированный CScriptWrapper.
///
/// 36 фабрик в `M2DE_g_WrapperFactoryMap` (RB-дерево).
/// Все имеют общую vtable `off_141918858`.
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
/// Используется 49 типами модулей (E_ModuleId 0-48).
/// Хеш: FNV-1 32-bit (seed=0x811C9DC5, prime=0x01000193).
#[repr(C)]
pub struct CServiceIdentity {
    pub vtable: *const c_void, // +0x00
    pub name_hash: u32,        // +0x08 (FNV-1 32-bit)
    pub module_id: u32,        // +0x0C (E_ModuleId)
}

// =============================================================================
//  TypeRegistry — создание нативных entity из SDS
// =============================================================================

/// Дескриптор типа для создания entity из SDS-ресурсов.
///
/// 48 типов зарегистрировано через `M2DE_TypeRegistry_RegisterDescriptor`.
/// Глобал: `M2DE_g_TypeRegistry` (0x141CAE228).
///
/// **ВАЖНО**: хеш имени использует FNV-1 64-bit с **seed=0** (не стандартный):
/// ```c
/// for (i = 0LL; *name; ) {       // seed = 0 !
///     i = byte ^ (0x100000001B3LL * i);
/// }
/// ```
#[repr(C)]
#[allow(non_snake_case)]
pub struct CTypeDescriptor {
    pub next: *mut CTypeDescriptor, // +0x00
    pub type_id: u32,               // +0x08
    pub _pad_0c: u32,               // +0x0C
    pub name_hash: u64,             // +0x10 (FNV-1 64-bit seed=0)
    pub create_fn: *const c_void,   // +0x18
    pub parse_fn: *const c_void,    // +0x20
    pub aligned_size: u32,          // +0x28
    pub _pad_2c: u32,               // +0x2C
}

// =============================================================================
//  Документация цепочки конструкторов
// =============================================================================

/// Цепочка конструкторов для C_Human entity (подтверждено из IDA):
///
/// ```text
/// 1. M2DE_BaseEntity_Construct (0x14039B710)
///    - vtable = M2DE_VT_CEntity
///    - Обнуляет +0x08..+0x70
///    - Аллоцирует два RB-tree sentinel (+0x40, +0x50)
///    - Генерирует table_id из глобального счётчика
///    - Регистрирует в WorldEntityManager
///
/// 2. M2DE_ActorEntity_Construct (0x14039A7E0)
///    - vtable = M2DE_VT_CActor
///    - Обнуляет frame_node (+0x78), owner (+0x80), и соседние ptr
///    - Устанавливает alive-флаги в +0x24
///
/// 3. M2DE_CHuman_BaseConstructor (0x140D730B0)
///    - vtable = M2DE_VT_CActor_Abstract (с _purecall)
///    - Аллоцирует 2648 байт для ВСЕХ компонентов
///    - Инициализирует:
///        +0x148 = 210.0f (здоровье)
///        +0x14C = 210.0f (макс. здоровье NPC)
///        +0x150 = 1.0f   (множитель урона)
///        +0x154 = 5.0f   (дистанция урона)
///        +0x160 = 0      (неуязвимость + мёртв)
///        +0x162 = 0      (полубог)
///
/// 4. M2DE_CHumanNPC_Constructor (0x140D712E0)
///    - vtable = 0x1418E5188 (NPC)
///    - Тип = 0x0E через SetTypeID
///    - self_ref (+0x190) = this
///    - 8 smart ptr слотов (+0x1C0..+0x238)
///
/// 5. M2DE_CPlayerEntity_Constructor (0x1400B9160) [только Player]
///    - vtable = 0x14184C060 (Player)
///    - Тип = 0x10 через SetTypeID
///    - Player-специфичные поля от +0x338
/// ```
pub const _CONSTRUCTOR_CHAIN_DOC: () = ();