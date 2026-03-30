//! Корневая игровая структура — GameManager (C_Game).
//!
//! ## Иерархия наследования
//!
//! ```text
//! I_Game
//!   └─ C_TickedModule
//!       └─ C_Game (GameManager)
//! ```
//!
//! ## Внутренний layout
//!
//! ```text
//! C_Game (0x10C48 = 68680 байт)
//!   ├─ vtable (+0x00, off_14186F450, 25 слотов)
//!   ├─ game_state_flags (+0x08, u32)
//!   │    bit 0 = initialized (GameInit/GameDone)
//!   │    bit 1 = loaded (Open/Close)
//!   │    bit 2 = suspended (SetSuspended)
//!   │    bit 3 = неизвестно
//!   │    bit 4 = устанавливается слотом [19]
//!   ├─ game_tick_counter (+0x0C, u32, 114 xrefs)
//!   ├─ sds_path_city (+0x10, *char)
//!   ├─ sds_path_extra (+0x18, *char)
//!   ├─ sds_path_streaming (+0x20, *char)
//!   ├─ sds_path_traffic (+0x28, *char)
//!   ├─ reserved (+0x30..+0x50, 5 × u64)
//!   ├─ MissionManager (+0x58, inline 296B, C_Mission)
//!   ├─ entity_slots (+0x180, 4 × *C_Human)
//!   │    [0] = active player
//!   │    [1..3] = companion / camera target / context entity
//!   ├─ EntityHashTable #1 (+0x1A0, inline 0x46B0B)
//!   ├─ EntityHashTable #2 (+0x86D8, inline 0x46B0B)
//!   └─ tail fields (+0x10C08..+0x10C48)
//! ```
//!
//! ## Статистика использования (316 xrefs на g_M2DE_GlobalManager)
//!
//! | Поле | Xrefs | Описание |
//! |:-----|:-----:|:---------|
//! | +0x180 entity_slots[0] | **166** | Активный игрок |
//! | +0x0C tick_counter | **114** | Глобальный счётчик тиков |
//! | +0x08 state_flags | 15 | Флаги состояния |
//! | +0x10..+0x28 paths | 1–2 | Базовые пути SDS-файлов |

use super::vtables::game_manager::CGameVTable;
use super::CPlayer;
use crate::macros::assert_field_offsets;
use crate::memory::Ptr;
use std::ffi::{c_char, c_void};

/// Хеш-таблица сущностей (inline подобъект внутри GameManager).
///
/// Используется для хранения и быстрого поиска entity по table_id.
/// Две таблицы существуют в GameManager: основная (+0x1A0) и вторичная (+0x86D8).
///
/// ## Внутренний layout
///
/// | Смещение | Тип | Описание |
/// |:---------|:----|:---------|
/// | +0x00 | `*u16` | Указатель на хеш-бакеты (= self+0x3EB0) |
/// | +0x08 | `*u16` | Конец хеш-бакетов (= self+0x46B0) |
/// | +0x10 | `u32` | Количество записей |
/// | +0x18..+0x28 | — | Служебные поля (обнулены) |
/// | +0x30 | `[u8; 0x3E80]` | 250 записей × 64 байта |
/// | +0x3EB0 | `[u16; 1024]` | Хеш-бакеты (init = 0xFFFF) |
///
/// Конструктор: `M2DE_EntityHashTable_Constructor` (RVA `0x1403D0F50`).
///
/// Полный размер: **0x46B0 байт (18096)**.
#[repr(C)]
pub struct EntityHashTable {
    _data: [u8; 0x46B0],
}

/// Менеджер миссий (inline подобъект внутри GameManager).
///
/// Управляет текущей миссией, скрипт-контекстом, ресурсами уровня.
/// В GameManager расположен по смещению +0x58.
///
/// ## Иерархия
///
/// ```text
/// GameModuleBase (vtable off_14186D9C8)
///   └─ MissionManager (vtable off_14186EFB8, строка "C_Mission")
/// ```
///
/// Конструктор: `M2DE_MissionManager_Constructor` (RVA `0x1403D10D0`).
/// Базовый конструктор: `M2DE_GameModuleBase_Constructor` (RVA `0x14039C1A0`).
///
/// Размер: **0x128 байт (296)**.
#[repr(C)]
pub struct MissionManagerSub {
    _data: [u8; 0x128],
}

