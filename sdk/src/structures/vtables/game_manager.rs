//! Типизированная VTable для C_Game (GameManager) — 25 слотов.
//!
//! VTable RVA: `off_14186F450`.
//! Класс идентифицируется строкой `"C_Game"` (слот [2]).
//!
//! ## Маппинг слотов
//!
//! | Слот | DE функция | Имя | Доказательство |
//! |:----:|:-----------|:----|:---------------|
//! | 0 | `0x1403D5AB0` | Dtor | Вызывает Destructor_Impl |
//! | 1 | `0x1400A78D0` | GetModuleID | `return 7` |
//! | 2 | `0x1403EB2C0` | GetClassName | `lea rax, "C_Game"` |
//! | 3 | nullsub | NOP (StaticRegister) | — |
//! | 4 | `0x1400A7D80` | GetFixedTimeStep | `return 0.005f` |
//! | 5 | `0x1403EFE50` | Open | Строки: `"/games/%s.bin"`, `"Game open"` |
//! | 6 | `0x1403DB0F0` | Close | Обнуляет entity_slots, ScriptContext |
//! | 7 | `0x1403EC5D0` | IsLoaded | `(flags & 2) != 0` |
//! | 8 | `0x1403E3880` | GameInit | Спавнит PlayerModel, бит 0 |
//! | 9 | `0x1403E3160` | GameDone | Деспавн entities, сброс бита 0 |
//! | 10 | `0x1403EC4F0` | IsInitialized | `flags & 1` |
//! | 11 | `0x1403EC710` | IsFlagBit3 | `(flags & 8) != 0` |
//! | 12 | nullsub | NOP (GameRestore) | — |
//! | 13 | `0x1403E2890` | SetSuspended | Бит 2 в flags |
//! | 14 | `0x1403EC4E0` | IsSuspended | `(flags & 4) != 0` |
//! | 15 | `0x1403F9F00` | Tick | Главный update, ++tick_counter |
//! | 16 | `0x1403EC290` | InvalidateEntity | Удаляет из entity_slots |
//! | 17 | `0x1403EB340` | GetTickCounter | `return *(this+0x0C)` |
//! | 18 | `0x1403E9100` | GetMissionManager | `return this + 0x58` |
//! | 19 | `0x1403E0030` | SetFlagBit4 | `flags \|= 0x10` |
//! | 20 | `0x1403E90B0` | GetEntityFromIndex | `*(this + 8*i + 0x180)` |
//! | 21 | `0x1403F7A20` | SetEntityAtIndex | `*(this + 8*i + 0x180) = e` |
//! | 22 | `0x1403ECF70` | GetGamePhase | `return *(byte*)(this+0x48)` |
//! | 23 | `0x1403D7C30` | ActivateEntity | Добавляет в hash tables |
//! | 24 | `0x1403EFAD0` | OnEntityDeleted | Удаляет из hash tables |

use std::ffi::{c_char, c_void};

/// VTable для C_Game (GameManager) — 25 слотов.
///
/// Все слоты подтверждены через:
/// - Строки внутри функций (`"C_Game"`, `"/games/%s.bin"`)
/// - Битовые маски флагов (+0x08)
/// - Поведение (обнуление entity_slots, entity hash table операции)
/// - Кросс-ссылки с Mac Classic (vtable `__ZTV6C_Game`)
#[repr(C)]
pub struct CGameVTable {
    // ====================================================================
    //  Жизненный цикл (0–4)
    // ====================================================================

    /// `[0]` Деструктор.
    ///
    /// Очищает: pending-векторы (+0x10C10, +0x10C28),
    /// entity hash tables (+0x1A0, +0x86D8),
    /// MissionManager (+0x58), entity_slots (+0x180..+0x198).
    pub dtor: unsafe extern "fastcall" fn(this: *mut c_void, flags: u32),

    /// `[1]` GetModuleID. Возвращает 7.
    ///
    /// Общий для системы модулей. C_Game = модуль #7.
    pub get_module_id: unsafe extern "fastcall" fn(this: *const c_void) -> u32,

