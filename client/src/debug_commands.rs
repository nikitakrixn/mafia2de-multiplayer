//! Тестовые команды разработчика
//!
//! Всё здесь — отладочноеное меню для проверки работы SDK и игровых функций.

use common::logger;
use sdk::game::Player;

/// Заблокировать/разблокировать управление игроком.
pub fn lock_controls(player: &Player, lock: bool) {
    let action = if lock { "Блокирую" } else { "Разблокирую" };
    logger::info(&format!("{action} управление..."));

    if player.lock_controls(lock) {
        let state = if lock { "ЗАБЛОКИРОВАНО" } else { "РАЗБЛОКИРОВАНО" };
        logger::info(&format!("  → Управление {state}"));
    } else {
        logger::error("  → Не удалось изменить состояние управления");
    }
}

/// Вывести текущий статус игрока: управление, позиция, хуки.
pub fn show_status(player: &Player) {
    logger::info(&format!("Хуки установлены: {}", crate::hooks::is_installed()));
    logger::info(&format!("Фокус окна: {:?}", crate::events::app_focus_state()));
    logger::info(&format!("Состояние сессии: {}", crate::state::get().as_str()));

    match player.are_controls_locked() {
        Some(locked) => logger::info(&format!("Управление заблокировано: {locked}")),
        None => logger::error("Не удалось прочитать состояние управления"),
    }

    match player.get_control_style_str() {
        Some(style) => logger::info(&format!("Стиль управления: \"{style}\"")),
        None => logger::error("Стиль управления: недоступен"),
    }

    match player.get_position() {
        Some(pos) => logger::info(&format!("Позиция: {pos}")),
        None => logger::error("Позиция: недоступна"),
    }
}

/// Телепортировать игрока в заданную точку.
/// Координаты захардкожены — это временная отладка.
pub fn teleport(player: &Player) {
    match player.get_position() {
        Some(mut pos) => {
            logger::info(&format!("Текущая позиция: {pos}"));

            // TODO: вынести координаты в конфиг или консольную команду
            pos.x = 1261.0;
            pos.y = 1169.0;
            pos.z = 0.5;

            if player.set_position(&pos) {
                logger::info(&format!("Телепортирован в: {pos}"));
            } else {
                logger::error("Телепорт не удался");
            }
        }
        None => logger::error("Позиция недоступна для телепорта"),
    }
}

/// Добавить или снять деньги (в долларах, со знаком).
pub fn add_money(player: &Player, dollars: i32) {
    let action = if dollars >= 0 { "Добавляю" } else { "Снимаю" };
    logger::info(&format!("{action} ${}", dollars.abs()));

    // Пробуем через игровую функцию с HUD-уведомлением.
    // Если HUD ещё не готов — fallback на прямую запись в память.
    if player.add_money_dollars_with_hud(dollars) {
        show_balance(player);
    } else {
        logger::warn("HUD недоступен, пишу напрямую в память");
        match player.add_money_dollars(dollars) {
            Some(new) => logger::info(&format!(
                "  → Баланс: $ {}.{:02}", new / 100, (new % 100).abs()
            )),
            None => logger::error("  → Кошелёк не инициализирован"),
        }
    }
}

/// Установить деньги напрямую (в центах).
pub fn set_money(player: &Player, cents: i64) {
    logger::info(&format!("Устанавливаю баланс: $ {}.{:02}", cents / 100, (cents % 100).abs()));
    player.set_money(cents);
    show_balance(player);
}

/// Показать текущий баланс.
pub fn show_balance(player: &Player) {
    match player.get_money_cents() {
        Some(c) => logger::info(&format!(
            "Баланс: {} центов = $ {}.{:02}",
            c, c / 100, (c % 100).abs()
        )),
        None => logger::info("Баланс: кошелёк не инициализирован"),
    }
}

/// Выдать оружие с патронами.
pub fn give_weapon(player: &Player, weapon_id: u32, ammo: u32, name: &str) {
    logger::info(&format!("Выдаю {name} + {ammo} патронов"));
    if player.add_weapon(weapon_id, ammo) {
        logger::info(&format!("  → {name} добавлен!"));
    } else {
        logger::error(&format!("  → Не удалось добавить {name}"));
    }
}

/// Показать текущий FOV.
pub fn show_fov() {
    sdk::game::camera::log_status();
}

/// Изменить FOV на delta.
pub fn adjust_fov(delta: f32) {
    let current = sdk::game::camera::get_interier_fov().unwrap_or(65.0);
    let new_fov = (current + delta).clamp(30.0, 150.0);
    logger::info(&format!("FOV: {current:.1} → {new_fov:.1}"));
    sdk::game::camera::set_all_fov(new_fov);
}

/// Установить FOV для ВСЕХ камер (player + car).
pub fn set_fov(fov: f32) {
    logger::info(&format!("Устанавливаю FOV для всех камер: {fov:.1}"));
    if sdk::game::camera::set_all_fov(fov) {
        logger::info(&format!("  → FOV = {fov:.1} (player + car)"));
    } else {
        logger::error("  → Не удалось установить FOV");
    }
}