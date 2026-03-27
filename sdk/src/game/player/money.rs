//! Деньги — чтение, запись, игровые функции, HUD popup.

use crate::addresses::{self, constants, fields};
use crate::memory;
use common::logger;

use super::{Player, base};

// =============================================================================
//  Чтение
// =============================================================================

impl Player {
    /// Баланс в центах. $600.00 = 60000.
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

    /// Кошелёк инициализирован и готов к операциям.
    pub fn is_wallet_ready(&self) -> bool {
        self.resolve_money_address().is_some()
    }
}

// =============================================================================
//  Запись (прямая)
// =============================================================================

impl Player {
    /// Установить баланс напрямую. Без HUD, без статистики.
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

    /// Прибавить/вычесть центы.
    pub fn add_money(&self, delta_cents: i64) -> Option<i64> {
        let current = self.get_money_cents()?;
        let new_amount = current + delta_cents;
        self.set_money(new_amount).then_some(new_amount)
    }

    /// Прибавить/вычесть доллары.
    pub fn add_money_dollars(&self, dollars: i32) -> Option<i64> {
        self.add_money(dollars as i64 * 100)
    }
}

// =============================================================================
//  Через игровые функции
// =============================================================================

impl Player {
    /// Добавить деньги через движок (тихо, без HUD).
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

    /// Добавить деньги + показать HUD уведомление.
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

    /// Показать HUD popup о деньгах (±$X.XX). Не добавляет реально.
    fn show_money_notification(&self, cents: i64) {
        unsafe {
            let hud_mgr = match memory::read_ptr(base() + addresses::globals::HUD_MANAGER) {
                Some(p) => p,
                None => return,
            };
            let money_display = match memory::read_ptr(hud_mgr + fields::hud_manager::MONEY_DISPLAY)
            {
                Some(p) => p,
                None => return,
            };

            // Сбросить таймер анимации
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
}

// =============================================================================
//  Внутренний pointer-chain до ячейки денег
// =============================================================================

impl Player {
    /// Полная цепочка указателей до ячейки с деньгами.
    ///
    /// ```text
    /// CHuman.inventory -> slots_start -> slot[5] ->
    /// -> vec_begin -> money_item -> wallet.inner ->
    /// -> money_container -> value (i64 центы)
    /// ```
    ///
    /// Любой указатель может быть NULL на ранних стадиях.
    /// Здесь `memory::read_ptr` оправдан — промежуточные структуры
    /// не полностью типизированы.
    fn resolve_money_address(&self) -> Option<usize> {
        unsafe {
            // Первый шаг — через структуру
            let inv = self.human()?.inventory as usize;
            if !memory::is_valid_ptr(inv) {
                return None;
            }

            // Дальше — pointer chain (нет полных repr(C) структур)
            let slots_base = memory::read_ptr(inv + fields::inventory::SLOTS_START)?;
            let slot5 = memory::read_ptr(slots_base + constants::slots::MONEY * 8)?;
            let vec_begin = memory::read_ptr_raw(slot5 + fields::slot::VEC_BEGIN)?;
            let vec_end = memory::read_ptr_raw(slot5 + fields::slot::VEC_END)?;

            if vec_begin == 0 || vec_end <= vec_begin || !memory::is_valid_ptr(vec_begin) {
                return None;
            }

            let money_item = memory::read_ptr(vec_begin)?;
            let inner = memory::read_ptr(money_item + fields::wallet::INNER_STRUCT)?;
            let container = memory::read_ptr(inner + fields::wallet_inner::MONEY_CONTAINER_PTR)?;
            let addr = container + fields::money_container::VALUE;

            memory::is_valid_ptr(addr).then_some(addr)
        }
    }
}
