//! Высокоуровневый API для `C_SysInput` — низкоуровневая пауза и
//! инвентаризация DI-устройств.
//!
//! Обычно отдельно использовать не требуется: достаточно
//! [`GameInputModule::pause_input`][super::game_input_module::GameInputModule::pause_input],
//! который сам дёрнет [`SysInput::pause`] вместе с reset-ом
//! `C_InputLayer`-ов и Force Feedback. Прямой вызов `SysInput::pause`
//! пригодится в редких сценариях вроде «заморозить мышь, не трогая
//! gameplay-listener'ов» (например, во время кат-сцены или при
//! сериализации save-файла).

use crate::addresses;
use crate::memory::{self, Ptr};
use crate::structures::CSysInput;

use super::base;

/// Сигнатура движковой `M2DE_C_SysInput_Pause` (`0x1407A0680`).
type PauseFn = unsafe extern "C" fn(*mut CSysInput, u8);

/// Обёртка над singleton'ом `C_SysInput`.
#[derive(Debug, Clone, Copy)]
pub struct SysInput {
    ptr: Ptr<CSysInput>,
}

impl SysInput {
    /// Получить singleton. Возвращает `None`, если игра ещё не успела
    /// сконструировать его (mid-injection или до WinMain).
    #[inline]
    pub fn get() -> Option<Self> {
        let addr = unsafe { memory::read_ptr(base() + addresses::globals::SYS_INPUT_INSTANCE)? };
        if !memory::is_valid_ptr(addr) {
            return None;
        }
        Some(Self { ptr: Ptr::new(addr) })
    }

    /// Сырой адрес объекта.
    #[inline]
    pub fn as_ptr(&self) -> usize {
        self.ptr.addr()
    }

    /// Ссылка на структуру.
    ///
    /// # Safety
    /// Объект жив на всё время процесса, но из других потоков может
    /// одновременно идти `Update` — параллельная мутация UB.
    #[inline]
    pub unsafe fn as_ref(&self) -> Option<&CSysInput> {
        unsafe { self.ptr.as_ref() }
    }

    /// `true`, если устройства сейчас на паузе.
    #[inline]
    pub fn is_paused(&self) -> bool {
        unsafe { self.as_ref().map(|s| s.is_paused()).unwrap_or(false) }
    }

    /// Поставить или снять паузу всех зарегистрированных DI-устройств
    /// через движковую `M2DE_C_SysInput_Pause`.
    ///
    /// Внутри функция итерирует RB-tree устройств и для каждого зовёт
    /// `Suspend`/`Resume` через vtable, плюс выставляет
    /// [`CSysInput::m_b_paused`].
    ///
    /// Идемпотентно: если флаг уже в нужном состоянии, движок выходит
    /// сразу.
    pub fn pause(&self, paused: bool) {
        let func: PauseFn = unsafe {
            memory::fn_at(base() + addresses::functions::input::SYS_INPUT_PAUSE)
        };
        unsafe { func(self.ptr.addr() as *mut _, paused as u8) };
    }
}
