//! High-level доступ к ScriptEntity family.
//!
//! Этот модуль даёт безопасную и удобную обёртку над native
//! `C_ScriptEntity`-подобными объектами.
//!
//! Он НЕ пытается "угадать" все возможные подклассы,
//! а работает только с тем, что уже подтверждено реверсом:
//! - базовый `C_ScriptEntity`
//! - direct police-script child path (`Sub5`)
//! - child-пути, связанные с DamageZone / TelephoneReg / PhoneCalls
//!
//! Полезно для:
//! - runtime инспекции script-entity объектов
//! - devtools / debug dump
//! - police-script owner ветки
//! - аккуратного чтения полей `+0x78/+0x7C/+0x80/+0x88`

use common::logger;

use crate::game::{base, entity_ref::EntityRef, entity_types::FactoryType};
use crate::memory::Ptr;
use crate::structures::CScriptEntity;
use crate::{addresses, memory};

/// Дружественная обёртка над native script-entity-like объектом.
///
/// ВАЖНО:
/// это может быть:
/// - top-level `C_ScriptEntity` (`type=0x62`)
/// - direct police child path
/// - другой child path, если ты получил его pointer вручную
///
/// Поэтому есть два конструктора:
/// - `from_ptr_checked()` — только для top-level `0x62`
/// - `new_unchecked()` — если указатель уже получен из подтверждённого reverse-path
#[derive(Debug, Clone, Copy)]
pub struct ScriptEntity {
    ptr: Ptr<CScriptEntity>,
}

/// Человеко-читаемая классификация script-entity family по vtable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptEntityKind {
    /// Базовый `C_ScriptEntity`
    Base,
    /// Child внутри `C_DamageZone`
    DamageZoneChild,
    /// Child внутри `C_TelephoneReg`
    TelephoneRegChild,
    /// PhoneCalls child / host path
    PhoneCallsChild,
    /// Direct police-script child path (`Sub5`)
    PoliceChild,
    /// Неизвестная vtable family
    Unknown,
}

impl ScriptEntityKind {
    /// Имя для логов.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Base => "SCRIPT_ENTITY_BASE",
            Self::DamageZoneChild => "SCRIPT_ENTITY_DAMAGE_ZONE_CHILD",
            Self::TelephoneRegChild => "SCRIPT_ENTITY_TELEPHONE_REG_CHILD",
            Self::PhoneCallsChild => "SCRIPT_ENTITY_PHONECALLS_CHILD",
            Self::PoliceChild => "SCRIPT_ENTITY_POLICE_CHILD",
            Self::Unknown => "SCRIPT_ENTITY_UNKNOWN",
        }
    }
}

impl ScriptEntity {
    // =============================================================================
    //  Создание обёртки
    // =============================================================================

    /// Создать `ScriptEntity` из pointer с проверкой top-level типа.
    ///
    /// Подходит для объектов из `EntityDatabase`, где ожидается
    /// factory type `0x62`.
    pub fn from_ptr_checked(ptr: usize) -> Option<Self> {
        let ent = EntityRef::from_ptr(ptr)?;
        let ft = ent.factory_type()?;

        if ft != FactoryType::ScriptEntity {
            return None;
        }

        Some(Self { ptr: Ptr::new(ptr) })
    }

    /// Создать `ScriptEntity` без проверки factory type.
    ///
    /// Использовать только если pointer уже получен из подтверждённого
    /// child-path reverse'а.
    pub fn new_unchecked(ptr: usize) -> Self {
        Self { ptr: Ptr::new(ptr) }
    }

    /// Создать из `EntityRef`, если это top-level script entity.
    pub fn from_entity_ref(ent: EntityRef) -> Option<Self> {
        let ft = ent.factory_type()?;
        if ft != FactoryType::ScriptEntity {
            return None;
        }
        Some(Self {
            ptr: Ptr::new(ent.ptr()),
        })
    }

    // =============================================================================
    //  Базовые аксессоры
    // =============================================================================

    unsafe fn native(&self) -> Option<&CScriptEntity> {
        unsafe { self.ptr.as_ref() }
    }

    /// Сырой указатель на native объект.
    pub fn as_ptr(self) -> usize {
        self.ptr.addr()
    }

