use crate::game::entity;
use crate::game::player::Vec3;
use crate::addresses::fields;
use crate::addresses::constants::entity_types;
use crate::memory;
use common::logger;

pub struct Npc {
    native: usize,
    name: String,
}

impl Npc {
    pub fn find(name: &str) -> Option<Self> {
        let native = entity::find_native_entity(name)?;

        // Проверить что это human (type 0x0E или 0x10)
        let entity_type = unsafe {
            memory::read_value::<u8>(native + fields::player::ENTITY_TYPE)?
        };
        if entity_type != entity_types::HUMAN_NPC
            && entity_type != entity_types::HUMAN_PLAYER
        {
            logger::warn(&format!(
                "[npc] '{}' is not a human (type=0x{:02X})", name, entity_type
            ));
            return None;
        }

        Some(Self { native, name: name.to_string() })
    }

    pub fn ptr(&self) -> usize { self.native }

    /// Жив ли NPC.
    pub fn is_alive(&self) -> bool {
        unsafe {
            memory::read_value::<u8>(self.native + fields::player::IS_DEAD)
                .map(|v| v == 0)
                .unwrap_or(false)
        }
    }

    /// Текущее здоровье.
    pub fn health(&self) -> f32 {
        unsafe {
            memory::read_value::<f32>(self.native + fields::player::CURRENT_HEALTH)
                .unwrap_or(0.0)
        }
    }

    /// Установить здоровье.
    pub fn set_health(&self, hp: f32) {
        unsafe { memory::write_value(self.native + fields::player::CURRENT_HEALTH, hp); }
    }

    /// Сделать неуязвимым.
    pub fn set_invulnerable(&self, enabled: bool) {
        unsafe { memory::write_value(self.native + fields::player::INVULNERABILITY, enabled as u8); }
    }

    /// Установить полубога.
    pub fn set_demigod(&self, enabled: bool) {
        unsafe { memory::write_value(self.native + fields::player::DEMIGOD, enabled as u8); }
    }

    /// Получить позицию.
    pub fn get_position(&self) -> Option<Vec3> {
        unsafe {
            let mut out = Vec3::default();
            type GetPosFn = unsafe extern "C" fn(usize, *mut Vec3) -> *mut Vec3;
            let func: GetPosFn = memory::fn_at(
                crate::game::base() + crate::addresses::functions::entity::GET_POS,
            );
            let ret = func(self.native, &mut out);
            if ret.is_null() { None } else { Some(out) }
        }
    }

    /// Телепортировать NPC.
    pub fn set_position(&self, pos: &Vec3) {
        unsafe {
            type SetPosFn = unsafe extern "C" fn(usize, *const Vec3);
            let func: SetPosFn = memory::fn_at(
                crate::game::base() + crate::addresses::functions::entity::SET_POS,
            );
            func(self.native, pos);
        }
    }

    /// Вывести инфо в лог.
    pub fn log_info(&self) {
        let pos = self.get_position().unwrap_or_default();
        logger::info(&format!(
            "[npc] {} | ptr=0x{:X} | HP={:.0} | alive={} | pos={}",
            self.name, self.native, self.health(), self.is_alive(), pos,
        ));
    }
}