/// Глобальный менеджер игры (C_Game).
///
/// Центральная структура движка. Содержит состояние игры,
/// ссылки на активные entity, хеш-таблицы для поиска,
/// менеджер миссий и базовые пути к SDS-ресурсам.
///
/// **Размер: 0x10C48 байт (68680).**
///
/// Глобальный указатель: `g_M2DE_GlobalManager`.
/// VTable: `off_14186F450` (25 виртуальных методов).
/// Конструктор: `M2DE_CGame_Constructor` (RVA `0x1403D1650`).
/// Создаётся в `M2DE_InitAllManagers` (RVA `0x1403DD280`).
///
/// ## Доступ к активному игроку
///
/// ```text
/// GameManager* gm = *g_M2DE_GlobalManager;
/// C_Human* player = gm->entity_slots[0];     // +0x180
/// C_HumanInventory* inv = player->inventory;  // +0xE8
/// ```
///
/// ## Флаги состояния (+0x08)
///
/// | Бит | Устанавливается | Сбрасывается | Значение |
/// |:---:|:----------------|:-------------|:---------|
/// | 0 | GameInit [8] | GameDone [9] | Мир инициализирован |
/// | 1 | Open [5] | Close [6] | Мир загружен |
/// | 2 | SetSuspended(true) [13] | SetSuspended(false) [13] | Приостановлен |
/// | 3 | — | — | Неизвестно (проверяется слотом [11]) |
/// | 4 | SetFlagBit4 [19] | — | Неизвестно |
#[repr(C)]
pub struct GameManager {
    // =====================================
    //  Заголовок (+0x00..+0x58)
    // =====================================

    /// +0x00: Указатель на VTable (25 слотов).
    ///
    /// RVA: `off_14186F450`.
    /// Класс идентифицируется строкой `"C_Game"` (слот [2]).
    pub vtable: *const CGameVTable,

    /// +0x08: Битовые флаги состояния игры.
    ///
    /// **15 обращений** (7R / 8W).
    ///
    /// Управляется vtable-слотами:
    /// - Бит 0: initialized — `GameInit` [8] / `GameDone` [9]
    /// - Бит 1: loaded — `Open` [5] / `Close` [6]
    /// - Бит 2: suspended — `SetSuspended` [13]
    /// - Бит 3: проверяется `IsFlagBit3` [11]
    /// - Бит 4: устанавливается `SetFlagBit4` [19]
    pub game_state_flags: u32,

    /// +0x0C: Глобальный счётчик игровых тиков.
    ///
    /// **114 обращений** — второе по частоте поле.
    /// Инкрементируется в `Tick` [15] каждый игровой кадр.
    ///
    /// Используется повсеместно для таймеров и проверок:
    /// - **45 мест**: сохранение снимка (`obj.saved_tick = tick_counter`)
    /// - **26 мест**: вычисление дельты (`elapsed = tick - saved`)
    /// - **18 мест**: сравнение (`if saved == tick` — в этом тике?)
    ///
    /// Доступен также через vtable слот [17] `GetTickCounter`.
    pub game_tick_counter: u32,

    /// +0x10: Базовый путь SDS — магазины и городские объекты.
    ///
    /// **2 обращения**. Используется для формирования пути:
    /// `sprintf("%s%s.sds", sds_path_city, object_name)`.
    ///
    /// Заполняется в `Open` [5] при загрузке мира.
    pub sds_path_city: *const c_char,

    /// +0x18: Базовый путь SDS — дополнительный.
    ///
    /// **1 обращение**. Используется при ночном режиме
    /// (проверяется через `M2DE_IsNightOrDarkMode()`).
    pub sds_path_extra: *const c_char,

    /// +0x20: Базовый путь SDS — стриминг контента.
    ///
    /// **1 обращение**. Используется в `C_TrafficStreaming::C_Model::LoadModel`
    /// для загрузки стриминговых моделей (state=2).
    pub sds_path_streaming: *const c_char,

    /// +0x28: Базовый путь SDS — модели транспорта.
    ///
    /// **2 обращения**. Используется в `C_TrafficStreaming::C_Model::LoadModel`
    /// для загрузки моделей автомобилей (state=1, 3).
    pub sds_path_traffic: *const c_char,

    /// +0x30..+0x50: Зарезервированные указатели.
    ///
    /// **0 обращений** через `g_M2DE_GlobalManager`.
    /// Обнулены в конструкторе. Доступ только изнутри методов.
    _reserved_30: [u64; 5],

    // =====================================
    //  Менеджер миссий (+0x58..+0x180)
    // =====================================

