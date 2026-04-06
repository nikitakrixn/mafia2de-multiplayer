//! Корневая игровая структура — `C_Game`.
//!
//! Центральный объект игрового мира в Mafia II: Definitive Edition.
//!
//! ## Иерархия
//!
//! ```text
//! I_Game
//!   └─ C_TickedModule
//!       └─ C_Game
//! ```
//!
//! ## Layout (`C_Game`, 0x10C48 байт)
//!
//! ```text
//! +0x000  vtable              -> M2DE_VT_CGame (25 слотов)
//! +0x008  game_state_flags    u32
//! +0x00C  game_tick_counter   u32
//! +0x010  game_name           *c_char
//! +0x018  game_str1           *c_char
//! +0x020  game_str2           *c_char
//! +0x028  game_str3           *c_char
//! +0x010  sds_path_city       *c_char  (пример: "/sds/City/")
//! +0x018  sds_path_shops      *c_char  (пример: "/sds/Shops/")
//! +0x020  sds_path_traffic    *c_char  (пример: "/sds/Traffic/")
//! +0x028  sds_path_cars       *c_char  (пример: "/sds/Cars/")
//! +0x030  sds_count           u32
//! +0x038  script_name         *c_char  (пример: "CITY_trick")
//! +0x040  actors_bin_path     *c_char  (пример: "/missions/CITY/actors_player.bin")
//! +0x048  weather_type        u8   (из .bin тег 3, byte[0])
//! +0x049  _unk_049            u8   (из .bin тег 3, byte[1])
//! +0x050  game_bin_data       *mut c_void  (загруженный .bin буфер)
//! +0x058  actors_pack         C_ActorsPack (0x128 байт)
//! +0x180  entity_slots        [*mut c_void; 4]
//! +0x1A0  entity_table_1      EntityHashTable (0x46B0 байт)
//! +0x86D0 entity_table_2_count u32
//! +0x86D8 entity_table_2      EntityHashTable (0x46B0 байт)
//! +0x10C08 threshold_float    f32 (= 0.2)
//! +0x10C10 pending_add        [u64; 3]  (std::vector begin/end/cap)
//! +0x10C28 pending_remove     [u64; 3]  (std::vector begin/end/cap)
//! +0x10C40 tick_in_progress   u8
//! +0x10C44 tick_entity_index  i32 (init = -1)
//! ```
//!
//! ## Флаги состояния (`game_state_flags`, +0x08)
//!
//! | Бит | Константа | Назначение |
//! |:---:|:----------|:-----------|
//! | 0 | `STATE_GAME_INIT` | initialized — `GameInit` вызван |
//! | 1 | `STATE_OPEN` | loaded — `Open` вызван |
//! | 2 | `STATE_PAUSED` | игра на паузе |
//! | 3 | `STATE_UNK3` | неизвестно |
//! | 4 | `STATE_DELETE_PENDING` | объект помечен на удаление |

use super::CPlayer;
use super::vtables::game_manager::CGameVTable;
use crate::macros::assert_field_offsets;
use crate::memory::Ptr;
use std::ffi::{c_char, c_void};

/// Флаги состояния `C_Game` (`game_state_flags`, +0x08).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStateFlag {
    /// Бит 0 — мир инициализирован (`GameInit` вызван).
    GameInit = 0x01,
    /// Бит 1 — мир загружен (`Open` вызван).
    Open = 0x02,
    /// Бит 2 — игра на паузе (`SetSuspended` / tick context).
    Paused = 0x04,
    /// Бит 3 — назначение не установлено (всегда true когда мир загружен).
    Unk3 = 0x08,
    /// Бит 4 — объект помечен на удаление (`STATE_DELETE_PENDING`).
    DeletePending = 0x10,
}

impl GameStateFlag {
    pub fn is_set(self, flags: u32) -> bool {
        flags & (self as u32) != 0
    }
}

/// Индексы entity-слотов в `C_Game`.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySlot {
    /// Слот 0 — активный игрок (`C_Human`).
    Player = 0,
    Slot1 = 1,
    Slot2 = 2,
    Slot3 = 3,
}

