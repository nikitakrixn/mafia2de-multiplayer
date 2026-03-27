//! Диагностика и логирование.

use crate::addresses::fields;
use crate::memory;
use common::logger;

use super::Player;

impl Player {
    /// Подробная информация об игроке.
    pub fn log_debug_info(&self) {
        logger::debug(&format!("Player ptr: 0x{:X} (C_Player*)", self.as_ptr()));

        match self.inventory_ptr() {
            Some(inv) => {
                logger::debug(&format!("  Инвентарь: 0x{inv:X}"));
                unsafe {
                    if let Some(t) = memory::read_value::<u8>(inv + fields::inventory::TYPE) {
                        logger::debug(&format!("  Тип инвентаря: {t}"));
                    }

                    if let Some(start) = memory::read_ptr(inv + fields::inventory::SLOTS_START) {
                        let end = memory::read_ptr(inv + fields::inventory::SLOTS_END).unwrap_or(0);
                        let count = if end > start { (end - start) / 8 } else { 0 };
                        logger::debug(&format!("  Слотов: {count}"));
                    }

                    match memory::read_ptr(inv + fields::inventory::OWNER_ENTITY_REF) {
                        Some(pr) => {
                            let ptype = memory::read_value::<u8>(pr + 0x24).unwrap_or(0);
                            logger::debug(&format!("  Владелец: 0x{pr:X} (type={ptype})"));
                        }
                        None => logger::debug("  Владелец: NULL"),
                    }
                }

                match self.get_money_formatted() {
                    Some(money) => logger::debug(&format!("  Деньги: {money}")),
                    None => logger::debug("  Деньги: кошелёк не инициализирован"),
                }
            }
            None => logger::debug("  Инвентарь: NULL"),
        }
    }

    /// Информация об оружии в руках.
    pub fn log_weapon_info(&self) {
        match self.get_weapon_in_hand_id() {
            Some(id) => {
                let ammo = self.get_current_ammo().unwrap_or(-1);
                let is_fire = self.has_fire_weapon_in_hand().unwrap_or(false);
                let unlimited = self.is_unlimited_ammo().unwrap_or(false);
                logger::info(&format!(
                    "[weapon] ID={id} | Патроны={ammo} | Огнестрельное={is_fire} | Бесконечные={unlimited}"
                ));
            }
            None => {
                logger::info("[weapon] Руки пусты");
            }
        }
    }
}
