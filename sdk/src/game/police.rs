//! High-level доступ к police-script owner ветке.
//!
//! Это reverse-based API поверх уже подтверждённой цепочки:
//! - singleton owner (`M2DE_g_PoliceScriptOwner`)
//! - active child по `owner + 0x10`
//! - child init/destroy path
//! - remove-policeman child call path
//!
//! Полезно для:
//! - runtime диагностики
//! - devtools
//! - быстрой проверки корректности reverse'а
//! - controlled экспериментов над police-script системой

use common::logger;

use crate::game::{base, entity_ref::EntityRef, script_entity::ScriptEntity};
use crate::{addresses, memory};

/// Дружественная обёртка над singleton owner object police-script ветки.
#[derive(Debug, Clone, Copy)]
pub struct PoliceScriptOwner {
    ptr: usize,
}

impl PoliceScriptOwner {
    // =============================================================================
    //  Получение owner object
    // =============================================================================

    /// Получить singleton owner, если он уже существует.
    ///
    /// Ничего не создаёт.
    pub fn get() -> Option<Self> {
        let ptr = unsafe { memory::read_ptr(base() + addresses::globals::POLICE_SCRIPT_OWNER)? };

        Some(Self { ptr })
    }

    /// Получить owner, при необходимости создав его.
    ///
    /// Вызывает lazy path:
    /// `M2DE_PoliceScriptOwner_GetOrCreate`.
    ///
    /// ВАЖНО:
    /// функция самого движка не возвращает pointer owner напрямую —
    /// после вызова нужно заново читать global.
    pub fn get_or_create() -> Option<Self> {
        if let Some(owner) = Self::get() {
            return Some(owner);
        }

        type Fn = unsafe extern "C" fn();
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::police_script_owner::GET_OR_CREATE)
        };

        unsafe { func() };

        Self::get()
    }

    // =============================================================================
    //  Базовые аксессоры
    // =============================================================================

    /// Сырой указатель на owner object.
    pub fn as_ptr(self) -> usize {
        self.ptr
    }

    /// `+0x00` -> root/sentinel pointer.
    pub fn root_or_sentinel(self) -> Option<usize> {
        unsafe {
            memory::read_ptr(self.ptr + addresses::fields::police_script_owner::ROOT_OR_SENTINEL)
        }
    }

    /// `+0x08` -> state/count-like field.
    pub fn count_or_state(self) -> Option<usize> {
        unsafe {
            memory::read_value::<usize>(
                self.ptr + addresses::fields::police_script_owner::COUNT_OR_STATE,
            )
        }
    }

    /// `+0x10` -> active child ptr.
    pub fn active_child_ptr(self) -> Option<usize> {
        unsafe {
            let raw = memory::read_ptr_raw(
                self.ptr + addresses::fields::police_script_owner::ACTIVE_CHILD,
            )?;
            if raw == 0 { None } else { Some(raw) }
        }
    }

    /// Active child как `EntityRef`, если pointer валиден как entity-like object.
    pub fn active_child_entity(self) -> Option<EntityRef> {
        EntityRef::from_ptr(self.active_child_ptr()?)
    }

    /// Active child как `ScriptEntity`.
    ///
    /// Использует `new_unchecked`, потому что это owner-owned child path,
    /// а не только top-level `type=0x62` из world DB.
    pub fn active_child_script(self) -> Option<ScriptEntity> {
        Some(ScriptEntity::new_unchecked(self.active_child_ptr()?))
    }

    /// Есть ли сейчас активный child.
    pub fn has_child(self) -> bool {
        self.active_child_ptr().is_some()
    }

    // =============================================================================
    //  Mutating owner / child operations
    // =============================================================================

    /// Инициализировать child через owner path.
    ///
    /// По текущему reverse'у:
    /// - создаёт direct police-script child
    /// - сохраняет его по `owner + 0x10`
    /// - вызывает generic entity post-create activation
    pub fn init_child(self) -> bool {
        type Fn = unsafe extern "C" fn(usize);
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::police_script_owner::INIT_CHILD)
        };

        unsafe { func(self.ptr) };
        self.has_child()
    }

    /// Уничтожить активный child через owner path.
    pub fn destroy_child(self) -> bool {
        type Fn = unsafe extern "C" fn(usize);
        let func: Fn = unsafe {
            memory::fn_at(base() + addresses::functions::police_script_owner::DESTROY_CHILD)
        };

        unsafe { func(self.ptr) };
        !self.has_child()
    }

    /// Передать guid/id в remove-path child'a.
    ///
    /// По текущему reverse'у:
    /// - owner берёт active_child по `+0x10`
    /// - затем вызывает child-side Lua bridge:
    ///   `RemovePoliceman(self, guid)`
    pub fn remove_policeman_by_guid(self, guid: u32) -> bool {
        type Fn = unsafe extern "C" fn(usize, u32);
        let func: Fn = unsafe {
            memory::fn_at(
                base() + addresses::functions::police_script_owner::REMOVE_POLICEMAN_BY_GUID,
            )
        };

        unsafe { func(self.ptr, guid) };
        true
    }

    // =============================================================================
    //  Отладка
    // =============================================================================

    /// Вывести подробную информацию об owner object в лог.
    pub fn log_debug(self) {
        let root = self.root_or_sentinel().unwrap_or(0);
        let state = self.count_or_state().unwrap_or(0);
        let child_ptr = self.active_child_ptr().unwrap_or(0);

        logger::info(&format!(
            "[police-owner] ptr=0x{:X} root=0x{:X} state/count=0x{:X} child=0x{:X}",
            self.ptr, root, state, child_ptr,
        ));

        if let Some(child) = self.active_child_script() {
            child.log_debug();
        }
    }
}

// =============================================================================
//  Global helpers
// =============================================================================

/// Прочитать shutdown flag owner singleton.
///
/// Поведение:
/// - `0` перед созданием owner
/// - `1` после atexit shutdown path
pub fn shutdown_flag() -> Option<u8> {
    unsafe {
        memory::read_value::<u8>(base() + addresses::globals::POLICE_SCRIPT_OWNER_SHUTDOWN_FLAG)
    }
}

/// Проверить, поставлен ли shutdown flag.
pub fn is_shutdown_flag_set() -> bool {
    shutdown_flag().unwrap_or(0) != 0
}

/// Вывести ключевые police globals в лог.
pub fn log_globals() {
    let owner = PoliceScriptOwner::get().map(|o| o.as_ptr()).unwrap_or(0);
    let flag = shutdown_flag().unwrap_or(0);

    logger::info(&format!(
        "[police-owner-globals] owner=0x{:X} shutdown_flag={}",
        owner, flag,
    ));
}
