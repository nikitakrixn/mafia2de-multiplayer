use std::time::{Duration, Instant};

use common::logger;
use crate::addresses;
use crate::addresses::constants;
use crate::addresses::fields;
use crate::memory;
use super::base;

#[derive(Debug, Clone, Copy)]
pub struct Player {
    ptr: usize, // C_Human*
}

unsafe impl Send for Player {}

impl Player {
    // ═══════════════════════════════════════════════════════════════════
    //  Конструкторы
    // ═══════════════════════════════════════════════════════════════════

    pub fn get_active() -> Option<Self> {
        unsafe {
            let mgr = memory::read_ptr(base() + addresses::globals::GAME_MANAGER)?;
            let player = memory::read_ptr(mgr + fields::game_manager::ACTIVE_PLAYER)?;
            Some(Self { ptr: player })
        }
    }

    pub fn wait_until_ready(timeout_secs: u64) -> Option<Self> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let mut reported = false;

        loop {
            if let Some(player) = Self::get_active() {
                if !reported {
                    logger::info(&format!(
                        "Player pointer at 0x{:X}, waiting for inventory...",
                        player.ptr,
                    ));
                    reported = true;
                }
                if player.is_ready() {
                    logger::info("Player fully initialized");
                    return Some(player);
                }
            }
            if Instant::now() > deadline {
                logger::error(if reported {
                    "Timeout: inventory never initialized"
                } else {
                    "Timeout: player pointer never appeared"
                });
                return None;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Базовые аксессоры
    // ═══════════════════════════════════════════════════════════════════

    pub fn as_ptr(&self) -> usize { self.ptr }

    pub fn is_ready(&self) -> bool { self.inventory_ptr().is_some() }

    pub fn inventory_ptr(&self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + fields::player::INVENTORY) }
    }

