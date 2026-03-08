use std::time::{Duration, Instant};
use std::ffi::{CStr, CString};

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

    /// Добавить деньги через игровую функцию (тихо, без HUD).
    ///
    /// Использует `M2DE_Inventory_ModifyMoney(inv, cents, do_apply=1)`.
    /// Гарантировано добавляет деньги если inventory валиден.
    pub fn add_money_game(&self, cents: i64) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_money_game: inventory NULL");
            return false;
        };

        type ModifyMoneyFn = unsafe extern "C" fn(usize, i64, u8) -> u8;
        let func: ModifyMoneyFn = unsafe {
            crate::memory::fn_at(base() + addresses::functions::player::INVENTORY_MODIFY_MONEY)
        };

        unsafe { func(inv, cents, 1) };
        true
    }

    /// Показать HUD popup о деньгах (± $X.XX).
    ///
    /// **Не добавляет деньги** — только визуальный эффект.
    /// Обходит проверку entity type через прямой доступ к g_HUDManager.
    ///
    /// Безопасно вызывать если HUD не инициализирован — просто ничего не произойдёт.
    fn show_money_notification(&self, cents: i64) {
        unsafe {
            // g_HUDManager → +0x98 → money display component
            let hud_mgr = match memory::read_ptr(
                base() + addresses::globals::HUD_MANAGER
            ) {
                Some(p) => p,
                None => {
                    logger::debug("show_money_notification: HUD manager not initialized");
                    return;
                }
            };

            let money_display = match memory::read_ptr(
                hud_mgr + addresses::fields::hud_manager::MONEY_DISPLAY
            ) {
                Some(p) => p,
                None => {
                    logger::debug("show_money_notification: money display component NULL");
                    return;
                }
            };

            std::ptr::write((money_display + addresses::fields::hud_money_display::ANIM_TIMER) as *mut f32, 0.0f32);

            type UpdateFn = unsafe extern "C" fn(usize, i64, i64) -> i64;
            let update: UpdateFn = std::mem::transmute(
                base() + addresses::functions::hud::UPDATE_MONEY_COUNTER
            );

            update(money_display, cents, 0);
        }
    }

    /// Добавить деньги + показать HUD уведомление (зелёный/красный popup).
    ///
    /// Двухэтапный процесс:
    /// 1. Добавляет деньги через игровую функцию (гарантировано)
    /// 2. Показывает HUD popup через g_HUDManager (обходит проверку type==16)
    pub fn add_money_with_hud(&self, cents: i64) -> bool {
        // 1. Добавить деньги
        if !self.add_money_game(cents) {
            return false;
        }

        // 2. Показать HUD popup
        self.show_money_notification(cents);

        true
    }

    /// Добавить деньги с HUD (в долларах).
    pub fn add_money_dollars_with_hud(&self, dollars: i32) -> bool {
        self.add_money_with_hud(dollars as i64 * 100)
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Позиция
    // ═══════════════════════════════════════════════════════════════════

    /// Получить текущую мировую позицию игрока через игровую функцию.
    ///
    /// Это основной и самый надёжный способ чтения позиции.
    ///
    /// Внутри игра делает так:
    /// - сначала пробует physics/provider по `C_Human + 0x258`
    /// - если его нет, берёт позицию из frame node по `C_Human + 0x78`
    ///
    /// Таким образом мы повторяем "официальный" путь самой игры,
    /// а не читаем координаты наугад из памяти.
    ///
    /// Подтверждено:
    /// - IDA: `sub_140DA7630`
    /// - runtime проверкой против Lua `GetPos()`
    pub fn get_position(&self) -> Option<Vec3> {
        unsafe {
            let mut out = Vec3::default();

            type GetPosFn = unsafe extern "C" fn(usize, *mut Vec3) -> *mut Vec3;
            let func: GetPosFn =
                std::mem::transmute(base() + addresses::functions::entity::GET_POS);

            let ret = func(self.ptr, &mut out as *mut Vec3);
            if ret.is_null() {
                return None;
            }

            if out.x.is_finite() && out.y.is_finite() && out.z.is_finite() {
                Some(out)
            } else {
                None
            }
        }
    }

    /// Прямое чтение позиции из frame/transform node.
    ///
    /// Это debug/fallback-метод.
    /// Он полезен для сверки реверса, потому что `M2DE_Entity_GetPos`
    /// в fallback-path читает те же поля:
    ///
    /// - `frame + 0x64` = x
    /// - `frame + 0x74` = y
    /// - `frame + 0x84` = z
    ///
    /// По факту это и есть реальные world coordinates, что было подтверждено
    /// сравнением с Lua `GetPos()`.
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

    /// Установить мировую позицию игрока через штатный engine-level setter.
    ///
    /// Почему так, а не прямой записью в память:
    /// игра хранит позицию не только в одном месте. Помимо frame node,
    /// при перемещении обновляются physics и внутренние dirty/cached state.
    ///
    /// Поэтому используется `M2DE_Entity_SetPos`, который делает полный
    /// корректный путь обновления движка.
    ///
    /// Возвращает `false`, если переданы некорректные координаты.
    pub fn set_position(&self, pos: &Vec3) -> bool {
        if !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
            logger::error("set_position: non-finite coordinate");
            return false;
        }

        unsafe {
            type SetPosFn = unsafe extern "C" fn(usize, *const Vec3);
            let func: SetPosFn =
                std::mem::transmute(base() + addresses::functions::entity::SET_POS);

            func(self.ptr, pos as *const Vec3);
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Оружие
    // ═══════════════════════════════════════════════════════════════════

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
            logger::error("add_weapon: inventory NULL");
            return false;
        };

        type AddWeaponFn = unsafe extern "C" fn(usize, u32, i32) -> u8;
        let func: AddWeaponFn = unsafe {
            std::mem::transmute(base() + addresses::functions::player::INVENTORY_ADD_WEAPON_CORE)
        };

        logger::debug(&format!(
            "M2DE_Inventory_AddWeapon_Core(0x{:X}, weapon={}, ammo={})",
            inv, weapon_id, ammo,
        ));

        let result = unsafe { func(inv, weapon_id, ammo as i32) };
        if result != 0 {
            logger::debug("  → weapon added/ammo updated");
            true
        } else {
            logger::warn("  → add_weapon returned 0 (slots full or invalid ID)");
            false
        }
    }

    /// Добавить только патроны (без оружия).
    ///
    /// Патроны попадают в slot[4]. Если оружия нет — не упадёт,
    /// но патроны будут "висеть" до получения оружия.
    pub fn add_ammo(&self, weapon_id: u32, ammo: u32) -> bool {
        let Some(inv) = self.inventory_ptr() else {
            logger::error("add_ammo: inventory NULL");
            return false;
        };

        type AddAmmoFn = unsafe extern "C" fn(usize, u32, u32);
        let func: AddAmmoFn = unsafe {
            std::mem::transmute(base() + addresses::functions::player::INVENTORY_ADD_AMMO)
        };

        unsafe { func(inv, weapon_id, ammo) };
        true
    }

     pub fn control_component_ptr(&self) -> Option<usize> {
        unsafe { memory::read_ptr(self.ptr + fields::player::CONTROL_COMPONENT) }
    }
    
    // ══════════════════════════════════════════════════════════════════
    //  Контроль игрока (стиль управления, блокировка и т.д.)
    // ══════════════════════════════════════════════════════════════════

    pub fn are_controls_locked(&self) -> Option<bool> {
        let control = self.control_component_ptr()?;

        type Fn = unsafe extern "C" fn(usize) -> i64;
        let func: Fn = unsafe {
            std::mem::transmute(base() + addresses::functions::player_control::IS_LOCKED)
        };

        Some(unsafe { func(control) != 0 })
    }

    pub fn lock_controls(&self, locked: bool) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls: control component is NULL");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            std::mem::transmute(base() + addresses::functions::player_control::SET_LOCKED)
        };

        unsafe { func(control, locked as u8, 0) };
        true
    }

    pub fn lock_controls_to_play_anim(&self) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("lock_controls_to_play_anim: control component is NULL");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, u8, u8) -> i64;
        let func: Fn = unsafe {
            std::mem::transmute(base() + addresses::functions::player_control::SET_LOCKED)
        };

        unsafe { func(control, 1, 1) };
        true
    }

    pub fn get_control_style_str(&self) -> Option<String> {
        let control = self.control_component_ptr()?;

        type Fn = unsafe extern "C" fn(usize) -> *const i8;
        let func: Fn = unsafe {
            std::mem::transmute(base() + addresses::functions::player_control::GET_STYLE_STR)
        };

        let ptr = unsafe { func(control) };
        if ptr.is_null() {
            return None;
        }

        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
    }

    pub fn set_control_style_str(&self, style: &str) -> bool {
        let Some(control) = self.control_component_ptr() else {
            logger::error("set_control_style_str: control component is NULL");
            return false;
        };

        let Ok(c_style) = CString::new(style) else {
            logger::error("set_control_style_str: style contains interior NUL");
            return false;
        };

        type Fn = unsafe extern "C" fn(usize, *const i8) -> i64;
        let func: Fn = unsafe {
            std::mem::transmute(base() + addresses::functions::player_control::SET_STYLE_STR)
        };

        unsafe { func(control, c_style.as_ptr()) != 0 }
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
                    match memory::read_ptr(inv + fields::inventory::OWNER_ENTITY_REF) {
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