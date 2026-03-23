//! Высокоуровневый API игрока.
//!
//! Player — обёртка над CPlayer*, скрывающая цепочки указателей
//! и вызовы функций движка. Через неё клиент работает с позицией,
//! деньгами, оружием и управлением.
//!
//! ВАЖНО: Player содержит сырой указатель на объект движка.
//! Движок не потокобезопасен — использовать Player можно только
//! в том потоке, где он был получен (обычно главный поток игры).

use std::ffi::{CStr, CString};
use std::time::{Duration, Instant};

use super::base;
use super::entity_types::{EntityType, FactoryType};
use crate::addresses;
use crate::addresses::constants;
use crate::addresses::fields;
use crate::memory;
use common::logger;

/// Обёртка над указателем на CPlayer в памяти игры.
///
/// Не реализует Send — указатель валиден только
/// в потоке, где был получен. Для передачи между потоками
/// используй `as_ptr()` и пересоздавай Player на месте.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    /// Указатель на CPlayer в памяти игры.
    ptr: usize,
}

/// 3D позиция в игровом мире.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}

impl Player {
    // =============================================================================
    //  Получение указателя на игрока
    // =============================================================================

    /// Получить активного игрока через GameManager.
    ///
    /// Возвращает None если:
    /// - GameManager ещё не создан (ранний старт)
    /// - указатель на игрока NULL (меню, загрузка)
    pub fn get_active() -> Option<Self> {
        unsafe {
            let mgr = memory::read_ptr(base() + addresses::globals::GAME_MANAGER)?;
            let player = memory::read_ptr(mgr + fields::game_manager::ACTIVE_PLAYER)?;
            Some(Self { ptr: player })
        }
    }

    /// Ждёт появления и полной инициализации игрока.
    ///
    /// Используется при старте клиента — игроку нужно время
    /// чтобы загрузился мир, создался инвентарь и т.д.
    /// Опрашивает каждые 500мс до таймаута.
    pub fn wait_until_ready(timeout_secs: u64) -> Option<Self> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let mut reported = false;

        loop {
            if let Some(player) = Self::get_active() {
                if !reported {
                    logger::info(&format!(
                        "Указатель на игрока: 0x{:X}, жду инвентарь...",
                        player.ptr,
                    ));
                    reported = true;
                }
                if player.is_ready() {
                    logger::info("Игрок полностью инициализирован");
                    return Some(player);
                }
            }
            if Instant::now() > deadline {
                logger::error(if reported {
                    "Таймаут: инвентарь так и не появился"
                } else {
                    "Таймаут: указатель на игрока так и не появился"
                });
                return None;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    // =============================================================================
    //  Базовые аксессоры
    // =============================================================================

    /// Сырой указатель на C_Human. Для сравнений и передачи между потоками.
    pub fn as_ptr(&self) -> usize {
        self.ptr
    }

    /// Готов ли игрок к работе (есть инвентарь).
    pub fn is_ready(&self) -> bool {
        self.inventory_ptr().is_some()
    }

    /// Указатель на Inventory*. NULL на ранних стадиях загрузки.
    pub fn inventory_ptr(&self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + fields::player::INVENTORY) }
    }

    /// Проверяет что кошелёк полностью инициализирован —
    /// слот денег существует и внутренний вектор не пуст.
    pub fn is_wallet_ready(&self) -> bool {
        self.resolve_money_address().is_some()
    }

