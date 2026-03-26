//! High-level обёртка над humanoid entity по имени.
//!
//! Подходит для Joe / Henry / других NPC, доступных через FindByName.
//!
//! Для подтверждённых полей `CHuman` используем `repr(C)` структуру:
//! - health
//! - invulnerability
//! - demigod
//! - owner / in-vehicle
//!
//! Для системных действий движка (например SetPos) используем
//! native функции по RVA через `memory::fn_at()`.

use crate::game::{
    entity,
    entity_ref::EntityRef,
    entity_types::{EntityType, FactoryType},
};
use crate::structures::CHuman;
use crate::types::Vec3;
use common::logger;
use crate::memory::Ptr;

/// Высокоуровневая обёртка над humanoid entity по имени.
#[derive(Debug)]
pub struct Npc {
    entity: EntityRef,
    ptr: Ptr<CHuman>,
    name: String,
}

impl Npc {
    /// Найти NPC по имени.
    ///
    /// Пропускает только humanoid factory types:
    /// - `HumanNpc`
    /// - `Player`
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
            ptr: Ptr::<CHuman>::new(entity.ptr()),
            entity,
            name: name.to_string(),
        })
    }

    /// Сырой native pointer.
    pub fn ptr(&self) -> usize {
        self.ptr.addr()
    }

    /// Typed low-level entity reference.
    pub fn entity(&self) -> EntityRef {
        self.entity
    }

    /// Typed reference на `CHuman`.
    ///
    /// # Safety
    ///
    /// Указатель должен указывать на живой humanoid object.
    unsafe fn human(&self) -> Option<&CHuman> {
        unsafe { self.ptr.as_ref() }
    }

    /// Typed mutable reference на `CHuman`.
    ///
    /// # Safety
    ///
    /// Использовать только из game thread.
    unsafe fn human_mut(&self) -> Option<&mut CHuman> {
        unsafe { self.ptr.as_mut() }
    }

    pub fn factory_type(&self) -> Option<FactoryType> {
        self.entity.factory_type()
    }

    pub fn entity_type(&self) -> Option<EntityType> {
        self.entity.lua_entity_type()
    }

    /// Жив ли NPC.
    pub fn is_alive(&self) -> bool {
        unsafe { self.human().map(|h| h.is_alive()).unwrap_or(false) }
    }

    /// Текущее здоровье.
    pub fn health(&self) -> f32 {
        unsafe { self.human().map(|h| h.health()).unwrap_or(0.0) }
    }

    /// Установить здоровье напрямую в `CHuman.current_health`.
    pub fn set_health(&self, hp: f32) {
        if let Some(h) = unsafe { self.human_mut() } {
            h.current_health = hp;
        }
    }

    /// Сделать неуязвимым.
    pub fn set_invulnerable(&self, enabled: bool) {
        if let Some(h) = unsafe { self.human_mut() } {
            h.invulnerability = enabled as u8;
        }
    }

    /// Установить полубога.
    pub fn set_demigod(&self, enabled: bool) {
        if let Some(h) = unsafe { self.human_mut() } {
            h.demigod = enabled as u8;
        }
    }

    /// Получить позицию.
    pub fn get_position(&self) -> Option<Vec3> {
        self.entity.position()
    }

    /// Телепортировать NPC через native SetPos.
    ///
    /// Не пишем напрямую в frame node — игра обновляет не только transform,
    /// но и внутренние кеши/dirty flags.
     pub fn set_position(&self, pos: &Vec3) {
        unsafe {
            let entity_ptr = self.entity.ptr();
            type SetPosFn = unsafe extern "C" fn(usize, *const Vec3);
            let func: SetPosFn =
                crate::memory::fn_at(crate::game::base() + crate::addresses::functions::entity::SET_POS);
            func(entity_ptr, pos);
        }
    }

    /// Выставить basis-векторы frame node вручную.
    ///
    /// Это низкоуровневый helper для debug/экспериментов.
    /// Для обычной телепортации/поворота предпочтительнее engine-функции.
    pub fn set_forward(&self, dir: &Vec3) -> bool {
        let frame = unsafe {
            match self.human() {
                Some(h) => h.actor.frame_node as usize,
                None => return false,
            }
        };

        if !crate::memory::is_valid_ptr(frame) {
            return false;
        }

        let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
        if !len.is_finite() || len < 0.0001 {
            return false;
        }

        let fx = dir.x / len;
        let fy = dir.y / len;
        let fz = dir.z / len;

        let rx = fy;
        let ry = -fx;
        let rz = 0.0f32;

        unsafe {
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::FORWARD_X,
                fx,
            );
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::FORWARD_Y,
                fy,
            );
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::FORWARD_Z,
                fz,
            );

            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::RIGHT_X,
                rx,
            );
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::RIGHT_Y,
                ry,
            );
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::RIGHT_Z,
                rz,
            );

            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::UP_X,
                0.0f32,
            );
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::UP_Y,
                0.0f32,
            );
            crate::memory::write(
                frame + crate::addresses::fields::entity_frame::UP_Z,
                1.0f32,
            );
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