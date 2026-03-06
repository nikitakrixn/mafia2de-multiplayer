use std::ffi::c_void;

/// Менеджер игровых событий и обратных вызовов.
///
/// VTable: `vtables::callbacks::GAME_CALLBACK_MANAGER`
/// Global: `globals::GAME_CALLBACK_MANAGER`
///
/// Содержит 39 зарегистрированных событий:
/// System Init/Done, Game Tick, Mission events, Loading, Save/Load...
#[repr(C)]
pub struct GameCallbackManager {
    pub vtable: *const c_void,                  // +0x00
    _pad_008: [u8; 0x08],
    pub callbacks_begin: *mut CallbackEntry,    // +0x10
    pub callbacks_end: *mut CallbackEntry,      // +0x18
    pub callbacks_capacity: *mut CallbackEntry, // +0x20
}

/// Одна запись обратного вызова (64 байта).
#[repr(C)]
pub struct CallbackEntry {
    pub name: [u8; 32],                         // +0x00: Имя события (null-terminated)
    pub event_type: i32,                        // +0x20: Тип события (GameEventType)
    pub event_id: i32,                          // +0x24: ID события (обычно 0)
    pub callback: *const c_void,                // +0x28: Указатель на функцию
    pub context: *mut c_void,                   // +0x30: Контекст (опционально)
    _unknown: [u8; 24],                         // +0x38: Неизвестные поля
}

/// Типы игровых событий.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEventType {
    SystemInit                  = 1,
    SystemDone                  = 2,
    GameTick                    = 3,
    GameTickPaused              = 4,
    GameTickAlways              = 5,
    GameRender                  = 7,
    MissionQuit                 = 8,
    MissionBeforeOpen           = 9,
    MissionAfterOpen            = 10,
    MissionBeforeClose          = 11,
    MissionAfterClose           = 12,
    GameInit                    = 13,
    GameDone                    = 14,
    InvalidateEntity            = 15,
    InvalidateFrame             = 16,
    WriteGameInfo               = 17,
    ReadGameInfo                = 18,
    GameRestore                 = 19,
    NoGameStart                 = 20,
    NoGameEnd                   = 21,
    NoGameTick                  = 22,
    NoGameRender                = 23,
    NoGameAfterGameLoop         = 24,
    CollisionsLoaded            = 25,
    ApackFromSdsLoaded          = 26,
    RegisterGameSaveCb          = 27,
    GameparamsChanged           = 28,
    GameparamsPresave           = 29,
    AppDeactivate               = 30,
    AppActivate                 = 31,
    LoadingProcessStarted       = 32,
    LoadingProcessFinished      = 33,
    GamePaused                  = 34,
    GameUnpaused                = 35,
    LoadingFadeFinished         = 36,
    SlotWaitingTick             = 37,
    SlotWaitingRender           = 38,
    Shutdown                    = 40,
    WeatherManagerCreated       = 4097,
}

const _: () = {
    assert!(std::mem::offset_of!(CallbackEntry, event_type) == 0x20);
    assert!(std::mem::offset_of!(CallbackEntry, callback) == 0x28);
};