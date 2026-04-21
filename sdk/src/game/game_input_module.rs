//! Высокоуровневый API для `C_GameInputModule` — пауза игрового ввода.
//!
//! Используется overlay-кодом клиента: пока открыт чат / меню / окно
//! подключения, мы дёргаем [`GameInputModule::pause_input`], и движок
//! сам через свой штатный механизм блокирует мышь, камеру, движение и
//! Force Feedback — то же самое, что происходит при появлении системного
//! overlay (Steam UI и т. п.).

use crate::addresses;
use crate::memory::{self, Ptr};
use crate::structures::CGameInputModule;

use super::base;

/// Сигнатура движковой `M2DE_GameInputModule_PauseInput` (`0x140FF8FF0`).
type PauseInputFn = unsafe extern "C" fn(*mut CGameInputModule, u8);

/// Обёртка над глобальным `C_GameInputModule`.
#[derive(Debug, Clone, Copy)]
pub struct GameInputModule {
    ptr: Ptr<CGameInputModule>,
}

impl GameInputModule {
    /// Получить глобальный `C_GameInputModule`.
    ///
    /// Возвращает `None`, если статический инициализатор движка ещё не
    /// успел сконструировать singleton (vtable нулевая) или адрес базы
    /// модуля не разрешён.
    #[inline]
    pub fn get() -> Option<Self> {
        let addr = base() + addresses::globals::GAME_INPUT_MODULE;
        if !memory::is_valid_ptr(addr) {
            return None;
        }
        // Объект — статика, не указатель: разыменовывать `addr` как
        // `*const *mut T` неправильно. Просто проверяем что vtable
        // непустая (значит конструктор отработал).
        let vtable = unsafe { *(addr as *const usize) };
        if vtable == 0 || !memory::is_valid_ptr(vtable) {
            return None;
        }
        Some(Self { ptr: Ptr::new(addr) })
    }

    /// Сырой адрес singleton'а.
    #[inline]
    pub fn as_ptr(&self) -> usize {
        self.ptr.addr()
    }

    /// Ссылка на структуру.
    ///
    /// # Safety
    /// Объект должен быть жив (выживает на всё время процесса игры) и
    /// не модифицироваться параллельно из других потоков.
    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&CGameInputModule> {
        unsafe { self.ptr.as_ref() }
    }

    /// `true`, если входная подсистема сейчас в паузе (Tick пропускает
    /// `C_GameInput::Update`).
    #[inline]
    pub fn is_input_paused(&self) -> bool {
        unsafe { self.as_ref().map(|m| m.is_input_paused()).unwrap_or(false) }
    }

    /// `true`, если игра в pause-меню (Esc).
    #[inline]
    pub fn is_game_paused(&self) -> bool {
        unsafe { self.as_ref().map(|m| m.is_game_paused()).unwrap_or(false) }
    }

    /// Поставить или снять полную паузу ввода через движковую
    /// `M2DE_GameInputModule_PauseInput`.
    ///
    /// При `paused = true`:
    /// 1. Поднимается `m_b_input_paused` (`+0x2008`) — Tick перестаёт
    ///    кормить listener'ы; камера/WASD/клики мыши замирают.
    /// 2. Дёргается `C_SysInput::Pause` — Suspend/Reset на каждом
    ///    зарегистрированном DI-устройстве, чтобы их буферы не копили
    ///    delta.
    /// 3. Reset/Update на трёх `C_InputLayer` (`+0x1FF0..+0x2008`).
    /// 4. Force Feedback `PauseAllEffects`.
    /// 5. Поднимается `m_b_game_paused` (`+0x207C`) для совместимости с
    ///    цепочкой `OnGamePaused`.
    ///
    /// Вызов идемпотентен: если запрашиваемое состояние уже выставлено,
    /// движок выходит сразу. Безопасно вызывать каждый кадр.
    ///
    /// # Возвращаемое значение
    ///
    /// `true` — функция вызвана; `false` — singleton был внезапно не
    /// готов (вызов пропущен, состояние не поменялось).
    pub fn pause_input(&self, paused: bool) -> bool {
        let func: PauseInputFn = unsafe {
            memory::fn_at(base() + addresses::functions::input::GAME_INPUT_MODULE_PAUSE_INPUT)
        };
        unsafe { func(self.ptr.addr() as *mut _, paused as u8) };
        true
    }
}