    /// Vtable pointer.
    pub fn vtable(self) -> Option<usize> {
        let se = unsafe { self.native()? };
        Some(se.base.vtable as usize)
    }

    /// Packed table_id.
    pub fn table_id(self) -> Option<u32> {
        let se = unsafe { self.native()? };
        Some(se.base.table_id)
    }

    /// Factory type byte.
    pub fn factory_type_byte(self) -> Option<u8> {
        let se = unsafe { self.native()? };
        Some(se.base.factory_type())
    }

    /// FactoryType enum.
    pub fn factory_type(self) -> Option<FactoryType> {
        FactoryType::from_byte(self.factory_type_byte()?)
    }

    /// Entity flags.
    pub fn entity_flags(self) -> Option<u32> {
        let se = unsafe { self.native()? };
        Some(se.base.entity_flags)
    }

    /// Name hash.
    pub fn name_hash(self) -> Option<u64> {
        let se = unsafe { self.native()? };
        Some(se.base.name_hash)
    }

    // =============================================================================
    //  ScriptEntity-specific поля
    // =============================================================================

    /// `+0x78` -> script slot / entry id.
    ///
    /// В direct police child path участвует как индекс в `scripts[...]`.
    pub fn script_entry_id(self) -> Option<u32> {
        let se = unsafe { self.native()? };
        Some(se.script_entry_id)
    }

    /// `+0x7C` -> script context index / selector.
    ///
    /// В police child path низкий байт читается как:
    /// `movzx ecx, byte ptr [this+7Ch]`.
    pub fn script_context_index(self) -> Option<u32> {
        let se = unsafe { self.native()? };
        Some(se.script_context_index)
    }

    /// `+0x80` -> дополнительное code/state-like поле.
    pub fn aux_code_or_state(self) -> Option<i32> {
        let se = unsafe { self.native()? };
        Some(se.aux_code_or_state)
    }

    /// `+0x88` -> provider/list-like pointer.
    pub fn script_provider_or_list(self) -> Option<usize> {
        let se = unsafe { self.native()? };
        let ptr = se.script_provider_or_list as usize;
        memory::is_valid_ptr(ptr).then_some(ptr)
    }

    // =============================================================================
    //  Классификация по vtable
    // =============================================================================

    /// Определить family-kind по vtable.
    pub fn kind(self) -> ScriptEntityKind {
        let Some(vt) = self.vtable() else {
            return ScriptEntityKind::Unknown;
        };

        let rva = vt.wrapping_sub(base());

        match rva {
            x if x == addresses::vtables::script_entity::BASE => ScriptEntityKind::Base,
            x if x == addresses::vtables::script_entity::DAMAGE_ZONE_CHILD => {
                ScriptEntityKind::DamageZoneChild
            }
            x if x == addresses::vtables::script_entity::TELEPHONE_REG_CHILD => {
                ScriptEntityKind::TelephoneRegChild
            }
            x if x == addresses::vtables::script_entity::PHONECALLS_CHILD => {
                ScriptEntityKind::PhoneCallsChild
            }
            x if x == addresses::vtables::script_entity::SUB5 => ScriptEntityKind::PoliceChild,
            _ => ScriptEntityKind::Unknown,
        }
    }

    /// Является ли direct police child path.
    pub fn is_police_child(self) -> bool {
        self.kind() == ScriptEntityKind::PoliceChild
    }

    // =============================================================================
    //  Отладка
    // =============================================================================

    /// Вывести подробную информацию в лог.
    pub fn log_debug(self) {
        let vt = self.vtable().unwrap_or(0);
        let tid = self.table_id().unwrap_or(0);
        let ft = self
            .factory_type()
            .map(|f| f.display_name())
            .unwrap_or("UNKNOWN");

        logger::info(&format!(
            "[script-entity] ptr=0x{:X} vt=0x{:X} table_id=0x{:08X} ft={} kind={}",
            self.ptr,
            vt,
            tid,
            ft,
            self.kind().display_name(),
        ));

        logger::info(&format!(
            "  entry_id={} ctx={} aux={} provider=0x{:X}",
            self.script_entry_id().unwrap_or(0),
            self.script_context_index().unwrap_or(0),
            self.aux_code_or_state().unwrap_or(0),
            self.script_provider_or_list().unwrap_or(0),
        ));
    }
}
