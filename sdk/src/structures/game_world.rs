//! Корневая игровая структура — GameManager (`C_Game`).
//!
//! Это центральный объект игрового мира:
//! - хранит состояние загрузки/инициализации игры,
//! - ведёт глобальный счётчик игровых тиков,
//! - содержит пути к SDS-ресурсам,
//! - владеет inline-менеджером миссии,
//! - хранит 4 entity-слота (включая активного игрока),
//! - содержит две hash table для entity,
//! - держит pending-списки для отложенного добавления/удаления entity.
//!
//! ## Иерархия
//!
//! ```text
//! I_Game
//!   └─ C_TickedModule
//!       └─ C_Game (GameManager)
//! ```
//!
//! ## Layout
//!
//! ```text
//! C_Game (0x10C48 = 68680 байт)
//!   ├─ +0x000 vtable (off_14186F450)
//!   ├─ +0x008 game_state_flags
//!   ├─ +0x00C game_tick_counter
//!   ├─ +0x010 sds_path_city
//!   ├─ +0x018 sds_path_extra
//!   ├─ +0x020 sds_path_streaming
//!   ├─ +0x028 sds_path_traffic
//!   ├─ +0x058 MissionManager inline (0x128 байт)
//!   ├─ +0x180 entity_slots[4]
//!   │    [0] = active player
//!   ├─ +0x1A0 EntityHashTable #1
//!   ├─ +0x86D8 EntityHashTable #2
//!   ├─ +0x10C10 pending_add_entities
//!   ├─ +0x10C28 pending_remove_entities
//!   ├─ +0x10C40 tick_in_progress
//!   └─ +0x10C44 tick_entity_index
//! ```
//!
//! ## Статистика использования
//!
//! По xrefs на `g_M2DE_GlobalManager`:
//!
//! | Поле | Обращений | Назначение |
//! |:-----|:---------:|:-----------|
//! | `+0x180` | **166** | entity slot 0 — активный игрок |
//! | `+0x0C` | **114** | глобальный счётчик тиков |
//! | `+0x08` | 15 | флаги состояния |
//! | `+0x10..+0x28` | 1–2 | базовые пути SDS |
//!
//! ## Флаги состояния (`game_state_flags`, +0x08)
//!
//! Известные биты:
//! - bit 0 = мир инициализирован (`GameInit` / `GameDone`)
//! - bit 1 = мир загружен (`Open` / `Close`)
//! - bit 2 = suspended
//! - bit 3 = неизвестно, но проверяется отдельным vfunc
//! - bit 4 = устанавливается отдельным vfunc

use super::vtables::game_manager::CGameVTable;
use super::CPlayer;
use crate::macros::assert_field_offsets;
use crate::memory::Ptr;
use std::ffi::{c_char, c_void};

/// Хеш-таблица сущностей (inline подобъект внутри GameManager).
///
/// Используется для хранения и быстрого поиска entity по `table_id >> 8`.
/// Внутри `C_Game` таких таблиц две:
/// - основная по `+0x1A0`,
/// - вторичная по `+0x86D8`.
///
/// ## Внутренний layout
///
/// | Смещение | Тип | Описание |
/// |:---------|:----|:---------|
/// | `+0x00` | `*u16` | начало массива бакетов (`self + 0x3EB0`) |
/// | `+0x08` | `*u16` | конец массива бакетов |
/// | `+0x10` | `u32` | число записей |
/// | `+0x18..+0x28` | — | служебные поля |
/// | `+0x30..+0x3EB0` | — | 250 записей по 64 байта |
/// | `+0x3EB0..+0x46B0` | `[u16; 1024]` | хеш-бакеты (`0xFFFF` при init) |
///
/// Конструктор: `M2DE_EntityHashTable_Constructor` (`0x1403D0F50`).
///
/// Полный размер: **0x46B0 байт (18096)**.
#[repr(C)]
pub struct EntityHashTable {
    _data: [u8; 0x46B0],
}

/// Менеджер миссий (inline подобъект внутри `C_Game`).
///
/// Хранит текущее состояние миссии, связанный скриптовый контекст и
/// служебные миссионные данные.
///
/// Расположен по смещению `+0x58` внутри `C_Game`.
///
/// ## Иерархия
///
/// ```text
/// GameModuleBase (vtable off_14186D9C8)
///   └─ MissionManager (vtable off_14186EFB8, строка "C_Mission")
/// ```
///
/// Конструкторы:
/// - `M2DE_GameModuleBase_Constructor` (`0x14039C1A0`)
/// - `M2DE_MissionManager_Constructor` (`0x1403D10D0`)
///
/// Полный размер: **0x128 байт (296)**.
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
/// Конструктор: `M2DE_CGame_Constructor` (`0x1403D1650`).
/// Создаётся в `M2DE_InitAllManagers`.
///
/// Полный размер: **0x10C48**.
#[repr(C)]
pub struct GameManager {
    // =========================================================================
    //  Заголовок (+0x00..+0x58)
    // =========================================================================

