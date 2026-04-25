//! `C_GameInputModule` — глобальный singleton подсистемы ввода.
//!
//! Хранит и тикает три `C_InputLayer`, регистрирует listeners экшенов,
//! управляет паузой ввода и Force Feedback. Каждый кадр игра дёргает его
//! `Tick` через `GameCallbackManager`, и из него вызывается
//! `C_GameInput::Update` -> `NotifyAllInputListeners`, который и кормит
//! камеру, движение, UI-mouse-кликами.
//!
//! ## Singleton
//!
//! Объект сам по себе живёт по фиксированному RVA
//! [`globals::GAME_INPUT_MODULE`][crate::addresses::globals::GAME_INPUT_MODULE]
//! — это не указатель, а сама структура. Конструируется в статическом
//! инициализаторе `M2DE_GameInputModule_StaticInit` (`0x14007D5B0`) до
//! `WinMain`.
//!
//! ## Layout (по `M2DE_GameInputModule_Constructor` @ `0x140FD7880`)
//!
//! ```text
//! +0x0000  vtable               -> 0x141908AC0
//! +0x0008  sys_input            *C_SysInput      низкоуровневый менеджер DI-устройств
//! +0x0010  game_input           *void            внутренний `C_GameInput`-менеджер
//! +0x0018  ... 0x1FD0 байт служебного состояния (listener-pools, params, и т. п.)
//! +0x1FE8  m_bSomeFlag          u8
//! +0x1FF0  layer0               *C_InputLayer
//! +0x1FF8  layer1               *C_InputLayer
//! +0x2000  layer2               *C_InputLayer
//! +0x2008  m_b_input_paused     u8   <-- Tick проверяет; true -> весь Update пропущен
//! +0x207C  m_b_game_paused      u8   <-- OnGamePaused/PauseInput, для совместимости
//! +0x20A8  listeners_sentinel   *Node sentinel двусвязного списка action-listener'ов
//! ```
//!
//! Полный размер 8385 байт. Большая часть полей внутри pad-блока пока
//! не нужна и хранится как непрозрачный массив байт (`_pad_*`), чтобы
//! гарантировать корректные офсеты без UB.

use std::ffi::c_void;
use std::marker::PhantomData;

use super::vtables::game_input_module::CGameInputModuleVTable;
use crate::macros::assert_field_offsets;

/// Узел двусвязного списка action-listener'ов внутри `C_GameInputModule`.
#[repr(C)]
pub struct CInputListenerNode {
    pub next: *mut CInputListenerNode,
    pub prev: *mut CInputListenerNode,
    pub data: *mut c_void,
    pub flags: u16,
}

/// Глобальный объект подсистемы ввода (`C_GameInputModule`).
///
/// Конструктор: `M2DE_GameInputModule_Constructor` (`0x140FD7880`).
/// Vtable: `0x141908AC0` (5 слотов: dtor, get_module_id (=7),
/// get_module_name ("GameInputModule"), register_callbacks, ?).
#[repr(C)]
pub struct CGameInputModule {
    /// `+0x0000` VTable -> `M2DE_VT_CGameInputModule` (`0x141908AC0`).
    pub vtable: *const CGameInputModuleVTable,

    /// `+0x0008` Низкоуровневый `C_SysInput` (мышь/клавиатура/гамепад).
    pub sys_input: *mut c_void,

    /// `+0x0010` Внутренний `C_GameInput`-менеджер (хранит layer'ы и
    /// listener'ы).
    pub game_input: *mut c_void,

    /// `+0x0018..+0x1FE8` Служебное состояние (3 пула listener'ов по
    /// 56*48 байт, локальные params и т. п.). Подробный layout не
    /// разобран, обращаться напрямую запрещено.
    _pad_0018: [u8; 0x1FD0],

    /// `+0x1FE8` Неизвестный bool-флаг (выставлен в 0 в конструкторе).
    pub unk_flag_1fe8: u8,

    _pad_1fe9: [u8; 7],

    /// `+0x1FF0` Первый из трёх `C_InputLayer`-ов.
    pub layer0: *mut c_void,

    /// `+0x1FF8` Второй `C_InputLayer`.
    pub layer1: *mut c_void,

    /// `+0x2000` Третий `C_InputLayer`.
    pub layer2: *mut c_void,

    /// `+0x2008` Главный pause-флаг подсистемы.
    ///
    /// Читается каждым [`Tick`](crate::addresses::functions::input::GAME_INPUT_MODULE_TICK):
    /// если поднят, весь `C_GameInput::Update` пропускается, listener'ы
    /// не получают событий, камера/движение/мышь стоят без накопления
    /// delta. Выставляется через
    /// [`PauseInput`](crate::addresses::functions::input::GAME_INPUT_MODULE_PAUSE_INPUT).
    pub m_b_input_paused: u8,

    /// `+0x2009..+0x207C` Прочие настройки/состояние (sensitivity,
    /// текущий controller-type и т. п.).
    _pad_2009: [u8; 0x73],

    /// `+0x207C` Game-paused флаг (Esc-меню).
    ///
    /// Выставляется callback'ом
    /// [`OnGamePaused`](crate::addresses::functions::callbacks::FIRE_EVENT_BY_ID)
    /// (event 34) и в конце [`PauseInput`] для совместимости.
    /// Сам по себе мышь/камеру не блокирует.
    pub m_b_game_paused: u8,

    _pad_207d: [u8; 0x2B],

    /// `+0x20A8` Sentinel-узел двусвязного списка action-listener'ов.
    pub listeners_sentinel: *mut CInputListenerNode,
}

assert_field_offsets!(CGameInputModule {
    vtable             == 0x0000,
    sys_input          == 0x0008,
    game_input         == 0x0010,
    unk_flag_1fe8      == 0x1FE8,
    layer0             == 0x1FF0,
    layer1             == 0x1FF8,
    layer2             == 0x2000,
    m_b_input_paused   == 0x2008,
    m_b_game_paused    == 0x207C,
    listeners_sentinel == 0x20A8,
});

impl CGameInputModule {
    /// `true`, если поднят `m_b_input_paused`. В этом состоянии Tick
    /// пропускает `C_GameInput::Update` и никакие listener'ы не
    /// получают входных событий.
    #[inline]
    pub fn is_input_paused(&self) -> bool {
        self.m_b_input_paused != 0
    }

    /// `true`, если поднят `m_b_game_paused` (соответствует Esc-меню).
    #[inline]
    pub fn is_game_paused(&self) -> bool {
        self.m_b_game_paused != 0
    }
}

// PhantomData чтобы Rust никогда не считал указатели внутри объекта
// безопасными для авто-Send/Sync (у нас не должно быть пересылки между
// потоками без явной синхронизации со стороны движка).
#[allow(dead_code)]
struct _NotAutoTraits(PhantomData<*mut c_void>);