/// Хеш-таблица entity (inline подобъект внутри `C_Game`).
///
/// Внутри `C_Game` таких таблиц две: по `+0x1A0` и `+0x86D8`.
///
/// ## Layout (0x46B0 байт)
///
/// | Смещение | Тип | Описание |
/// |:---------|:----|:---------|
/// | `+0x00` | `*u16` | указатель на начало массива бакетов |
/// | `+0x08` | `*u16` | указатель на конец массива бакетов |
/// | `+0x10` | `u32` | число записей |
/// | `+0x30..+0x3EB0` | — | 250 записей по 64 байта |
/// | `+0x3EB0..+0x46B0` | `[u16; 1024]` | хеш-бакеты (init = `0xFFFF`) |
///
/// Конструктор: `M2DE_EntityHashTable_Constructor` (`0x1403D0F50`).
#[repr(C)]
pub struct EntityHashTable {
    _data: [u8; 0x46B0],
}

/// Inline подобъект `C_ActorsPack` внутри `C_Game` (`+0x58`).
///
/// Возвращается vtable-слотом `[18] GetActorsPack()`.
///
/// ## Иерархия
///
/// ```text
/// I_ActorsPack  (vtable M2DE_VT_IActorsPack  @ 0x14186D9C8)
///   └─ C_ActorsPack (vtable M2DE_VT_CActorsPack @ 0x14186EFB8)
/// ```
///
/// Конструктор: `M2DE_CActorsPack_Constructor`.
/// Размер: **0x128 байт**.
#[repr(C)]
pub struct CActorsPackSub {
    _data: [u8; 0x128],
}

/// Глобальный объект `C_Game` — корень игрового мира.
///
/// Глобальный указатель: `g_M2DE_CGame` (`0x141CAF770`).
/// Конструктор: `M2DE_CGame_Constructor` (`0x1403D1650`).
///
/// Размер: **0x10C48 байт**.
#[repr(C)]
pub struct GameManager {
    // =========================================================================
    //  Заголовок (+0x00..+0x58)
    // =========================================================================
    /// `+0x000` VTable `C_Game` -> `M2DE_VT_CGame` (`0x14186F450`), 25 слотов.
    pub vtable: *const CGameVTable,

    /// `+0x008` Флаги состояния игры.
    ///
    /// | Бит | Константа | Назначение |
    /// |:---:|:----------|:-----------|
    /// | 0 | `STATE_GAME_INIT` | initialized |
    /// | 1 | `STATE_OPEN` | loaded / opened |
    /// | 2 | `STATE_PAUSED` | игра на паузе |
    /// | 3 | `STATE_UNK3` | неизвестно |
    /// | 4 | `STATE_DELETE_PENDING` | объект помечен на удаление |
    pub game_state_flags: u32,

    /// `+0x00C` Монотонный счётчик игрового времени.
    ///
    /// Увеличивается в `C_Game::Tick` на delta из tick-context.
    /// В меню = 0, на pause menu не растёт.
    /// Используется как timestamp для таймеров и дедлайнов.
    pub game_tick_counter: u32,

    /// `+0x010` Базовый путь SDS для городских ресурсов.
    ///
    /// Пример: `"/sds/City/"`.
    pub sds_path_city: *const c_char,

    /// `+0x018` Базовый путь SDS для shop-ресурсов.
    ///
    /// Пример: `"/sds/Shops/"`.
    pub sds_path_shops: *const c_char,

    /// `+0x020` Базовый путь SDS для traffic-ресурсов.
    ///
    /// Пример: `"/sds/Traffic/"`.
    pub sds_path_traffic: *const c_char,

    /// `+0x028` Базовый путь SDS для vehicle model ресурсов.
    ///
    /// Пример: `"/sds/Cars/"`.
    pub sds_path_cars: *const c_char,

    /// `+0x030` Счётчик SDS-слотов, заполняется в `C_Game::Open`.
    ///
    /// `*(_DWORD *)(a1 + 48) = v42` в `ParseData` — количество SDS записей.
    pub sds_count: u32,