    /// `[2]` GetClassName. Возвращает `"C_Game"`.
    pub get_class_name: unsafe extern "fastcall" fn(this: *const c_void) -> *const c_char,

    /// `[3]` StaticRegister. NOP в DE.
    pub _slot_03_nop: usize,

    /// `[4]` GetFixedTimeStep. Возвращает `0.005f` (200 Hz).
    pub get_fixed_time_step: unsafe extern "fastcall" fn(this: *const c_void) -> f32,

    // ====================================================================
    //  Загрузка мира (5–7)
    // ====================================================================

    /// `[5]` Open — загрузка игрового мира.
    ///
    /// Парсит файл `/games/{name}.bin`, загружает SDS-ресурсы,
    /// заполняет SDS-пути (+0x10..+0x28).
    /// Устанавливает бит 1 (loaded) в `game_state_flags`.
    pub open: unsafe extern "fastcall" fn(this: *mut c_void, game_name: *const c_char) -> bool,

    /// `[6]` Close — выгрузка игрового мира.
    ///
    /// Обнуляет `entity_slots` (+0x180..+0x198),
    /// очищает `ScriptContext`, `PlayerModelManager`.
    /// Выгружает все SDS-ресурсы.
    /// Сбрасывает бит 1 (loaded).
    pub close: unsafe extern "fastcall" fn(this: *mut c_void) -> bool,

    /// `[7]` IsLoaded. `(game_state_flags & 2) != 0`.
    pub is_loaded: unsafe extern "fastcall" fn(this: *const c_void) -> bool,

    // ====================================================================
    //  Инициализация (8–12)
    // ====================================================================

    /// `[8]` GameInit — пост-загрузо��ная инициализация.
    ///
    /// Спавнит PlayerModel через `PlayerModelManager`,
    /// активирует entity из `EntityDatabase`.
    /// Устанавливает бит 0 (initialized).
    pub game_init: unsafe extern "fastcall" fn(this: *mut c_void),

    /// `[9]` GameDone — деинициализация перед выгрузкой.
    ///
    /// Деактивирует все entity из `EntityDatabase`.
    /// Обнуляет `entity_slots` (+0x180..+0x198).
    /// Сбрасывает бит 0 (initialized).
    pub game_done: unsafe extern "fastcall" fn(this: *mut c_void),

    /// `[10]` IsInitialized. `game_state_flags & 1`.
    pub is_initialized: unsafe extern "fastcall" fn(this: *const c_void) -> bool,

    /// `[11]` IsFlagBit3. `(game_state_flags & 8) != 0`.
    pub is_flag_bit3: unsafe extern "fastcall" fn(this: *const c_void) -> bool,

    /// `[12]` NOP. В Classic это GameRestore, в DE заглушка.
    pub _slot_12_nop: usize,

    // ====================================================================
    //  Управление состоянием (13–14)
    // ====================================================================

    /// `[13]` SetSuspended. Управляет битом 2 в `game_state_flags`.
    pub set_suspended: unsafe extern "fastcall" fn(this: *mut c_void, suspended: bool),

    /// `[14]` IsSuspended. `(game_state_flags & 4) != 0`.
    pub is_suspended: unsafe extern "fastcall" fn(this: *const c_void) -> bool,

    // ====================================================================
    //  Основной тик (15–16)
    // ====================================================================

    /// `[15]` Tick — главный цикл обновления.
    ///
    /// Инкрементирует `game_tick_counter` (+0x0C).
    /// Обрабатывает entity из обеих hash table.
    /// В конце обрабатывает pending-векторы (add/remove).
    pub tick: unsafe extern "fastcall" fn(this: *mut c_void, context: *const c_void),

    /// `[16]` InvalidateEntity — удаляет entity из `entity_slots`.
    ///
    /// Проходит по слотам [0..3], если entity == context → обнуляет.
    pub invalidate_entity:
        unsafe extern "fastcall" fn(this: *mut c_void, context: *const c_void),

    // ====================================================================
    //  Запросы (17–22)
    // ====================================================================

