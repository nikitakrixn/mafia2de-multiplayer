// //! Обработка пользовательского ввода (клавиши).
// //!
// //! Работает в отдельном потоке — опрашивает GetAsyncKeyState
// //! каждые INPUT_POLL_MS. Тестовые команды разработчика живут
// //! в debug_commands, чтобы их было легко убрать перед релизом.

// use std::time::Duration;

// use windows::Win32::UI::Input::KeyboardAndMouse::{
//     GetAsyncKeyState, VK_INSERT, VK_DELETE,
//     VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6,
//     VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
//     VK_ADD, VK_SUBTRACT, VK_MULTIPLY, VK_DIVIDE,
//     VIRTUAL_KEY,
// };

// use common::logger;
// use sdk::game::Player;

// use crate::{debug_commands, lua_queue, runtime, player_probe};

// const INPUT_POLL_MS: u64 = 100;

// /// Проверяет, была ли клавиша нажата с момента последнего опроса.
// /// Бит 0 в результате GetAsyncKeyState — "нажата с прошлого вызова".
// fn just_pressed(vk: VIRTUAL_KEY) -> bool {
//     let state = unsafe { GetAsyncKeyState(vk.0 as i32) };
//     (state & 0x0001) != 0
// }

// /// Вывести в лог список привязок клавиш.
// pub fn log_keybinds() {
//     logger::info("  Клавиши:");
//     logger::info("    INSERT — Lua-команда через main-thread");
//     logger::info("    DELETE — Выключить хуки/рантайм");
//     logger::info("    F1     — Заблокировать управление");
//     logger::info("    F2     — Разблокировать управление");
//     logger::info("    F3     — Статус (управление + позиция)");
//     logger::info("    F4     — Телепорт");
//     logger::info("    F5-F8  — Деньги (+100, +500, +1000, -500)");
//     logger::info("    F9     — Установить $9999.99");
//     logger::info("    F10    — Показать баланс");
//     logger::info("    F11    — Дать Thompson + 200 патронов");
//     logger::info("    F12    — Дать Colt 1911 + 50 патронов");
//     logger::info("    Num+   — FOV +5");
//     logger::info("    Num-   — FOV -5");
//     logger::info("    Num*   — Показать FOV / сброс на 65");
//     logger::info("    Num/   — Установить FOV на 75");
// }

// /// Главный цикл обработки ввода.
// /// Вызывается из initialize(), блокирует поток до shutdown.
// pub fn run() {
//     logger::debug("[input] цикл ввода запущен");

//     loop {
//         if runtime::is_shutting_down() {
//             logger::debug("[input] остановка");
//             break;
//         }

//         std::thread::sleep(Duration::from_millis(INPUT_POLL_MS));

//         // Системные клавиши — работают всегда
//         if just_pressed(VK_INSERT) {
//             logger::info("Отправляю Lua-команду в очередь main-thread...");
//             lua_queue::queue_exec_named(
//                 r#"
//         local joe = game.entitywrapper:GetEntityByName("Joe")
//         if joe then
//             local player = game.game:GetActivePlayer()
//             local pos = player:GetPos()
//             joe:SetPos(Math:newVector(pos.x + 2, pos.y, pos.z))
//             joe:Activate()
//             joe:InventoryAddWeapon(11, 250)
//             joe:SetAggressivity(4)
//             joe.invulnerability = true
//             joe:SetCarAttackPermission(true)
//             joe:Follow(player, "RUN", 2, 3.5, true)
//         end
//     "#,
//                 "=m2mp_insert_test",
//             );
//         }

//         if just_pressed(VK_DELETE) {
//             logger::info("Ручное завершение работы...");
//             runtime::shutdown();
//             break;
//         }

//         // Команды, требующие активного игрока
//         let Some(player) = Player::get_active() else {
//             continue;
//         };

//         if just_pressed(VK_F1)  {
//     debug_commands::dump_entity_factory_registry(); }
//         if just_pressed(VK_F2)  { 
//             debug_commands::scan_all_cached_entities(); }
//         if just_pressed(VK_F3)  { debug_commands::dump_sds_config_v2(); }
//         if just_pressed(VK_F4)  { debug_commands::dump_sds_line_manager(); }
//         if just_pressed(VK_F5)  { debug_commands::init_map_for_mapping("winter", "snow"); }
//         if just_pressed(VK_F6)  { debug_commands::dump_sds_lines(); }
//         if just_pressed(VK_F7)  { 
//             //sdk::game::sds::load_sds_file("/sds/Cars/Shubert_Frigate_pha.sds");
//         }
//         if just_pressed(VK_F8)  { debug_commands::add_money(&player, -500); }
//         if just_pressed(VK_F9)  { debug_commands::set_money(&player, 999_999); }
//         if just_pressed(VK_F10) { debug_commands::show_balance(&player); }
//         if just_pressed(VK_F11) { logger::info("Dumping frame directions...");
//             player_probe::dump_frame_directions(); }
//         if just_pressed(VK_F12) { logger::info("Dumping player memory range...");
//             player_probe::dump_player_range(0x00, 0x80);
// player_probe::dump_frame_directions(); }
//         if just_pressed(VK_ADD) { debug_commands::adjust_fov(5.0); }
//         if just_pressed(VK_SUBTRACT) { debug_commands::adjust_fov(-5.0); }
//         if just_pressed(VK_MULTIPLY) { debug_commands::show_fov(); }
//         if just_pressed(VK_DIVIDE)  { debug_commands::set_fov(75.0); }
//     }
// }

//! Обработка ввода клиента — только мультиплеер-функции.

use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_F9, VK_DELETE, VK_TAB, VK_RETURN,
    VIRTUAL_KEY,
};

use common::logger;
use crate::runtime;

const INPUT_POLL_MS: u64 = 100;

fn just_pressed(vk: VIRTUAL_KEY) -> bool {
    let state = unsafe { GetAsyncKeyState(vk.0 as i32) };
    (state & 0x0001) != 0
}

pub fn log_keybinds() {
    logger::info("  Keybinds:");
    logger::info("    DELETE — Выгрузить клиент");
    logger::info("    F9     — Toggle overlay");
    logger::info("    TAB    — Scoreboard (hold)");
    logger::info("    ENTER  — Открыть чат");
}

pub fn run() {
    logger::debug("[input] started");

    loop {
        if runtime::is_shutting_down() {
            break;
        }

        std::thread::sleep(Duration::from_millis(INPUT_POLL_MS));

        if just_pressed(VK_DELETE) {
            logger::info("Client shutting down...");
            runtime::shutdown();
            break;
        }

        if just_pressed(VK_F9) {
            crate::overlay::toggle_visibility();
        }

        // TAB — показать scoreboard (пока toggle, потом hold)
        if just_pressed(VK_TAB) {
            // TODO: toggle scoreboard visibility
        }

        // ENTER — открыть чат
        if just_pressed(VK_RETURN) {
            // TODO: toggle chat input
        }
    }
}