    /// Указатель на компонент управления (блокировка, стиль боя).
    pub fn control_component_ptr(&self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + fields::player::CONTROL_COMPONENT) }
    }

    /// Packed table_id игрока.
    pub fn table_id(&self) -> Option<u32> {
        unsafe { memory::read_value::<u32>(self.ptr + fields::player::TABLE_ID) }
    }

    /// Factory type byte из packed table_id.
    pub fn factory_type_byte(&self) -> Option<u8> {
        self.table_id().map(|tid| (tid & 0xFF) as u8)
    }

    /// Typed factory type.
    pub fn factory_type(&self) -> Option<FactoryType> {
        FactoryType::from_byte(self.factory_type_byte()?)
    }

    /// Lua-facing entity type.
    pub fn entity_type(&self) -> Option<EntityType> {
        let ft = self.factory_type()?;
        EntityType::from_factory_type(ft as u8)
    }

    /// Name hash (`entity + 0x30`).
    pub fn name_hash(&self) -> Option<u64> {
        unsafe { memory::read_value::<u64>(self.ptr + addresses::fields::entity::NAME_HASH) }
    }

    // =============================================================================
    //  Деньги — чтение (прямое чтение памяти)
    // =============================================================================

    /// Прочитать текущий баланс в центах.
    /// $600.00 = 60000 центов.
    pub fn get_money_cents(&self) -> Option<i64> {
        let addr = self.resolve_money_address()?;
        unsafe { memory::read_value::<i64>(addr) }
    }

    /// Баланс в центах, 0 если кошелёк не готов.
    pub fn get_money_cents_or_zero(&self) -> i64 {
        self.get_money_cents().unwrap_or(0)
    }

    /// Баланс в долларах (целая часть).
    pub fn get_money_dollars(&self) -> Option<i32> {
        self.get_money_cents().map(|c| (c / 100) as i32)
    }

    /// Баланс в формате "$ 600.00".
    pub fn get_money_formatted(&self) -> Option<String> {
        self.get_money_cents().map(|c| {
            let d = c / 100;
            let r = (c % 100).abs();
            format!("$ {d}.{r:02}")
        })
    }

    // =============================================================================
    //  Деньги — запись (прямая запись в память)
    // =============================================================================

    /// Установить баланс напрямую в память.
    /// Без HUD-уведомления, без обновления статистики.
    pub fn set_money(&self, cents: i64) -> bool {
        unsafe {
            match self.resolve_money_address() {
                Some(addr) => {
                    std::ptr::write(addr as *mut i64, cents);
                    true
                }
                None => {
                    logger::error("set_money: кошелёк не инициализирован");
                    false
                }
            }
        }
    }

    /// Установить баланс в долларах.
    pub fn set_money_dollars(&self, dollars: i32) -> bool {
        self.set_money(dollars as i64 * 100)
    }

    /// Прибавить/вычесть центы напрямую в памяти.
    pub fn add_money(&self, delta_cents: i64) -> Option<i64> {
        let current = self.get_money_cents()?;
        let new_amount = current + delta_cents;
        if self.set_money(new_amount) {
            Some(new_amount)
        } else {
            None
        }
    }

    /// Прибавить/вычесть доллары напрямую в памяти.
    pub fn add_money_dollars(&self, dollars: i32) -> Option<i64> {
        self.add_money(dollars as i64 * 100)
    }

    // =============================================================================
    //  Деньги — через игровые функции
    // =============================================================================

    /// Добавить деньги через функцию движка (тихо, без HUD).
    ///
    /// Вызывает M2DE_Inventory_ModifyMoney(inv, cents, do_apply=1).
    /// Надёжнее прямой записи — обновляет внутренние счётчики.
    pub fn add_money_game(&self, cents: i64) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_money_game: инвентарь NULL");
            return false;
        };

        type ModifyMoneyFn = unsafe extern "C" fn(usize, i64, u8) -> u8;
        let func: ModifyMoneyFn =
            unsafe { memory::fn_at(base() + addresses::functions::player::INVENTORY_MODIFY_MONEY) };

        unsafe { func(inv, cents, 1) };
        true
    }

    /// Показать HUD popup о деньгах (± $X.XX).
    ///
    /// Не добавляет деньги — только визуальный эффект.
    /// Обходит проверку entity type (inv.type != 16)
    /// через прямой доступ к g_HUDManager.
    fn show_money_notification(&self, cents: i64) {
        unsafe {
            // g_HUDManager -> +0x98 -> money display component
            let hud_mgr = match memory::read_ptr(base() + addresses::globals::HUD_MANAGER) {
                Some(p) => p,
                None => {
                    logger::debug("show_money_notification: HUD-менеджер не готов");
                    return;
                }
            };

            let money_display = match memory::read_ptr(hud_mgr + fields::hud_manager::MONEY_DISPLAY)
            {
                Some(p) => p,
                None => {
                    logger::debug("show_money_notification: компонент денег NULL");
                    return;
                }
            };

            // Сбросить таймер анимации — иначе popup не покажется
            // если предыдущий ещё не догорел
            std::ptr::write(
                (money_display + fields::hud_money_display::ANIM_TIMER) as *mut f32,
                0.0f32,
            );

            type UpdateFn = unsafe extern "C" fn(usize, i64, i64) -> i64;
            let update: UpdateFn =
                memory::fn_at(base() + addresses::functions::hud::UPDATE_MONEY_COUNTER);

            update(money_display, cents, 0);
        }
    }

    /// Добавить деньги + показать HUD уведомление.
    ///
    /// Двухэтапный процесс:
    /// 1. Реально добавить через игровую функцию
    /// 2. Показать popup через g_HUDManager
    ///
    /// Popup обходит проверку type==16 — без этого у игрока
    /// (type=0) уведомление просто не покажется.
    pub fn add_money_with_hud(&self, cents: i64) -> bool {
        if !self.add_money_game(cents) {
            return false;
        }
        self.show_money_notification(cents);
        true
    }

    /// Добавить деньги с HUD (в долларах).
    pub fn add_money_dollars_with_hud(&self, dollars: i32) -> bool {
        self.add_money_with_hud(dollars as i64 * 100)
    }

    // =============================================================================
    //  Позиция
    // =============================================================================

    /// Получить мировую позицию через функцию движка.
    ///
    /// Это основной способ — повторяет "официальный" путь самой игры:
    /// - сначала пробует physics/provider (C_Human + 0x258)
    /// - если нет — fallback на frame node (C_Human + 0x78)
    ///
    /// Подтверждено: IDA sub_140DA7630, runtime сверка с Lua GetPos().
    pub fn get_position(&self) -> Option<Vec3> {
        unsafe {
            let mut out = Vec3::default();

            type GetPosFn = unsafe extern "C" fn(usize, *mut Vec3) -> *mut Vec3;
            let func: GetPosFn = memory::fn_at(base() + addresses::functions::entity::GET_POS);

            let ret = func(self.ptr, &mut out as *mut Vec3);
            if ret.is_null() {
                return None;
            }

            // Проверяем на NaN/Inf — бывает на ранних стадиях загрузки
            if out.x.is_finite() && out.y.is_finite() && out.z.is_finite() {
                Some(out)
            } else {
                None
            }
        }
    }

    /// Прямое чтение позиции из frame/transform node.
    ///
    /// Debug/fallback-метод для сверки реверса.
    /// Читает те же поля, что и M2DE_Entity_GetPos в fallback-path:
    /// frame+0x64 (X), frame+0x74 (Y), frame+0x84 (Z).
    pub fn get_position_from_frame(&self) -> Option<Vec3> {
        unsafe {
            let frame = memory::read_ptr_raw(self.ptr + fields::player::FRAME_NODE)?;
            if frame == 0 || !memory::is_valid_ptr(frame) {
                return None;
            }

            let x = memory::read_value::<f32>(frame + fields::entity_frame::POS_X)?;
            let y = memory::read_value::<f32>(frame + fields::entity_frame::POS_Y)?;
            let z = memory::read_value::<f32>(frame + fields::entity_frame::POS_Z)?;

            if x.is_finite() && y.is_finite() && z.is_finite() {
                Some(Vec3 { x, y, z })
            } else {
                None
            }
        }
    }

    /// Установить мировую позицию через функцию движка.
    ///
    /// Не пишем напрямую в frame node — игра хранит позицию
    /// не только там. M2DE_Entity_SetPos обновляет и physics,
    /// и cache, и dirty-флаги.
    pub fn set_position(&self, pos: &Vec3) -> bool {
        if !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
            logger::error("set_position: координата NaN или Inf");
            return false;
        }

        unsafe {
            type SetPosFn = unsafe extern "C" fn(usize, *const Vec3);
            let func: SetPosFn = memory::fn_at(base() + addresses::functions::entity::SET_POS);

            func(self.ptr, pos as *const Vec3);
            true
        }
    }

    // =============================================================================
    //  Оружие
    // =============================================================================

    /// Добавить оружие с патронами.
    ///
    /// Если оружие уже есть — добавятся только патроны.
    /// Патроны ограничиваются максимумом из weapons.tbl.
    ///
    /// ```ignore
    /// use sdk::addresses::constants::weapons;
    /// player.add_weapon(weapons::THOMPSON_1928, 200);
    /// ```
    pub fn add_weapon(&self, weapon_id: u32, ammo: u32) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_weapon: инвентарь NULL");
            return false;
        };

        type AddWeaponFn = unsafe extern "C" fn(usize, u32, i32) -> u8;
        let func: AddWeaponFn = unsafe {
            memory::fn_at(base() + addresses::functions::player::INVENTORY_ADD_WEAPON_CORE)
        };

        logger::debug(&format!(
            "M2DE_Inventory_AddWeapon_Core(0x{:X}, weapon={}, ammo={})",
            inv, weapon_id, ammo,
        ));

        let result = unsafe { func(inv, weapon_id, ammo as i32) };
        if result != 0 {
            logger::debug("  -> оружие добавлено / патроны обновлены");
            true
        } else {
            logger::warn("  -> add_weapon вернул 0 (слоты заняты или невалидный ID)");
            false
        }
    }

    /// Добавить только патроны (без оружия).
    ///
    /// Патроны попадают в slot[4]. Если оружия пока нет —
    /// не упадёт, но патроны будут "висеть" до получения.
    pub fn add_ammo(&self, weapon_id: u32, ammo: u32) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_ammo: инвентарь NULL");
            return false;
        };

        type AddAmmoFn = unsafe extern "C" fn(usize, u32, u32);
        let func: AddAmmoFn =
            unsafe { memory::fn_at(base() + addresses::functions::player::INVENTORY_ADD_AMMO) };

        unsafe { func(inv, weapon_id, ammo) };
        true
    }

    // =============================================================================
    //  Патроны
    // =============================================================================

    /// Включить/выключить бесконечные патроны.
    pub fn set_unlimited_ammo(&self, enabled: bool) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("set_unlimited_ammo: инвентарь NULL");
            return false;
        };
        unsafe { memory::write_value(inv + fields::inventory::UNLIMITED_AMMO, enabled as u8) }
    }

    /// Проверить, включены ли бесконечные патроны.
    pub fn is_unlimited_ammo(&self) -> Option<bool> {
        let inv = self.inventory_ptr()?;
        unsafe { memory::read_value::<u8>(inv + fields::inventory::UNLIMITED_AMMO).map(|v| v != 0) }
    }

    // =============================================================================
    //  Оружие в руках
    // =============================================================================

    /// Проверить, есть ли оружие в руках.
    pub fn has_weapon_in_hand(&self) -> Option<bool> {
        unsafe {
            let ws = memory::read_ptr(self.ptr + fields::player::WEAPON_STATE_COMPONENT)?;
            let weapon_data = memory::read_ptr_raw(ws + fields::weapon_state::CURRENT_WEAPON_DATA)?;
            Some(weapon_data != 0)
        }
    }

    /// Получить ID оружия в руках (None = руки пусты).
    pub fn get_weapon_in_hand_id(&self) -> Option<u32> {
        unsafe {
            let ws = memory::read_ptr(self.ptr + fields::player::WEAPON_STATE_COMPONENT)?;
            let weapon_data = memory::read_ptr(ws + fields::weapon_state::CURRENT_WEAPON_DATA)?;
            memory::read_value::<u32>(weapon_data + fields::weapon_data::WEAPON_ID)
        }
    }

    /// Проверить, огнестрельное ли оружие в руках.
    pub fn has_fire_weapon_in_hand(&self) -> Option<bool> {
        unsafe {
            let ws = memory::read_ptr(self.ptr + fields::player::WEAPON_STATE_COMPONENT)?;
            let weapon_data = memory::read_ptr(ws + fields::weapon_state::CURRENT_WEAPON_DATA)?;
            let flags = memory::read_value::<u32>(weapon_data + fields::weapon_data::WEAPON_FLAGS)?;
            Some((flags & fields::weapon_type_flags::FIRE_WEAPON) != 0)
        }
    }

    /// Получить текущие патроны в обойме оружия в руках.
    pub fn get_current_ammo(&self) -> Option<i32> {
        use crate::addresses::fields::{value_container, value_store};
        unsafe {
            let ws = memory::read_ptr(self.ptr + fields::player::WEAPON_STATE_COMPONENT)?;
            let weapon_data = memory::read_ptr(ws + fields::weapon_state::CURRENT_WEAPON_DATA)?;
            let container = memory::read_ptr(weapon_data + fields::weapon_data::AMMO_CONTAINER)?;
            let store = memory::read_ptr(container + value_container::STORE_PTR)?;
            memory::read_value::<i32>(store + value_store::VALUE)
        }
    }

    /// Проверить, холодное ли оружие в руках (нож, бита).
    pub fn has_cold_weapon_in_hand(&self) -> Option<bool> {
        unsafe {
            let ws = memory::read_ptr(self.ptr + fields::player::WEAPON_STATE_COMPONENT)?;
            let wd = memory::read_ptr(ws + fields::weapon_state::CURRENT_WEAPON_DATA)?;
            let flags = memory::read_value::<u32>(wd + fields::weapon_data::WEAPON_FLAGS)?;
            Some((flags & fields::weapon_type_flags::COLD_WEAPON) != 0)
        }
    }

    /// Проверить, метательное ли оружие в руках (граната, молотов).
    pub fn has_throwing_weapon_in_hand(&self) -> Option<bool> {
        unsafe {
            let ws = memory::read_ptr(self.ptr + fields::player::WEAPON_STATE_COMPONENT)?;
            let wd = memory::read_ptr(ws + fields::weapon_state::CURRENT_WEAPON_DATA)?;
            let flags = memory::read_value::<u32>(wd + fields::weapon_data::WEAPON_FLAGS)?;
            Some((flags & fields::weapon_type_flags::THROWING_WEAPON) != 0)
        }
    }

    /// Проверить, пусты ли руки.
    pub fn has_empty_hands(&self) -> Option<bool> {
        self.has_weapon_in_hand().map(|has| !has)
    }

    /// Получить тип оружия в руках как строку (для отладки).
    pub fn get_weapon_type_str(&self) -> &'static str {
        if self.has_fire_weapon_in_hand().unwrap_or(false) {
            "огнестрельное"
        } else if self.has_cold_weapon_in_hand().unwrap_or(false) {
            "холодное"
        } else if self.has_throwing_weapon_in_hand().unwrap_or(false) {
            "метательное"
        } else if self.has_weapon_in_hand().unwrap_or(false) {
            "неизвестное"
        } else {
            "пусто"
        }
    }

    // =============================================================================
    //  Physics State
    // =============================================================================

    /// Получить текущее состояние физики.
    pub fn get_phys_state(&self) -> Option<u32> {
        unsafe {
            let physics = memory::read_ptr(self.ptr + fields::player::PHYSICS_PROVIDER)?;
            let vtable = memory::read_ptr(physics)?;
            let func_ptr = memory::read_ptr(vtable + 53 * 8)?;
            type GetStateFn = unsafe extern "C" fn(usize) -> u32;
            let func: GetStateFn = std::mem::transmute(func_ptr);
            Some(func(physics))
        }
    }

    /// Установить состояние физики.
    /// Безопасный путь через движковую функцию.
    pub fn set_phys_state(&self, state: u32) -> bool {
        let Some(prop_acc) =
            (unsafe { memory::read_ptr(self.ptr + fields::player::CONTROL_COMPONENT) })
        else {
            return false;
        };

        type SetPhysStateFn = unsafe extern "C" fn(usize, u32) -> u64;
        let func: SetPhysStateFn =
            unsafe { memory::fn_at(base() + addresses::functions::physics::SET_PHYS_STATE) };

        unsafe { func(prop_acc, state) };
        true
    }

    // =============================================================================
    //  Direction basis from frame matrix
    // =============================================================================

    /// Получить forward-вектор персонажа (куда смотрит, в плоскости XY).
    pub fn get_forward(&self) -> Option<Vec3> {
        unsafe {
            let frame = memory::read_ptr_raw(self.ptr + fields::player::FRAME_NODE)?;
            if frame == 0 {
                return None;
            }
            // Col1 = Forward
            let x = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_X)?;
            let y = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_Y)?;
            let z = memory::read_value::<f32>(frame + fields::entity_frame::FORWARD_Z)?;
            if x.is_finite() && y.is_finite() && z.is_finite() {
                Some(Vec3 { x, y, z })
            } else {
                None
            }
        }
    }

    /// Получить right-вектор персонажа (направление вправо).
    pub fn get_right(&self) -> Option<Vec3> {
        unsafe {
            let frame = memory::read_ptr_raw(self.ptr + fields::player::FRAME_NODE)?;
            if frame == 0 {
                return None;
            }
            // Col0 = Right
            let x = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_X)?;
            let y = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_Y)?;
            let z = memory::read_value::<f32>(frame + fields::entity_frame::RIGHT_Z)?;
            if x.is_finite() && y.is_finite() && z.is_finite() {
                Some(Vec3 { x, y, z })
            } else {
                None
            }
        }
    }

    // =============================================================================
    //  Управление (блокировка, стиль)
    // =============================================================================

    /// Заблокировано ли управление игроком.
    pub fn are_controls_locked(&self) -> Option<bool> {
        let control = self.control_component_ptr()?;

        type Fn = unsafe extern "C" fn(usize) -> i64;
        let func: Fn =
            unsafe { memory::fn_at(base() + addresses::functions::player_control::IS_LOCKED) };

        Some(unsafe { func(control) != 0 })
    }

    /// Заблокировать / разблокировать управление.
    pub fn lock_controls(&self, locked: bool) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls: компонент управления NULL");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn =
            unsafe { memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED) };

        unsafe { func(control, locked as u8, 0) };
        true
    }

    /// Заблокировать управление для проигрывания анимации.
    ///
    /// Отличается от обычной блокировки вторым флагом (play_anim=1),
    /// который позволяет движку доиграть текущую анимацию.
    pub fn lock_controls_to_play_anim(&self) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls_to_play_anim: компонент управления NULL");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn =
            unsafe { memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED) };

        unsafe { func(control, 1, 1) };
        true
    }

    /// Принудительно заблокировать/разблокировать управление.
    ///
    /// В отличие от `lock_controls()`, эта функция вызывает внутреннюю
    /// функцию блокировки напрямую, минуя проверку текущего состояния.
    /// Используется когда игра уже заблокировала управление (например, пауза),
    /// но нам нужно заблокировать ещё и ввод клавиатуры.
    pub fn lock_controls_force(&self, locked: bool) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls_force: компонент управления NULL");
            return false;
        };

        // control - это указатель на PlayerControlRef
        // Нам нужно прочитать *control (разыменовать), а затем +248
        // v3 = *a1;
        // v5 = *(_QWORD *)(*a1 + 248);

        let control_ref = unsafe {
            match memory::read_ptr(control) {
                Some(ptr) => ptr,
                None => {
                    logger::error("lock_controls_force: не удалось разыменовать control");
                    return false;
                }
            }
        };

        let internal_ptr = unsafe {
            match memory::read_ptr(control_ref + 248) {
                Some(ptr) => ptr,
                None => {
                    logger::error(&format!(
                        "lock_controls_force: не удалось прочитать internal_ptr (control_ref=0x{:X})",
                        control_ref
                    ));
                    return false;
                }
            }
        };

        // Вызываем sub_140DB1BE0(internal_ptr + 112, locked, 0)
        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED_INTERNAL)
        };

        unsafe { func(internal_ptr + 112, locked as u8, 0) };

        logger::debug(&format!(
            "[player] lock_controls_force({}) called successfully",
            locked
        ));
        true
    }

    /// Получить текущий стиль управления как строку.
    ///
    /// Примеры: "Normal", "DoNothing", "Intoxicated" и т.д.
    pub fn get_control_style_str(&self) -> Option<String> {
        let control = self.control_component_ptr()?;

        type Fn = unsafe extern "C" fn(usize) -> *const i8;
        let func: Fn =
            unsafe { memory::fn_at(base() + addresses::functions::player_control::GET_STYLE_STR) };

        let ptr = unsafe { func(control) };
        if ptr.is_null() {
            return None;
        }

        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }

    /// Установить стиль управления по имени.
    pub fn set_control_style_str(&self, style: &str) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("set_control_style_str: компонент управления NULL");
            return false;
        };

        let Ok(c_style) = CString::new(style) else {
            logger::error("set_control_style_str: строка содержит NUL-байт");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, *const i8) -> i64;
        let func: Fn =
            unsafe { memory::fn_at(base() + addresses::functions::player_control::SET_STYLE_STR) };

        unsafe { func(control, c_style.as_ptr()) != 0 }
    }

    // =============================================================================
    //  Внутренние хелперы
    // =============================================================================

    /// Полная цепочка указателей до ячейки с деньгами.
    ///
    /// Путь:
    /// C_Human -> Inventory -> slots_start -> slot[5] ->
    /// -> vec_begin -> money_item -> wallet.inner ->
    /// -> money_container -> value (i64 центы)
    ///
    /// Любой указатель в цепочке может быть NULL
    /// на ранних стадиях загрузки.
    fn resolve_money_address(&self) -> Option<usize> {
        unsafe {
            let inv = memory::read_ptr(self.ptr + fields::player::INVENTORY)?;
            let slots_base = memory::read_ptr(inv + fields::inventory::SLOTS_START)?;
            let slot5 = memory::read_ptr(slots_base + constants::slots::MONEY * 8)?;

            // Внутренний вектор слота
            let vec_begin = memory::read_ptr_raw(slot5 + fields::slot::VEC_BEGIN)?;
            let vec_end = memory::read_ptr_raw(slot5 + fields::slot::VEC_END)?;

            if vec_begin == 0 || vec_end <= vec_begin || !memory::is_valid_ptr(vec_begin) {
                return None;
            }

            // Цепочка до самого значения
            let money_item = memory::read_ptr(vec_begin)?;
            let inner = memory::read_ptr(money_item + fields::wallet::INNER_STRUCT)?;
            let container = memory::read_ptr(inner + fields::wallet_inner::MONEY_CONTAINER_PTR)?;

            let addr = container + fields::money_container::VALUE;
            if memory::is_valid_ptr(addr) {
                Some(addr)
            } else {
                None
            }
        }
    }

    // =============================================================================
    //  Здоровье
    // =============================================================================

    /// Получить текущее здоровье.
    /// 720.0 = полное на нормальной сложности.
    pub fn get_health(&self) -> Option<f32> {
        unsafe { memory::read_value::<f32>(self.ptr + fields::player::CURRENT_HEALTH) }
    }

    /// Установить текущее здоровье напрямую.
    ///
    /// Если поставить <= 0.0 — персонаж НЕ умрёт автоматически,
    /// смерть тригерится только из кода урона. Для убийства
    /// используй Lua `healthrel = 0`.
    pub fn set_health(&self, value: f32) -> bool {
        unsafe { memory::write_value(self.ptr + fields::player::CURRENT_HEALTH, value) }
    }

    /// Получить максимум здоровья игрока.
    /// Читает из глобальной структуры g_M2DE_PlayerData+0x00.
    /// НЕ из entity — у игрока healthmax хранится отдельно.
    pub fn get_health_max(&self) -> Option<f32> {
        unsafe {
            let player_data = base() + addresses::globals::PLAYER_DATA;
            memory::read_value::<f32>(player_data)
        }
    }

    /// Установить максимум здоровья игрока.
    /// Пишет в g_M2DE_PlayerData+0x00.
    pub fn set_health_max(&self, value: f32) -> bool {
        unsafe {
            let player_data = base() + addresses::globals::PLAYER_DATA;
            memory::write_value(player_data, value)
        }
    }

    /// Полностью восстановить здоровье до максимума.
    pub fn heal_full(&self) -> bool {
        match self.get_health_max() {
            Some(max_hp) => self.set_health(max_hp),
            None => {
                // Фоллбек: на нормальной сложности максимум 720
                logger::warn("heal_full: не удалось прочитать healthmax, ставлю 720.0");
                self.set_health(720.0)
            }
        }
    }

    /// Добавить здоровье (не выше максимума).
    pub fn add_health(&self, amount: f32) -> Option<f32> {
        let current = self.get_health()?;
        let max_hp = self.get_health_max().unwrap_or(720.0);
        let new_hp = (current + amount).min(max_hp).max(0.0);
        self.set_health(new_hp);
        Some(new_hp)
    }

    /// Получить здоровье в процентах (0.0 — 100.0).
    pub fn get_health_percent(&self) -> Option<f32> {
        let current = self.get_health()?;
        let max_hp = self.get_health_max().unwrap_or(720.0);
        if max_hp > 0.0 {
            Some((current / max_hp) * 100.0)
        } else {
            Some(0.0)
        }
    }

    // =============================================================================
    //  Флаги: неуязвимость, полубог, смерть
    // =============================================================================

    /// Жив ли игрок.
    pub fn is_alive(&self) -> Option<bool> {
        unsafe { memory::read_value::<u8>(self.ptr + fields::player::IS_DEAD).map(|v| v == 0) }
    }

    /// Проверить флаг неуязвимости.
    pub fn is_invulnerable(&self) -> Option<bool> {
        unsafe {
            memory::read_value::<u8>(self.ptr + fields::player::INVULNERABILITY).map(|v| v != 0)
        }
    }

    /// Установить/снять неуязвимость.
    /// При включённой неуязвимости весь урон пропускается.
    pub fn set_invulnerable(&self, enabled: bool) -> bool {
        unsafe { memory::write_value(self.ptr + fields::player::INVULNERABILITY, enabled as u8) }
    }

    /// Проверить режим полубога.
    pub fn is_demigod(&self) -> Option<bool> {
        unsafe { memory::read_value::<u8>(self.ptr + fields::player::DEMIGOD).map(|v| v != 0) }
    }

    /// Установить/снять режим полубога.
    /// При полубоге здоровье не падает ниже 1.0 — персонаж
    /// получает урон, но не умирает.
    pub fn set_demigod(&self, enabled: bool) -> bool {
        unsafe { memory::write_value(self.ptr + fields::player::DEMIGOD, enabled as u8) }
    }

    // =============================================================================
    //  God Mode (комбинированный)
    // =============================================================================

    /// Включить/выключить режим бога.
    ///
    /// Включает:
    /// 1. Неуязвимость (пропускает урон полностью)
    /// 2. Полубог (запасной — если неуязвимость сбросится)
    /// 3. Полное восстановление здоровья
    pub fn set_god_mode(&self, enabled: bool) -> bool {
        let ok1 = self.set_invulnerable(enabled);
        let ok2 = self.set_demigod(enabled);

        if enabled {
            self.heal_full();
        }

        if ok1 && ok2 {
            logger::info(&format!(
                "God Mode: {}",
                if enabled {
                    "ВКЛЮЧЁН"
                } else {
                    "ВЫКЛЮЧЕН"
                }
            ));
            true
        } else {
            logger::error("God Mode: не удалось записать флаги");
            false
        }
    }

    /// Проверить, активен ли God Mode.
    pub fn is_god_mode(&self) -> bool {
        self.is_invulnerable().unwrap_or(false) && self.is_demigod().unwrap_or(false)
    }

    // =============================================================================
    //  Транспорт
    // =============================================================================

    /// Проверить, находится ли игрок в транспорте.
    pub fn is_in_vehicle(&self) -> Option<bool> {
        unsafe { memory::read_ptr_raw(self.ptr + fields::player::OWNER).map(|owner| owner != 0) }
    }

    /// Получить указатель на текущий транспорт (или None если пешком).
    pub fn get_vehicle_ptr(&self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + fields::player::OWNER) }
    }

    // =============================================================================
    //  Диагностика
    // =============================================================================

    /// Вывести в лог подробную информацию об игроке.
    /// Полезно при отладке — сразу видно состояние инвентаря,
    /// тип entity, количество слотов и баланс.
    pub fn log_debug_info(&self) {
        logger::debug(&format!("Player ptr: 0x{:X} (C_Human*)", self.ptr));

        match self.inventory_ptr() {
            Some(inv) => {
                logger::debug(&format!("  Инвентарь: 0x{inv:X}"));
                unsafe {
                    // Тип инвентаря (0 = игрок, 16 = NPC)
                    if let Some(t) = memory::read_value::<u8>(inv + fields::inventory::TYPE) {
                        logger::debug(&format!("  Тип инвентаря: {t}"));
                    }

                    // Количество слотов
                    if let Some(start) = memory::read_ptr(inv + fields::inventory::SLOTS_START) {
                        let end = memory::read_ptr(inv + fields::inventory::SLOTS_END).unwrap_or(0);
                        let count = if end > start { (end - start) / 8 } else { 0 };
                        logger::debug(&format!("  Слотов: {count}"));
                    }

                    // Entity-владелец инвентаря
                    match memory::read_ptr(inv + fields::inventory::OWNER_ENTITY_REF) {
                        Some(pr) => {
                            let ptype = memory::read_value::<u8>(pr + 0x24).unwrap_or(0);
                            logger::debug(&format!("  Владелец: 0x{pr:X} (type={ptype})"));
                        }
                        None => logger::debug("  Владелец: NULL"),
                    }

                    // Деньги
                    match self.resolve_money_address() {
                        Some(addr) => {
                            let cents = memory::read_value::<i64>(addr).unwrap_or(0);
                            logger::debug(&format!(
                                "  Деньги: 0x{addr:X} = {cents} центов ($ {}.{:02})",
                                cents / 100,
                                (cents % 100).abs(),
                            ));
                        }
                        None => logger::debug("  Деньги: кошелёк не инициализирован"),
                    }
                }
            }
            None => logger::debug("  Инвентарь: NULL"),
        }
    }
    /// Вывести информацию об оружии в руках.
    pub fn log_weapon_info(&self) {
        match self.get_weapon_in_hand_id() {
            Some(id) => {
                let ammo = self.get_current_ammo().unwrap_or(-1);
                let is_fire = self.has_fire_weapon_in_hand().unwrap_or(false);
                let unlimited = self.is_unlimited_ammo().unwrap_or(false);
                logger::info(&format!(
                    "[weapon] ID={} | Патроны={} | Огнестрельное={} | Бесконечные={}",
                    id, ammo, is_fire, unlimited
                ));
            }
            None => {
                logger::info("[weapon] Руки пусты");
            }
        }
    }

    // =============================================================================
    //  Player state machine / multiplayer-relevant fields
    // =============================================================================

    /// Главный state code игрока (`player + 0x430`).
    ///
    /// Это основная state machine, которая крутится в `M2DE_CHuman_Update`.
    pub fn get_state_code_430(&self) -> Option<u32> {
        unsafe { memory::read_value(self.ptr + fields::player::STATE_CODE_430) }
    }

    /// Player state/flags dword (`player + 0x3D8`).
    ///
    /// ВАЖНО:
    /// это не просто byte-mode.
    pub fn get_state_flags_3d8(&self) -> Option<u32> {
        unsafe { memory::read_value(self.ptr + fields::player::STATE_FLAGS_3D8) }
    }

    /// State mask / profile (`player + 0x438`).
    ///
    /// Загружается через `M2DE_CPlayer_LoadStateMask438_ByName`.
    pub fn get_state_mask_438(&self) -> Option<u32> {
        unsafe { memory::read_value(self.ptr + fields::player::STATE_MASK_438) }
    }

    /// Player bitfield (`player + 0x490`).
    pub fn get_state_flags_490(&self) -> Option<u32> {
        unsafe { memory::read_value(self.ptr + fields::player::STATE_FLAGS_490) }
    }

    /// `player.sub45c.state` (`player + 0x464`).
    pub fn get_sub45c_state(&self) -> Option<u32> {
        unsafe { memory::read_value(self.ptr + fields::player::SUBOBJECT_45C_STATE) }
    }

    /// Дополнительный player state/flags dword (`player + 0x510`).
    pub fn get_state_flags_510(&self) -> Option<u32> {
        unsafe { memory::read_value(self.ptr + fields::player::STATE_FLAGS_510) }
    }

    /// Удобный alias: forward vector игрока.
    pub fn get_forward_vector(&self) -> Option<Vec3> {
        self.get_forward()
    }

    // =============================================================================
    //  Direction / Rotation (из vtable reverse)
    // =============================================================================

    /// Получить direction vector через locomotion controller.
    ///
    /// Более точный путь чем frame matrix — учитывает
    /// анимационное состояние и физику.
    /// Подтверждено: vtable[37], locomotion controller slot[22].
    pub fn get_direction(&self) -> Option<Vec3> {
        unsafe {
            let provider = memory::read_ptr(self.ptr + fields::player::PHYSICS_PROVIDER)?;
            let vtable = memory::read_ptr(provider)?;
            let func_ptr = memory::read_ptr(vtable + fields::locomotion_controller::VFUNC_GET_DIR)?;

            type GetDirFn = unsafe extern "C" fn(usize, *mut Vec3) -> *mut Vec3;
            let func: GetDirFn = std::mem::transmute(func_ptr);

            let mut out = Vec3::default();
            let ret = func(provider, &mut out);
            if ret.is_null() {
                return None;
            }

            if out.x.is_finite() && out.y.is_finite() && out.z.is_finite() {
                Some(out)
            } else {
                None
            }
        }
    }

    /// Получить позицию головы/глаз (GetPos + offset (0, 0, 2.0)).
    ///
    /// Подтверждено vtable[43]: `GetPos() + Vec3(0, 0, 2.0)`.
    /// Полезно для прицеливания, raycast'ов, камеры от первого лица.
    pub fn get_head_position(&self) -> Option<Vec3> {
        let mut pos = self.get_position()?;
        pos.z += 2.0;
        Some(pos)
    }

    /// Получить velocity (скорость) через locomotion controller.
    ///
    /// Подтверждено: vtable[68], locomotion controller slot[25].
    pub fn get_velocity(&self) -> Option<Vec3> {
        unsafe {
            let provider = memory::read_ptr(self.ptr + fields::player::PHYSICS_PROVIDER)?;
            let vtable = memory::read_ptr(provider)?;
            let func_ptr =
                memory::read_ptr(vtable + fields::locomotion_controller::VFUNC_GET_VELOCITY)?;

            type GetVelFn = unsafe extern "C" fn(usize) -> *mut Vec3;
            let func: GetVelFn = std::mem::transmute(func_ptr);

            let ptr = func(provider);
            if ptr.is_null() {
                return None;
            }

            let v = std::ptr::read(ptr);
            if v.x.is_finite() && v.y.is_finite() && v.z.is_finite() {
                Some(v)
            } else {
                None
            }
        }
    }

    // =============================================================================
    //  Movement Speed (из vtable[75/76/77])
    // =============================================================================

    /// Получить текущую скорость передвижения (интерполированную).
    ///
    /// Подтверждено vtable[76]: `return *(float*)(this + 0x298)`.
    pub fn get_movement_speed(&self) -> Option<f32> {
        unsafe {
            memory::read_value::<f32>(self.ptr + fields::human::MOVEMENT_SPEED_CURRENT)
        }
    }

    /// Получить целевую скорость передвижения.
    ///
    /// Подтверждено vtable[77]: пишет только +0x294 (target).
    /// Текущая скорость (+0x298) плавно интерполирует к target.
    pub fn get_movement_speed_target(&self) -> Option<f32> {
        unsafe {
            memory::read_value::<f32>(self.ptr + fields::human::MOVEMENT_SPEED_TARGET)
        }
    }

    /// Установить скорость передвижения мгновенно (target + current).
    ///
    /// Подтверждено vtable[75]: пишет в оба поля (+0x294 и +0x298).
    pub fn set_movement_speed(&self, speed: f32) -> bool {
        unsafe {
            let ok1 = memory::write_value(self.ptr + fields::human::MOVEMENT_SPEED_TARGET, speed);
            let ok2 = memory::write_value(self.ptr + fields::human::MOVEMENT_SPEED_CURRENT, speed);
            ok1 && ok2
        }
    }

    /// Установить целевую скорость (плавный переход).
    ///
    /// Подтверждено vtable[77]: пишет только +0x294.
    /// Текущая скорость будет плавно интерполировать к этому значению.
    pub fn set_movement_speed_target(&self, speed: f32) -> bool {
        unsafe {
            memory::write_value(self.ptr + fields::human::MOVEMENT_SPEED_TARGET, speed)
        }
    }

    // =============================================================================
    //  Model / Appearance (из vtable[80/81])
    // =============================================================================

    /// Получить текущий ID внешнего вида (одежда/модель).
    ///
    /// Подтверждено vtable[81]: `return *(u32*)(*(this+0xB8))` if active, else -1.
    /// Возвращает None если компонент не активен.
    pub fn get_appearance_id(&self) -> Option<u32> {
        unsafe {
            // component_b8 -> ptr_data -> appearance_id
            let comp = memory::read_ptr(self.ptr + fields::player::COMPONENT_B8)?;
            let ptr_data = memory::read_ptr(comp)?;
            if ptr_data == 0 {
                return None;
            }
            let id = memory::read_value::<u32>(ptr_data)?;
            if id == 0xFFFFFFFF { None } else { Some(id) }
        }
    }

    /// Получить имя текущей модели (из model descriptor +0xA8).
    ///
    /// Подтверждено vtable[61]: `strcmp(table_entry, descriptor+168)`.
    pub fn get_model_name(&self) -> Option<String> {
        unsafe {
            let desc = memory::read_ptr(self.ptr + fields::human::MODEL_DESCRIPTOR)?;
            let name_addr = desc + fields::model_descriptor::MODEL_NAME;
            if !memory::is_valid_ptr(name_addr) {
                return None;
            }
            let cstr = std::ffi::CStr::from_ptr(name_addr as *const i8);
            let s = cstr.to_string_lossy().into_owned();
            if s.is_empty() { None } else { Some(s) }
        }
    }

    // =============================================================================
    //  Collision Body (из vtable[85])
    // =============================================================================

    /// Проверить, есть ли у персонажа активное collision body.
    ///
    /// Подтверждено vtable[85]: `return *(qword*)(this+0x310) != 0`.
    pub fn has_collision_body(&self) -> Option<bool> {
        unsafe {
            let ptr = memory::read_ptr_raw(self.ptr + fields::human::COLLISION_BODY)?;
            Some(ptr != 0)
        }
    }

    // =============================================================================
    //  Locomotion Controller
    // =============================================================================

    /// Указатель на locomotion controller (он же physics_provider).
    ///
    /// Это C_HumanLocomotionController — управляет анимациями,
    /// передвижением, укрытиями, боем, рэгдоллом.
    /// Vtable: 0x141993998 (68 слотов).
    pub fn locomotion_controller_ptr(&self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + fields::player::PHYSICS_PROVIDER) }
    }

    /// Получить locomotion state через controller.
    ///
    /// Подтверждено: locomotion controller slot[36] = GetState.
    /// Вызывается из CHuman vtable[59].
    pub fn get_locomotion_state(&self) -> Option<u32> {
        unsafe {
            let provider = memory::read_ptr(self.ptr + fields::player::PHYSICS_PROVIDER)?;
            let vtable = memory::read_ptr(provider)?;
            let func_ptr =
                memory::read_ptr(vtable + fields::locomotion_controller::VFUNC_GET_STATE)?;

            type GetStateFn = unsafe extern "C" fn(usize) -> u32;
            let func: GetStateFn = std::mem::transmute(func_ptr);
            Some(func(provider))
        }
    }

    // =============================================================================
    //  Damage parameters (из vtable[74/78])
    // =============================================================================

    /// Получить damage scale factor.
    ///
    /// Подтверждено vtable[78]: пишет float в +0x15C.
    pub fn get_damage_scale(&self) -> Option<f32> {
        unsafe { memory::read_value::<f32>(self.ptr + fields::human::DAMAGE_SCALE_FACTOR) }
    }

    /// Установить damage scale factor.
    ///
    /// Подтверждено vtable[78] SetDamageScaleFactor.
    /// Множитель входящего урона (1.0 = нормальный).
    pub fn set_damage_scale(&self, scale: f32) -> bool {
        unsafe { memory::write_value(self.ptr + fields::human::DAMAGE_SCALE_FACTOR, scale) }
    }
}