    /// `[17]` GetTickCounter. `return *(u32*)(this + 0x0C)`.
    ///
    /// В DE 114 мест читают поле напрямую, без vtable-вызова.
    /// Vtable-версия используется редко.
    pub get_tick_counter: unsafe extern "fastcall" fn(this: *const c_void) -> u32,

    /// `[18]` GetMissionManager. `return this + 0x58`.
    ///
    /// Возвращает указатель на inline-подобъек�� MissionManager.
    pub get_mission_manager: unsafe extern "fastcall" fn(this: *const c_void) -> *mut c_void,

    /// `[19]` SetFlagBit4. `game_state_flags |= 0x10`.
    pub set_flag_bit4: unsafe extern "fastcall" fn(this: *mut c_void),

    /// `[20]` GetEntityFromIndex. `*(this + 8*index + 0x180)`.
    ///
    /// Индекс 0 = активный игрок. Массив из 4 слотов.
    pub get_entity_from_index:
        unsafe extern "fastcall" fn(this: *const c_void, index: u32) -> *mut c_void,

    /// `[21]` SetEntityAtIndex. `*(this + 8*index + 0x180) = entity`.
    ///
    /// Парный метод к `GetEntityFromIndex`.
    pub set_entity_at_index:
        unsafe extern "fastcall" fn(this: *mut c_void, entity: *mut c_void, index: u32),

    /// `[22]` GetGamePhase. `return *(u8*)(this + 0x48)`.
    ///
    /// Текущая фаза игры. Смещение +0x48 попадает в зону `_reserved_30`.
    pub get_game_phase: unsafe extern "fastcall" fn(this: *const c_void) -> u8,

    // ====================================================================
    //  Управление entity (23–24)
    // ====================================================================

    /// `[23]` ActivateEntity — добавляет entity в hash tables.
    ///
    /// Если `tick_in_progress` — откладывает в pending-вектор.
    /// Иначе добавляет в `entity_table_1` и/или `entity_table_2`
    /// в зависимости от типа entity.
    pub activate_entity:
        unsafe extern "fastcall" fn(this: *mut c_void, entity: *mut c_void, activate: bool),

    /// `[24]` OnEntityDeleted — обработка удаления entity.
    ///
    /// Удаляет из hash tables. Если `tick_in_progress` —
    /// добавляет в pending-remove вектор.
    pub on_entity_deleted:
        unsafe extern "fastcall" fn(this: *mut c_void, entity: *mut c_void),
}

const _: () = {
    assert!(std::mem::size_of::<CGameVTable>() == 25 * 8);

    assert!(std::mem::offset_of!(CGameVTable, dtor) == 0);
    assert!(std::mem::offset_of!(CGameVTable, get_class_name) == 2 * 8);
    assert!(std::mem::offset_of!(CGameVTable, open) == 5 * 8);
    assert!(std::mem::offset_of!(CGameVTable, close) == 6 * 8);
    assert!(std::mem::offset_of!(CGameVTable, is_loaded) == 7 * 8);
    assert!(std::mem::offset_of!(CGameVTable, game_init) == 8 * 8);
    assert!(std::mem::offset_of!(CGameVTable, game_done) == 9 * 8);
    assert!(std::mem::offset_of!(CGameVTable, is_initialized) == 10 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_suspended) == 13 * 8);
    assert!(std::mem::offset_of!(CGameVTable, tick) == 15 * 8);
    assert!(std::mem::offset_of!(CGameVTable, invalidate_entity) == 16 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_tick_counter) == 17 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_mission_manager) == 18 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_flag_bit4) == 19 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_entity_from_index) == 20 * 8);
    assert!(std::mem::offset_of!(CGameVTable, set_entity_at_index) == 21 * 8);
    assert!(std::mem::offset_of!(CGameVTable, get_game_phase) == 22 * 8);
    assert!(std::mem::offset_of!(CGameVTable, activate_entity) == 23 * 8);
    assert!(std::mem::offset_of!(CGameVTable, on_entity_deleted) == 24 * 8);
};