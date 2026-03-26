//! Оружие, патроны, оружие в руках.

use crate::addresses::{self, fields};
use crate::memory;
use common::logger;

use super::{Player, base};

// =============================================================================
//  Добавление оружия / патронов
// =============================================================================

impl Player {
    /// Добавить оружие с патронами. Если уже есть — только патроны.
    pub fn add_weapon(&self, weapon_id: u32, ammo: u32) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_weapon: инвентарь NULL");
            return false;
        };

        type AddWeaponFn = unsafe extern "C" fn(usize, u32, i32) -> u8;
        let func: AddWeaponFn = unsafe {
            memory::fn_at(base() + addresses::functions::player::INVENTORY_ADD_WEAPON_CORE)
        };

        let result = unsafe { func(inv, weapon_id, ammo as i32) };
        result != 0
    }

    /// Добавить только патроны (без оружия).
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

    /// Включить/выключить бесконечные патроны.
    pub fn set_unlimited_ammo(&self, enabled: bool) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            return false;
        };
        unsafe { memory::write(inv + fields::inventory::UNLIMITED_AMMO, enabled as u8) }
    }

    /// Включены ли бесконечные патроны.
    pub fn is_unlimited_ammo(&self) -> Option<bool> {
        let inv = self.inventory_ptr()?;
        unsafe { memory::read_value::<u8>(inv + fields::inventory::UNLIMITED_AMMO).map(|v| v != 0) }
    }
}

// =============================================================================
//  Оружие в руках
// =============================================================================

impl Player {
    /// Pointer на weapon_state компонент (`CHuman.weapon_state`).
    fn weapon_state_ptr(&self) -> Option<usize> {
        let ws = unsafe { self.human()?.weapon_state };
        let addr = ws as usize;
        memory::is_valid_ptr(addr).then_some(addr)
    }

    /// Pointer на WeaponData текущего оружия. `None` = руки пусты.
    fn current_weapon_data(&self) -> Option<usize> {
        let ws = self.weapon_state_ptr()?;
        unsafe { memory::read_ptr(ws + fields::weapon_state::CURRENT_WEAPON_DATA) }
    }

    /// Есть ли оружие в руках.
    pub fn has_weapon_in_hand(&self) -> Option<bool> {
        let ws = self.weapon_state_ptr()?;
        let data =
            unsafe { memory::read_ptr_raw(ws + fields::weapon_state::CURRENT_WEAPON_DATA)? };
        Some(data != 0)
    }

    /// ID оружия в руках.
    pub fn get_weapon_in_hand_id(&self) -> Option<u32> {
        let wd = self.current_weapon_data()?;
        unsafe { memory::read_value::<u32>(wd + fields::weapon_data::WEAPON_ID) }
    }

    /// Флаги типа оружия в руках.
    fn weapon_flags(&self) -> Option<u32> {
        let wd = self.current_weapon_data()?;
        unsafe { memory::read_value::<u32>(wd + fields::weapon_data::WEAPON_FLAGS) }
    }

    /// Огнестрельное ли оружие в руках.
    pub fn has_fire_weapon_in_hand(&self) -> Option<bool> {
        self.weapon_flags()
            .map(|f| (f & fields::weapon_type_flags::FIRE_WEAPON) != 0)
    }

    /// Холодное ли оружие (нож, бита).
    pub fn has_cold_weapon_in_hand(&self) -> Option<bool> {
        self.weapon_flags()
            .map(|f| (f & fields::weapon_type_flags::COLD_WEAPON) != 0)
    }

    /// Метательное ли оружие (граната, молотов).
    pub fn has_throwing_weapon_in_hand(&self) -> Option<bool> {
        self.weapon_flags()
            .map(|f| (f & fields::weapon_type_flags::THROWING_WEAPON) != 0)
    }

    /// Руки пусты.
    pub fn has_empty_hands(&self) -> Option<bool> {
        self.has_weapon_in_hand().map(|has| !has)
    }

    /// Текущие патроны в обойме.
    pub fn get_current_ammo(&self) -> Option<i32> {
        let wd = self.current_weapon_data()?;
        unsafe {
            let container =
                memory::read_ptr(wd + fields::weapon_data::AMMO_CONTAINER)?;
            let store =
                memory::read_ptr(container + fields::value_container::STORE_PTR)?;
            memory::read_value::<i32>(store + fields::value_store::VALUE)
        }
    }

    /// Тип оружия в руках как строка.
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
}