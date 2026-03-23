use crate::addresses::functions;
use crate::game::{
    entity,
    entity_ref::EntityRef,
    entity_types::{EntityType, FactoryType},
    player::Vec3,
};
use crate::memory;
use common::logger;

/// Высокоуровневая обёртка над humanoid entity по имени.
///
/// Подходит для Joe / Henry / других NPC, доступных через FindByName.
pub struct Npc {
    entity: EntityRef,
    name: String,
}

impl Npc {
    /// Найти NPC по имени.
    ///
    /// Пропускает только human/player factory types.
    pub fn find(name: &str) -> Option<Self> {
        let entity = entity::find_entity_ref_by_name(name)?;

        let Some(ft) = entity.factory_type() else {
            logger::warn(&format!("[npc] '{}' has unknown factory type", name));
            return None;
        };

        if !matches!(ft, FactoryType::HumanNpc | FactoryType::Player) {
            logger::warn(&format!(
                "[npc] '{}' is not humanoid (factory={})",
                name,
                ft.display_name()
            ));
            return None;
        }

        Some(Self {
            entity,
            name: name.to_string(),
        })
    }

    /// Сырой native pointer.
    pub fn ptr(&self) -> usize {
        self.entity.ptr()
    }

    /// Typed low-level entity reference.
    pub fn entity(&self) -> EntityRef {
        self.entity
    }

    /// FactoryType enum.
    pub fn factory_type(&self) -> Option<FactoryType> {
        self.entity.factory_type()
    }

    /// Lua-facing entity type.
    pub fn entity_type(&self) -> Option<EntityType> {
        self.entity.lua_entity_type()
    }

    /// Жив ли NPC.
    pub fn is_alive(&self) -> bool {
        self.entity.is_alive().unwrap_or(false)
    }

    /// Текущее здоровье.
    pub fn health(&self) -> f32 {
        self.entity.health().unwrap_or(0.0)
    }

    /// Установить здоровье.
    pub fn set_health(&self, hp: f32) {
        unsafe {
            memory::write_value(
                self.entity.ptr() + crate::addresses::fields::player::CURRENT_HEALTH,
                hp,
            );
        }
    }

    /// Сделать неуязвимым.
    pub fn set_invulnerable(&self, enabled: bool) {
        unsafe {
            memory::write_value(
                self.entity.ptr() + crate::addresses::fields::player::INVULNERABILITY,
                enabled as u8,
            );
        }
    }

    /// Установить полубога.
    pub fn set_demigod(&self, enabled: bool) {
        unsafe {
            memory::write_value(
                self.entity.ptr() + crate::addresses::fields::player::DEMIGOD,
                enabled as u8,
            );
        }
    }

    /// Получить позицию.
    pub fn get_position(&self) -> Option<Vec3> {
        self.entity.position()
    }

    /// Телепортировать NPC через native SetPos.
    pub fn set_position(&self, pos: &Vec3) {
        unsafe {
            type SetPosFn = unsafe extern "C" fn(usize, *const Vec3);
            let func: SetPosFn = memory::fn_at(crate::game::base() + functions::entity::SET_POS);
            func(self.entity.ptr(), pos);
        }
    }

    /// Установить forward direction NPC через frame node.
    ///
    /// ВАЖНО:
    /// это low-level запись в basis-векторы frame node.
    /// Для ground humanoid proxy этого достаточно.
    pub fn set_forward(&self, dir: &Vec3) -> bool {
        let Some(frame) = (unsafe {
            memory::read_ptr(self.entity.ptr() + crate::addresses::fields::entity::FRAME_NODE)
        }) else {
            return false;
        };

        // Нормализация — защита от мусора/нулевого вектора.
        let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
        if !len.is_finite() || len < 0.0001 {
            return false;
        }

        let fx = dir.x / len;
        let fy = dir.y / len;
        let fz = dir.z / len;

        // Для humanoid нам обычно достаточно right = perpendicular in XY plane.
        let rx = fy;
        let ry = -fx;
        let rz = 0.0f32;

        unsafe {
            // Forward (Col1)
            memory::write_value(frame + crate::addresses::fields::entity_frame::FORWARD_X, fx);
            memory::write_value(frame + crate::addresses::fields::entity_frame::FORWARD_Y, fy);
            memory::write_value(frame + crate::addresses::fields::entity_frame::FORWARD_Z, fz);

            // Right (Col0)
            memory::write_value(frame + crate::addresses::fields::entity_frame::RIGHT_X, rx);
            memory::write_value(frame + crate::addresses::fields::entity_frame::RIGHT_Y, ry);
            memory::write_value(frame + crate::addresses::fields::entity_frame::RIGHT_Z, rz);

            // Up (Col2) — оставляем world-up.
            memory::write_value(frame + crate::addresses::fields::entity_frame::UP_X, 0.0f32);
            memory::write_value(frame + crate::addresses::fields::entity_frame::UP_Y, 0.0f32);
            memory::write_value(frame + crate::addresses::fields::entity_frame::UP_Z, 1.0f32);
        }

        true
    }

    /// Вывести инфо в лог.
    pub fn log_info(&self) {
        let pos = self.get_position().unwrap_or_default();
        let ft = self
            .factory_type()
            .map(|t| t.display_name())
            .unwrap_or("UNKNOWN");

        logger::info(&format!(
            "[npc] {} | ptr=0x{:X} | type={} | HP={:.0} | alive={} | pos={}",
            self.name,
            self.entity.ptr(),
            ft,
            self.health(),
            self.is_alive(),
            pos,
        ));
    }
}