    /// `+0x00`: VTable `C_Game`.
    ///
    /// Адрес в `.rdata`: `off_14186F450`.
    /// Содержит 25 виртуальных методов.
    pub vtable: *const CGameVTable,

    /// `+0x08`: Флаги состояния игры.
    ///
    /// Известные биты:
    /// - bit 0 = initialized
    /// - bit 1 = loaded
    /// - bit 2 = suspended
    /// - bit 3 = неизвестно
    /// - bit 4 = устанавливается отдельной виртуальной функцией
    pub game_state_flags: u32,

    /// +0x0C: Глобальный счётчик игрового времени.
    ///
    /// Это не просто "счётчик кадров".
    /// Значение увеличивается в `C_Game::Tick` на delta из tick-context,
    /// то есть представляет собой монотонный таймер игрового мира.
    ///
    /// Runtime-наблюдения:
    /// - в меню = 0
    /// - в игре растёт примерно на число миллисекунд
    /// - на pause menu не увеличивается
    ///
    /// Используется как глобальный timestamp:
    /// - сохранение снимка события,
    /// - вычисление elapsed time,
    /// - сравнение с дедлайнами.
    pub game_tick_counter: u32,

    /// `+0x10`: Базовый путь SDS для городских и shop-ресурсов.
    pub sds_path_city: *const c_char,

    /// `+0x18`: Дополнительный SDS-путь.
    ///
    /// Используется редко. Замечен в логике ночных/альтернативных вариантов.
    pub sds_path_extra: *const c_char,

    /// `+0x20`: Базовый путь SDS для streaming-ресурсов.
    pub sds_path_streaming: *const c_char,

    /// `+0x28`: Базовый путь SDS для traffic / vehicle model ресурсов.
    pub sds_path_traffic: *const c_char,

    /// `+0x30..+0x50`: пока неразобранные служебные указатели.
    ///
    /// Через `g_M2DE_GlobalManager` напрямую почти не используются,
    /// но участвуют во внутренней логике `C_Game`.
    _reserved_30: [u64; 5],

    // =========================================================================
    //  Inline MissionManager (+0x58..+0x180)
    // =========================================================================

    /// `+0x58`: inline-подобъект менеджера миссии.
    ///
    /// Возвращается виртуальной функцией `GetMissionManager()`.
    pub mission_manager: MissionManagerSub,

    // =========================================================================
    //  Entity slots (+0x180..+0x1A0)
    // =========================================================================

    /// `+0x180..+0x198`: массив из 4 entity-слотов.
    ///
    /// Доступ через vtable:
    /// - `GetEntityFromIndex(i)` → `*(this + 0x180 + i*8)`
    /// - `SetEntityAtIndex(i, entity)` → запись туда же
    ///
    /// Известно:
    /// - `entity_slots[0]` = активный игрок
    /// - `entity_slots[1..=3]` = дополнительные контекстные entity
    ///
    /// Тип элементов оставлен как `*mut c_void`, потому что только слот 0
    /// уверенно известен как `C_Player2` / `C_Human`.
    pub entity_slots: [*mut c_void; 4],

    // =========================================================================
    //  Entity hash tables
    // =========================================================================

    /// `+0x1A0`: основная hash table entity.
    pub entity_table_1: EntityHashTable,

    /// `+0x4850..+0x86D0`: пока неразобранная область.
    _gap_4850: [u8; 0x3E80],

    /// `+0x86D0`: счётчик второй таблицы.
    pub entity_table_2_count: u32,

    /// `+0x86D4`: padding.
    _pad_86d4: u32,

    /// `+0x86D8`: вторичная hash table entity.
    pub entity_table_2: EntityHashTable,

    /// `+0xCD88..+0x10C08`: пока неразобранная область.
    _gap_cd88: [u8; 0x3E80],

    // =========================================================================
    //  Хвостовые поля (+0x10C08..+0x10C48)
    // =========================================================================

    /// `+0x10C08`: пороговое значение.
    ///
    /// Инициализируется как `0.2f`.
    /// Используется в логике `Tick` для расчёта частоты части обновлений.
    pub threshold_float: f32,

    /// `+0x10C0C`: padding.
    _pad_10c0c: u32,

    /// `+0x10C10..+0x10C28`: pending-add vector.
    ///
    /// Формат как у `std::vector<T>`:
    /// - begin
    /// - end
    /// - capacity_end
    ///
    /// Здесь откладываются entity на добавление,
    /// если `tick_in_progress != 0`.
    pub pending_add_entities: [u64; 3],

    /// `+0x10C28..+0x10C40`: pending-remove vector.
    ///
    /// Здесь откладываются entity на удаление,
    /// если `tick_in_progress != 0`.
    pub pending_remove_entities: [u64; 3],