    /// Проверяет что wallet полностью инициализирован
    /// (slot[5] != null И внутренний вектор не пуст).
    pub fn is_wallet_ready(&self) -> bool {
        self.resolve_money_address().is_some()
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Деньги — ЧТЕНИЕ (прямое чтение памяти)
    // ═══════════════════════════════════════════════════════════════════

    pub fn get_money_cents(&self) -> Option<i64> {
        unsafe {
            let addr = self.resolve_money_address()?;
            memory::read_value::<i64>(addr)
        }
    }

    /// Возвращает 0 если wallet не инициализирован.
    pub fn get_money_cents_or_zero(&self) -> i64 {
        self.get_money_cents().unwrap_or(0)
    }

    pub fn get_money_dollars(&self) -> Option<i32> {
        self.get_money_cents().map(|c| (c / 100) as i32)
    }

    pub fn get_money_formatted(&self) -> Option<String> {
        self.get_money_cents().map(|c| {
            let d = c / 100;
            let r = (c % 100).abs();
            format!("$ {d}.{r:02}")
        })
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Деньги — ЗАПИСЬ (прямая запись в память)
    // ═══════════════════════════════════════════════════════════════════

    pub fn set_money(&self, cents: i64) -> bool {
        unsafe {
            match self.resolve_money_address() {
                Some(addr) => {
                    std::ptr::write(addr as *mut i64, cents);
                    true
                }
                None => {
                    logger::error("set_money: wallet not allocated");
                    false
                }
            }
        }
    }

    pub fn set_money_dollars(&self, dollars: i32) -> bool {
        self.set_money(dollars as i64 * 100)
    }

    pub fn add_money(&self, delta_cents: i64) -> Option<i64> {
        let current = self.get_money_cents()?;
        let new_amount = current + delta_cents;
        if self.set_money(new_amount) { Some(new_amount) } else { None }
    }

    pub fn add_money_dollars(&self, dollars: i32) -> Option<i64> {
        self.add_money(dollars as i64 * 100)
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Деньги — через игровые функции (с HUD уведомлением)
    // ═══════════════════════════════════════════════════════════════════

    /// Добавить деньги с HUD уведомлением (зелёный/красный popup).
    ///
    /// Вызывает `M2DE_Inventory_AddMoneyNotify` напрямую.
    /// Перед вызовом проверяет что inventory и parent_ref валидны.
    ///
    /// Возвращает `false` если структуры не инициализированы.
    pub fn add_money_with_hud(&self, cents: i64) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_money_with_hud: inventory NULL");
            return false;
        };

        // Проверяем parent_ref — функция читает [inv+0x170]
        // и проверяет byte [parent+0x24] == 16
        unsafe {
            let parent = match memory::read_ptr(inv + fields::inventory::PARENT_REF) {
                Some(p) => p,
                None => {
                    logger::warn("add_money_with_hud: parent_ref NULL, fallback to direct write");
                    return self.add_money(cents).is_some();
                }
            };

            // Проверяем тип (должен быть 16 для player inventory)
            let inv_type = memory::read_value::<u8>(parent + 0x24).unwrap_or(0);
            if inv_type != 16 {
                logger::warn(&format!(
                    "add_money_with_hud: inv_type={} (need 16), fallback to direct write",
                    inv_type
                ));
                return self.add_money(cents).is_some();
            }

            // Всё валидно — вызываем игровую функцию
            type AddMoneyNotifyFn = unsafe extern "C" fn(usize, i64) -> u8;
            let func: AddMoneyNotifyFn = std::mem::transmute(
                base() + addresses::functions::player::INVENTORY_ADD_MONEY_NOTIFY
            );

            logger::debug(&format!(
                "Calling M2DE_Inventory_AddMoneyNotify(0x{:X}, {})",
                inv, cents
            ));

            func(inv, cents);
            true
        }
    }

    /// Добавить деньги с HUD (в долларах).
    pub fn add_money_dollars_with_hud(&self, dollars: i32) -> bool {
        self.add_money_with_hud(dollars as i64 * 100)
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Внутренние хелперы
    // ═══════════════════════════════════════════════════════════════════

    /// Полная цепочка указателей до ячейки с деньгами.
    /// Возвращает None если любой указатель в цепочке невалиден.
    fn resolve_money_address(&self) -> Option<usize> {
        unsafe {
            let inv = memory::read_ptr(self.ptr + fields::player::INVENTORY)?;
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
            if memory::is_valid_ptr(addr) { Some(addr) } else { None }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Диагностика
    // ═══════════════════════════════════════════════════════════════════

    pub fn log_debug_info(&self) {
        logger::debug(&format!("Player ptr: 0x{:X} (C_Human*)", self.ptr));

        match self.inventory_ptr() {
            Some(inv) => {
                logger::debug(&format!("  Inventory: 0x{inv:X}"));
                unsafe {
                    if let Some(t) = memory::read_value::<u8>(inv + fields::inventory::TYPE) {
                        logger::debug(&format!("  Inv type: {t}"));
                    }
                    if let Some(start) = memory::read_ptr(inv + fields::inventory::SLOTS_START) {
                        let end = memory::read_ptr(inv + fields::inventory::SLOTS_END).unwrap_or(0);
                        let count = if end > start { (end - start) / 8 } else { 0 };
                        logger::debug(&format!("  Slots: {count}"));
                    }
                    // Parent ref (нужен для HUD)
                    match memory::read_ptr(inv + fields::inventory::PARENT_REF) {
                        Some(pr) => {
                            let ptype = memory::read_value::<u8>(pr + 0x24).unwrap_or(0);
                            logger::debug(&format!("  Parent ref: 0x{pr:X} (type={ptype})"));
                        }
                        None => logger::debug("  Parent ref: NULL"),
                    }
                    // Деньги
                    match self.resolve_money_address() {
                        Some(addr) => {
                            let cents = memory::read_value::<i64>(addr).unwrap_or(0);
                            logger::debug(&format!(
                                "  Money addr: 0x{addr:X} = {cents} cents ($ {}.{:02})",
                                cents / 100, (cents % 100).abs()
                            ));
                        }
                        None => logger::debug("  Money: wallet not yet allocated"),
                    }
                }
            }
            None => logger::debug("  Inventory: NULL"),
        }
    }
}