    _pad_034: u32,

    /// `+0x038` Имя текущего скрипта / миссии.
    ///
    /// Пример: `"CITY_trick"`.
    pub script_name: *const c_char,

    /// `+0x040` Путь к `.bin` файлу актёров игрока.
    ///
    /// Пример: `"/missions/CITY/actors_player.bin"`.
    pub actors_bin_path: *const c_char,

    /// `+0x048` Тип погоды.
    ///
    /// Заполняется из `.bin` секции тег 3, byte[0].
    pub weather_type: u8,

    /// `+0x049` Неизвестный байт.
    ///
    /// Заполняется из `.bin` секции тег 3, byte[1].
    pub _unk_049: u8,

    _pad_04a: [u8; 6],

    /// `+0x050` Указатель на бинарные данные игры.
    ///
    /// Содержит бинарную структуру (не C-строку).
    /// Magic bytes в наблюдаемых сессиях: `rpmg` (`0x676D7072`).
    pub game_bin_data: *mut c_void,

    // =========================================================================
    //  Inline C_ActorsPack (+0x058..+0x180)
    // =========================================================================
    /// `+0x058` Inline-подобъект `C_ActorsPack`.
    ///
    /// Доступен через vtable-слот `[18] GetActorsPack()` -> `this + 0x58`.
    pub actors_pack: CActorsPackSub,

    // =========================================================================
    //  Entity slots (+0x180..+0x1A0)
    // =========================================================================
    /// `+0x180` Массив из 4 entity-слотов.
    ///
    /// | Индекс | Назначение |
    /// |:------:|:-----------|
    /// | 0 | активный игрок (`C_Human`) |
    /// | 1–3 | дополнительные контекстные entity |
    ///
    /// Доступ через vtable: `GetEntityFromIndex(i)` / `SetEntityAtIndex(i, e)`.
    pub entity_slots: [*mut c_void; 4],

    // =========================================================================
    //  Entity hash tables (+0x1A0..+0x10C08)
    // =========================================================================
    /// `+0x1A0` Основная hash table entity.
    pub entity_table_1: EntityHashTable,

    /// `+0x4850..+0x86D0` Неразобранная область между таблицами.
    _gap_4850: [u8; 0x3E80],

    /// `+0x86D0` Счётчик записей второй hash table.
    pub entity_table_2_count: u32,

    _pad_86d4: u32,

    /// `+0x86D8` Вторичная hash table entity.
    pub entity_table_2: EntityHashTable,

    /// `+0xCD88..+0x10C08` Неразобранная область.
    _gap_cd88: [u8; 0x3E80],

    // =========================================================================
    //  Хвостовые поля (+0x10C08..+0x10C48)
    // =========================================================================
    /// `+0x10C08` Пороговое значение для логики `Tick`.
    ///
    /// Инициализируется как `0.2f` (`3E4CCCCDh`).
    pub threshold_float: f32,

    _pad_10c0c: u32,

    /// `+0x10C10` Pending-add вектор (`std::vector<C_Entity*>`).
    ///
    /// Entity откладываются сюда если `tick_in_progress != 0`.
    /// Формат: `[begin, end, capacity_end]`.
    pub pending_add_entities: [u64; 3],

    /// `+0x10C28` Pending-remove вектор (`std::vector<C_Entity*>`).
    ///
    /// Entity откладываются сюда если `tick_in_progress != 0`.
    /// Формат: `[begin, end, capacity_end]`.
    pub pending_remove_entities: [u64; 3],

    /// `+0x10C40` Флаг активного обхода entity в `Tick`.
    ///
    /// Пока установлен — add/remove операции идут в pending-векторы.
    pub tick_in_progress: u8,

    _pad_10c41: [u8; 3],

    /// `+0x10C44` Индекс текущего entity при обходе в `Tick`.
    ///
    /// Инициализируется как `-1`.
    pub tick_entity_index: i32,
}

