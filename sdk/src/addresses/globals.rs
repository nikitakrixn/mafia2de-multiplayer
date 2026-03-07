//! Глобальные переменные (RVA).
//!
//! Все указатели — **двойная косвенность**:
//! `module_base + RVA` → `*const *mut T`
//!
//! Чтобы получить объект, нужно дважды разыменовать:
//! ```ignore
//! let ptr_addr = base + rva;
//! let obj: *mut T = *(ptr_addr as *const *mut T);
//! ```

/// `GameManager**` — указатель на менеджер игры.
///
/// IDA: `0x141CAF770` (`qword_141CAF770`)
///
/// Чтение:
/// ```text
/// GameManager* mgr = **(GameManager***)(module_base + GAME_MANAGER);
/// Player* player = *(Player**)(mgr + 0x180);
/// ```
pub const GAME_MANAGER: usize = 0x1CA_F770;

/// `C_GarageManager*` — менеджер гаража.
///
/// IDA: `0x143146A18` (`qword_143146A18`)
///
/// VTable: [`super::vtables::garage::GARAGE_MANAGER`]
pub const GARAGE_MANAGER: usize = 0x314_6A18;

/// `ResourceManager*` — менеджер ресурсов.
///
/// IDA: `0x141CA1FD0` (`qword_141CA1FD0`)
pub const RESOURCE_MANAGER: usize = 0x1CA_1FD0;

/// `GameCallbackManager*` — менеджер игровых событий.
///
/// IDA: `0x141CAF038` (`qword_141CAF038`)
///
/// Управляет 39 зарегистрированными событиями.
/// Вызывается ~494 раза за игровую сессию.
pub const GAME_CALLBACK_MANAGER: usize = 0x1CA_F038;

/// Менеджер объектов.
///
/// IDA: `0x1431360F8`
pub const OBJECT_MANAGER: usize = 0x313_60F8;

/// Аллокатор памяти.
///
/// IDA: `0x141CD4A28`
pub const MEMORY_ALLOCATOR: usize = 0x1CD_4A28;

/// Менеджер моделей машин.
///
/// IDA: `0x141CAE1D8`
pub const CAR_MODEL_MANAGER: usize = 0x1CA_E1D8;

/// Трансформация машины по умолчанию.
///
/// IDA: `0x141CBC0D0`
pub const DEFAULT_VEHICLE_TRANSFORM: usize = 0x1CB_C0D0;

/// Система загрузки ресурсов.
///
/// IDA: `0x141CA52B8`
pub const RESOURCE_LOADER: usize = 0x1CA_52B8;

/// HUD Manager (управление отображением денег, иконок).
///
/// IDA: `qword_143138FA8`
/// Доступ: `sub_140D01600()` возвращает этот глобал.
///
/// `+0x98` → Money display component (для popup)
pub const HUD_MANAGER: usize = 0x313_8FA8;

/// Notify Manager (система нотификаций).
///
/// IDA: `qword_141CABBE0`
pub const NOTIFY_MANAGER: usize = 0x1CA_BBE0;

/// Stats Tracker 1 (отслеживание доходов).
///
/// IDA: `qword_1431464A0`
pub const STATS_TRACKER_1: usize = 0x314_64A0;

/// Stats Tracker 2 (отслеживание расходов).
///
/// IDA: `qword_143140BD0`
pub const STATS_TRACKER_2: usize = 0x314_0BD0;