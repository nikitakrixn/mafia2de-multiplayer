//! Высокоуровневый API игрока.
//!
//! Player — обёртка над C_Human*, скрывающая цепочки указателей
//! и вызовы функций движка. Через неё клиент работает с позицией,
//! деньгами, оружием и управлением.
//!
//! ВАЖНО: Player содержит сырой указатель на объект движка.
//! Движок не потокобезопасен — использовать Player можно только
//! в том потоке, где он был получен (обычно главный поток игры).

use std::ffi::{CStr, CString};
use std::time::{Duration, Instant};

use common::logger;
use crate::addresses;
use crate::addresses::constants;
use crate::addresses::fields;
use crate::memory;
use super::base;

/// Обёртка над указателем на C_Human в памяти игры.
///
/// Не реализует Send — указатель валиден только
/// в потоке, где был получен. Для передачи между потоками
/// используй `as_ptr()` и пересоздавай Player на месте.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    /// Указатель на C_Human в памяти игры.
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
    // ═══════════════════════════════════════════════════════════════
    //  Получение указателя на игрока
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    //  Базовые аксессоры
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    //  Деньги — чтение (прямое чтение памяти)
    // ═══════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    //  Деньги — запись (прямая запись в память)
    // ═══════════════════════════════════════════════════════════════

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
        if self.set_money(new_amount) { Some(new_amount) } else { None }
    }

    /// Прибавить/вычесть доллары напрямую в памяти.
    pub fn add_money_dollars(&self, dollars: i32) -> Option<i64> {
        self.add_money(dollars as i64 * 100)
    }

    // ═══════════════════════════════════════════════════════════════
    //  Деньги — через игровые функции
    // ═══════════════════════════════════════════════════════════════

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
        let func: ModifyMoneyFn = unsafe {
            memory::fn_at(base() + addresses::functions::player::INVENTORY_MODIFY_MONEY)
        };

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
            // g_HUDManager → +0x98 → money display component
            let hud_mgr = match memory::read_ptr(
                base() + addresses::globals::HUD_MANAGER,
            ) {
                Some(p) => p,
                None => {
                    logger::debug("show_money_notification: HUD-менеджер не готов");
                    return;
                }
            };

            let money_display = match memory::read_ptr(
                hud_mgr + fields::hud_manager::MONEY_DISPLAY,
            ) {
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
            let update: UpdateFn = memory::fn_at(
                base() + addresses::functions::hud::UPDATE_MONEY_COUNTER,
            );

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

    // ═══════════════════════════════════════════════════════════════
    //  Позиция
    // ═══════════════════════════════════════════════════════════════

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
            let func: GetPosFn = memory::fn_at(
                base() + addresses::functions::entity::GET_POS,
            );

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
            let func: SetPosFn = memory::fn_at(
                base() + addresses::functions::entity::SET_POS,
            );

            func(self.ptr, pos as *const Vec3);
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════
    //  Оружие
    // ═══════════════════════════════════════════════════════════════

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
            logger::debug("  → оружие добавлено / патроны обновлены");
            true
        } else {
            logger::warn("  → add_weapon вернул 0 (слоты заняты или невалидный ID)");
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
        let func: AddAmmoFn = unsafe {
            memory::fn_at(base() + addresses::functions::player::INVENTORY_ADD_AMMO)
        };

        unsafe { func(inv, weapon_id, ammo) };
        true
    }

    // ═══════════════════════════════════════════════════════════════
    //  Управление (блокировка, стиль)
    // ═══════════════════════════════════════════════════════════════

    /// Заблокировано ли управление игроком.
    pub fn are_controls_locked(&self) -> Option<bool> {
        let control = self.control_component_ptr()?;

        type Fn = unsafe extern "C" fn(usize) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::IS_LOCKED)
        };

        Some(unsafe { func(control) != 0 })
    }

    /// Заблокировать / разблокировать управление.
    pub fn lock_controls(&self, locked: bool) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls: компонент управления NULL");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED)
        };

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
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_LOCKED)
        };

        unsafe { func(control, 1, 1) };
        true
    }

    /// Получить текущий стиль управления как строку.
    ///
    /// Примеры: "Normal", "DoNothing", "Intoxicated" и т.д.
    pub fn get_control_style_str(&self) -> Option<String> {
        let control = self.control_component_ptr()?;

        type Fn = unsafe extern "C" fn(usize) -> *const i8;
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::GET_STYLE_STR)
        };

        let ptr = unsafe { func(control) };
        if ptr.is_null() {
            return None;
        }

        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
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
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::player_control::SET_STYLE_STR)
        };

        unsafe { func(control, c_style.as_ptr()) != 0 }
    }

    // ═══════════════════════════════════════════════════════════════
    //  Внутренние хелперы
    // ═══════════════════════════════════════════════════════════════

    /// Полная цепочка указателей до ячейки с деньгами.
    ///
    /// Путь:
    /// C_Human → Inventory → slots_start → slot[5] →
    /// → vec_begin → money_item → wallet.inner →
    /// → money_container → value (i64 центы)
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
            if memory::is_valid_ptr(addr) { Some(addr) } else { None }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    //  Диагностика
    // ═══════════════════════════════════════════════════════════════

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
}