assert_field_offsets!(GameManager {
    vtable                  == 0x000,
    game_state_flags        == 0x008,
    game_tick_counter       == 0x00C,
    sds_path_city           == 0x010,
    sds_path_shops          == 0x018,
    sds_path_traffic        == 0x020,
    sds_path_cars           == 0x028,
    sds_count               == 0x030,
    script_name             == 0x038,
    actors_bin_path         == 0x040,
    weather_type            == 0x048,
    game_bin_data           == 0x050,
    actors_pack             == 0x058,
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

pub const ENTITY_SLOT_PLAYER: usize = EntitySlot::Player as usize;
pub const ENTITY_SLOT_COUNT: usize = 4;

impl GameManager {
    /// Типизированный указатель на активного игрока (slot 0).
    #[inline]
    pub fn player_ptr(&self) -> Option<Ptr<CPlayer>> {
        let addr = self.entity_slots[ENTITY_SLOT_PLAYER] as usize;
        crate::memory::is_valid_ptr(addr).then_some(Ptr::new(addr))
    }

    /// Ссылка на активного игрока.
    ///
    /// # Safety
    /// `entity_slots[0]` должен указывать на живой `CPlayer`.
    #[inline]
    pub unsafe fn get_player(&self) -> Option<&CPlayer> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER] as *const CPlayer;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }

    /// Мутабельная ссылка на активного игрока.
    ///
    /// # Safety
    /// Те же требования что у [`get_player`](Self::get_player).
    /// Не должно быть других активных ссылок на этот объект.
    #[inline]
    pub unsafe fn get_player_mut(&mut self) -> Option<&mut CPlayer> {
        let ptr = self.entity_slots[ENTITY_SLOT_PLAYER] as *mut CPlayer;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *ptr })
    }

    /// Raw pointer из entity-слота по индексу.
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

    /// Текущий тик игры.
    #[inline]
    pub fn tick(&self) -> u32 {
        self.game_tick_counter
    }

    /// Количество тиков прошедших с момента `since` (wrapping).
    #[inline]
    pub fn ticks_since(&self, since: u32) -> u32 {
        self.game_tick_counter.wrapping_sub(since)
    }

    /// Мир инициализирован (bit 0).
    #[inline]
    pub fn is_initialized(&self) -> bool {
        GameStateFlag::GameInit.is_set(self.game_state_flags)
    }

    /// Мир загружен (bit 1).
    #[inline]
    pub fn is_loaded(&self) -> bool {
        GameStateFlag::Open.is_set(self.game_state_flags)
    }

    /// Игра приостановлена (bit 2).
    #[inline]
    pub fn is_suspended(&self) -> bool {
        GameStateFlag::Paused.is_set(self.game_state_flags)
    }

    /// Мир полностью готов (initialized + loaded).
    #[inline]
    pub fn is_ready(&self) -> bool {
        let mask = GameStateFlag::GameInit as u32 | GameStateFlag::Open as u32;
        self.game_state_flags & mask == mask
    }

    /// Выполняется обход entity в `Tick`.
    #[inline]
    pub fn is_tick_in_progress(&self) -> bool {
        self.tick_in_progress != 0
    }

    /// Текущий тип погоды / игровая фаза (vtable slot `[22]`).
    #[inline]
    pub fn game_phase(&self) -> u8 {
        unsafe { ((*self.vtable).get_game_phase)(self as *const _ as *const c_void) }
    }

    /// Указатель на inline `C_ActorsPack` (vtable slot `[18]`).
    #[inline]
    pub fn actors_pack_ptr(&self) -> *mut c_void {
        unsafe { ((*self.vtable).get_actors_pack)(self as *const _ as *const c_void) }
    }

    /// Имя текущего скрипта / миссии (`script_name`, `+0x038`).
    #[inline]
    pub fn get_script_name(&self) -> Option<&std::ffi::CStr> {
        if self.script_name.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(self.script_name) })
    }

    /// Путь к `.bin` файлу актёров игрока (`actors_bin_path`, `+0x040`).
    #[inline]
    pub fn get_actors_bin_path(&self) -> Option<&std::ffi::CStr> {
        if self.actors_bin_path.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(self.actors_bin_path) })
    }
}
