//! VTable `C_SysInput` — 16 слотов.
//!
//! Адрес в `.rdata`: финальная — `0x141895C00` (полное имя
//! `M2DE_VT_CSysInput`); промежуточная (base-class) — `0x141895820`,
//! ставится в начале `M2DE_C_SysInput_CreateInstance` и сразу
//! перезаписывается финальной.
//!
//! `C_SysInput` сам по себе **не наследует** `C_TickedModule` (его не
//! тикает `GameCallbackManager`). Update вызывается из других мест —
//! в частности из `C_GameInput::Update` (см. `Tick` цепочку
//! `C_GameInputModule`).
//!
//! Сейчас известны только 4 слота из 16 — остальные перенесены в IDA
//! как `M2DE_C_SysInput_VT_SlotN` (placeholder-имена) и пока не
//! разбирался, потому что в overlay pause хватает
//! одного `Pause` (НЕ vtable-метод, отдельная функция
//! `M2DE_C_SysInput_Pause`).
//!
//! | Слот | Имя | RVA | Описание |
//! |:---:|:---|:---|:---|
//! | 0  | `dtor`           | `0x14079B700` | Деструктор |
//! | 1  | `init`           | `0x14079FA60` | DirectInput8 + CreateDevices |
//! | 2  | `slot2`          | `0x14079BA50` | TBD |
//! | 3  | `slot3`          | `0x14079CC70` | TBD |
//! | 4  | `slot4`          | `0x1407A46C0` | TBD |
//! | 5  | `slot5`          | `0x14079CE90` | TBD |
//! | 6  | `slot6`          | `0x1407A0B20` | TBD |
//! | 7  | `slot7`          | `0x14079D6A0` | TBD |
//! | 8  | `slot8`          | `0x14079D2C0` | TBD |
//! | 9  | `slot9`          | `0x14079D340` | TBD |
//! | 10 | `slot10`         | `0x14079D440` | TBD |
//! | 11 | `slot11`         | `0x1407A3AB0` | TBD |
//! | 12 | `slot12`         | `0x14079CF60` | TBD |
//! | 13 | `slot13`         | `0x1407A0DD0` | TBD |
//! | 14 | `slot14`         | `0x1407A3860` | TBD |
//! | 15 | `slot15`         | `0x1407A0100` | TBD |

use std::ffi::c_void;

type FnDtor = unsafe extern "system" fn(this: *mut c_void, flags: u8);
type FnInit = unsafe extern "system" fn(this: *mut c_void, hwnd_hint: *mut c_void) -> u8;
type FnUnknown = unsafe extern "system" fn(this: *mut c_void) -> u64;

/// VTable `C_SysInput` — финальная (`M2DE_VT_CSysInput` @ `0x141895C00`).
///
/// Размер 16 слотов = 128 байт. Слоты 2..15 пока без сигнатур, доступны
/// для дальнейшего реверса под именами `slotN`.
#[repr(C)]
pub struct CSysInputVTable {
    /// `[0]` Деструктор.
    pub dtor: FnDtor,
    /// `[1]` `Init(HWND wnd_hint) -> bool`. Создаёт `IDirectInput8` и
    /// все DI-устройства.
    pub init: FnInit,
    pub slot2: FnUnknown,
    pub slot3: FnUnknown,
    pub slot4: FnUnknown,
    pub slot5: FnUnknown,
    pub slot6: FnUnknown,
    pub slot7: FnUnknown,
    pub slot8: FnUnknown,
    pub slot9: FnUnknown,
    pub slot10: FnUnknown,
    pub slot11: FnUnknown,
    pub slot12: FnUnknown,
    pub slot13: FnUnknown,
    pub slot14: FnUnknown,
    pub slot15: FnUnknown,
}

const _: () = {
    assert!(std::mem::size_of::<CSysInputVTable>() == 16 * 8);
    assert!(std::mem::offset_of!(CSysInputVTable, dtor) == 0 * 8);
    assert!(std::mem::offset_of!(CSysInputVTable, init) == 1 * 8);
    assert!(std::mem::offset_of!(CSysInputVTable, slot15) == 15 * 8);
};