    /// `+0x10C40`: флаг "тик в процессе".
    ///
    /// Пока установлен, операции добавления/удаления entity
    /// не применяются напрямую, а складываются в pending-векторы.
    pub tick_in_progress: u8,

    /// `+0x10C41..+0x10C43`: padding.
    _pad_10c41: [u8; 3],

    /// `+0x10C44`: индекс текущего entity при обходе в `Tick`.
    ///
    /// Начальное значение: `-1`.
    pub tick_entity_index: i32,
}

assert_field_offsets!(GameManager {
    vtable                  == 0x000,
    game_state_flags        == 0x008,
    game_tick_counter       == 0x00C,
    sds_path_city           == 0x010,
    sds_path_extra          == 0x018,
    sds_path_streaming      == 0x020,
    sds_path_traffic        == 0x028,
    mission_manager         == 0x058,
    entity_slots            == 0x180,
    entity_table_1          == 0x1A0,
    entity_table_2_count    == 0x86D0,
    entity_table_2          == 0x86D8,
    threshold_float         == 0x10C08,
    pending_add_entities    == 0x10C10,
    pending_remove_entities == 0x10C28,
    tick_in_progress        == 0x10C40,
    tick_entity_index       == 0x10C44,
});

const _: () = {
    assert!(std::mem::size_of::<GameManager>() == 0x10C48);
};

/// Индекс слота активного игрока в `entity_slots`.
pub const ENTITY_SLOT_PLAYER: usize = 0;

/// Количество entity-слотов в `C_Game`.
pub const ENTITY_SLOT_COUNT: usize = 4;

impl GameManager {
    /// Типизированный указатель на активного игрока (slot 0).
    ///
    /// Возвращает `None`, если игрок ещё не создан.
    #[inline]
    pub fn player_ptr(&self) -> Option<Ptr<CPlayer>> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER] as *mut CPlayer;
        let addr = ptr as usize;
        crate::memory::is_valid_ptr(addr).then_some(Ptr::new(addr))
    }

    /// Получить ссылку на активного игрока.
    ///
    /// # Safety
    ///
    /// - `entity_slots[0]` должен указывать на живой `CPlayer`
    /// - вызывать только в корректном игровом контексте
    #[inline]
    pub unsafe fn get_player(&self) -> Option<&CPlayer> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER] as *const CPlayer;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }

    /// Получить мутабельную ссылку на активного игрока.
    ///
    /// # Safety
    ///
    /// - те же требования, что и у [`get_player`](Self::get_player)
    /// - не должно быть других активных ссылок на этот объект
    #[inline]
    pub unsafe fn get_player_mut(&mut self) -> Option<&mut CPlayer> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER] as *mut CPlayer;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *ptr })
    }

    /// Получить raw entity pointer из одного из 4 слотов.
    #[inline]
    pub fn entity_slot(&self, index: usize) -> Option<*mut c_void> {
        if index >= ENTITY_SLOT_COUNT {
            return None;
        }
        let ptr = self.entity_slots[index];
        (!ptr.is_null()).then_some(ptr)
    }

    /// Есть ли активный игрок.
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

    /// Сколько тиков прошло с момента `since`.
    ///
    /// Используется wrapping arithmetic, так что работает и при переполнении.
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
        (self.game_state_flags & 0x3) == 0x3
    }

    /// Выполняется обход entity в `Tick`.
    ///
    /// Когда `true`, операции добавления/удаления entity
    /// откладываются в pending-векторы.
    #[inline]
    pub fn is_tick_in_progress(&self) -> bool {
        self.tick_in_progress != 0
    }

    /// Получить текущую игровую фазу.
    ///
    /// Значение хранится во внутреннем поле `C_Game` и доступно
    /// через виртуальную функцию `[22] GetGamePhase`.
    #[inline]
    pub fn game_phase(&self) -> u8 {
        unsafe { ((*self.vtable).get_game_phase)(self as *const _ as *const c_void) }
    }

    /// Получить inline MissionManager через vtable.
    ///
    /// Обычно это просто `self + 0x58`, но этот метод отражает
    /// реальную виртуальную функцию движка.
    #[inline]
    pub fn mission_manager_ptr(&self) -> *mut c_void {
        unsafe { ((*self.vtable).get_mission_manager)(self as *const _ as *const c_void) }
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

    /// Базовый путь для streaming SDS-файлов.
    #[inline]
    pub fn streaming_sds_path(&self) -> Option<&std::ffi::CStr> {
        if self.sds_path_streaming.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(self.sds_path_streaming) })
    }

    /// Базовый путь для traffic SDS-файлов.
    #[inline]
    pub fn traffic_sds_path(&self) -> Option<&std::ffi::CStr> {
        if self.sds_path_traffic.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(self.sds_path_traffic) })
    }
}