    /// +0x058: Менеджер миссий (inline, 296 байт).
    ///
    /// Содержит состояние текущей миссии, ресурсы, скрипт-контекст.
    /// Доступен через vtable слот [18] `GetMissionManager` (`return this + 0x58`).
    ///
    /// Наследует от `GameModuleBase` (vtable `off_14186D9C8`).
    /// Собственная vtable: `off_14186EFB8` (идентификация: `"C_Mission"`).
    pub mission_manager: MissionManagerSub,

    // =====================================
    //  Entity-слоты (+0x180..+0x1A0)
    // =====================================

    /// +0x180..+0x198: Массив entity-слотов (4 указателя).
    ///
    /// **166 обращений** к слоту 0 — самое используемое поле.
    /// Обнуляются в `GameDone` [9] и `Close` [6].
    ///
    /// Доступ через vtable:
    /// - `GetEntityFromIndex(i)` [20]: `return *(this + 8*i + 0x180)`
    /// - `SetEntityAtIndex(i, e)` [21]: `*(this + 8*i + 0x180) = e`
    ///
    /// | Индекс | Смещение | Описание |
    /// |:------:|:--------:|:---------|
    /// | 0 | +0x180 | Активный игрок (C_Human* / CPlayer*) |
    /// | 1 | +0x188 | Контекстный entity (companion?) |
    /// | 2 | +0x190 | Контекстный entity |
    /// | 3 | +0x198 | Контекстный entity |
    ///
    /// Слоты 1–3 имеют **0 прямых xrefs** через глобал — доступ
    /// только через vtable-методы `GetEntityFromIndex` / `SetEntityAtIndex`.
    pub entity_slots: [*mut CPlayer; 4],

    // =====================================
    //  Хеш-таблицы сущностей (+0x1A0..+0xCD88)
    // =====================================

    /// +0x1A0: Основная хеш-таблица сущностей (inline, 0x46B0 байт).
    ///
    /// Используется в `ActivateEntity` [23] и `OnEntityDeleted` [24].
    /// Содержит entity отсортированные по `table_id >> 8`.
    pub entity_table_1: EntityHashTable,

    /// +0x4850..+0x86D0: Промежуточная область (16000 байт).
    ///
    /// Содержимое не определено через xrefs на глобал.
    /// Включает inline-массив entity-указателей, обрабатываемый в `Tick`.
    _gap_4850: [u8; 0x3E80],

    /// +0x86D0: Счётчик записей вторичной таблицы.
    pub entity_table_2_count: u32,

    /// +0x86D4: Выравнивание.
    _pad_86d4: u32,

    /// +0x86D8: Вторичная хеш-таблица сущностей (inline, 0x46B0 байт).
    ///
    /// Аналогична основной. Используется для entity другого типа
    /// (различение по результату `sub_1403AF020`).
    pub entity_table_2: EntityHashTable,

    /// +0xCD88..+0x10C08: Промежуточная область (16000 байт).
    _gap_cd88: [u8; 0x3E80],

    // =====================================
    //  Хвостовые поля (+0x10C08..+0x10C48)
    // =====================================

    /// +0x10C08: Пороговое значение. Init = 0.2f.
    ///
    /// Используется в `Tick` [15] для расчёта частоты обработки
    /// entity из вторичной таблицы.
    pub threshold_float: f32,

    /// +0x10C0C: Выравнивание.
    _pad_10c0c: u32,

    /// +0x10C10..+0x10C28: Вектор pending-entity (std::vector<C_Entity*>).
    ///
    /// Entity, ожидающие добавления в таблицу во время тика.
    /// Три qword: begin, end, capacity.
    pub pending_add_entities: [u64; 3],

    /// +0x10C28..+0x10C40: Вектор pending-remove (std::vector<C_Entity*>).
    ///
    /// Entity, ожидающие удаления из таблицы во время тика.
    pub pending_remove_entities: [u64; 3],

    /// +0x10C40: Флаг «тик в процессе».
    ///
    /// Когда `true`, добавление/удаление entity откладывается
    /// в pending-векторы. Обрабатываются в конце `Tick`.
    pub tick_in_progress: u8,

    /// +0x10C41..+0x10C44: Выравнивание.
    _pad_10c41: [u8; 3],

    /// +0x10C44: Индекс текущего entity при обходе в `Tick`.
    ///
    /// Init = -1 (sentinel, обход не активен).
    /// При обходе: инкрементируется от 0 до count.
    pub tick_entity_index: i32,
}

assert_field_offsets!(GameManager {
    vtable               == 0x000,
    game_state_flags     == 0x008,
    game_tick_counter    == 0x00C,
    sds_path_city        == 0x010,
    sds_path_extra       == 0x018,
    sds_path_streaming   == 0x020,
    sds_path_traffic     == 0x028,
    mission_manager      == 0x058,
    entity_slots         == 0x180,
    entity_table_1       == 0x1A0,
    entity_table_2_count == 0x86D0,
    entity_table_2       == 0x86D8,
    threshold_float      == 0x10C08,
    pending_add_entities == 0x10C10,
    pending_remove_entities == 0x10C28,
    tick_in_progress     == 0x10C40,
    tick_entity_index    == 0x10C44,
});

const _: () = {
    assert!(std::mem::size_of::<GameManager>() == 0x10C48);
};

/// Индекс слота активного игрока в `entity_slots`.
pub const ENTITY_SLOT_PLAYER: usize = 0;

/// Максимальное количество entity-слотов.
pub const ENTITY_SLOT_COUNT: usize = 4;

impl GameManager {
    /// Указатель на активного игрока (entity slot 0).
    ///
    /// Возвращает `None` если игрок ещё не создан (меню, загрузка).
    #[inline]
    pub fn player_ptr(&self) -> Option<Ptr<CPlayer>> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER];
        let addr = ptr as usize;
        crate::memory::is_valid_ptr(addr).then_some(Ptr::new(addr))
    }

    /// Ссылка на активного игрока.
    ///
    /// # Safety
    ///
    /// - `entity_slots[0]` должен указывать на живой `CPlayer`.
    /// - Использовать только когда `is_initialized()` и `is_loaded()` вернули `true`.
    #[inline]
    pub unsafe fn get_player(&self) -> Option<&CPlayer> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER];
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }

    /// Мутабельная ссылка на активного игрока.
    ///
    /// # Safety
    ///
    /// Те же требования, что и для [`get_player`](Self::get_player),
    /// плюс не должно быть других активных ссылок.
    #[inline]
    pub unsafe fn get_player_mut(&mut self) -> Option<&mut CPlayer> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER];
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *ptr })
    }

    /// Entity по индексу слота (аналог vtable слота [20]).
    ///
    /// # Safety
    ///
    /// Указатель может быть невалидным если entity уничтожен.
    #[inline]
    pub unsafe fn get_entity_at_slot(&self, index: usize) -> Option<*mut CPlayer> {
        if index >= ENTITY_SLOT_COUNT {
            return None;
        }
        let ptr = self.entity_slots[index];
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Проверить наличие активного игрока.
    #[inline]
    pub fn has_player(&self) -> bool {
        !self.entity_slots[ENTITY_SLOT_PLAYER].is_null()
    }

    /// Текущий тик игры (монотонно растущий).
    ///
    /// Инкрементируется каждый кадр в `Tick` [15].
    /// Используется для таймеров: `deadline = tick() + delay`.
    #[inline]
    pub fn tick(&self) -> u32 {
        self.game_tick_counter
    }

    /// Количество тиков, прошедших с момента `since`.
    ///
    /// Корректно обрабатывает переполнение u32 (wrapping).
    #[inline]
    pub fn ticks_since(&self, since: u32) -> u32 {
        self.game_tick_counter.wrapping_sub(since)
    }

    /// Мир инициализирован (бит 0 — `GameInit` вызван).
    #[inline]
    pub fn is_initialized(&self) -> bool {
        (self.game_state_flags & 1) != 0
    }

    /// Мир загружен (бит 1 — `Open` вызван).
    #[inline]
    pub fn is_loaded(&self) -> bool {
        (self.game_state_flags & 2) != 0
    }

    /// Игра приостановлена (бит 2 — `SetSuspended(true)` вызван).
    #[inline]
    pub fn is_suspended(&self) -> bool {
        (self.game_state_flags & 4) != 0
    }

    /// Игра полностью готова (инициализирована и загружена).
    #[inline]
    pub fn is_ready(&self) -> bool {
        (self.game_state_flags & 3) == 3
    }

    /// Выполняется обход entity в `Tick`.
    ///
    /// Когда `true`, операции добавления/удаления entity
    /// откладываются в pending-векторы.
    #[inline]
    pub fn is_tick_in_progress(&self) -> bool {
        self.tick_in_progress != 0
    }

    /// Базовый путь для городских SDS-файлов.
    ///
    /// Возвращает `None` если мир не загружен.
    /// Использование: `format!("{}{}.sds", path, object_name)`.
    #[inline]
    pub fn city_sds_path(&self) -> Option<&std::ffi::CStr> {
        if self.sds_path_city.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(self.sds_path_city) })
    }
}