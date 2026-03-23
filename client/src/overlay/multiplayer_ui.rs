//! Multiplayer UI — меню подключения, список игроков, чат.
//!
//! Компоненты:
//! - Окно подключения к серверу (IP, порт, никнейм)
//! - Список игроков онлайн с пингом
//! - Чат с историей сообщений
//! - Статус подключения
//!
//! Управление:
//! - F2 — открыть/закрыть меню подключения
//! - F3 — показать/скрыть список игроков
//! - T — открыть чат для ввода сообщения
//! - ESC — закрыть активное окно

use egui::{Align2, Color32, RichText, ScrollArea, TextEdit, Vec2};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use super::egui_input;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::network;

// =============================================================================
//  Состояние UI
// =============================================================================

pub struct MultiplayerUIState {
    // Видимость окон
    show_connect_menu: AtomicBool,
    show_player_list: AtomicBool,
    show_chat: AtomicBool,
    chat_input_focused: AtomicBool,

    // Данные подключения
    connection_data: Mutex<ConnectionData>,

    // Список игроков
    players: Mutex<Vec<PlayerInfo>>,

    // Чат
    chat_messages: Mutex<Vec<ChatMessage>>,
    chat_input: Mutex<String>,
}

#[derive(Clone)]
struct ConnectionData {
    server_ip: String,
    server_port: String,
    nickname: String,
    is_connected: bool,
    connection_status: String,
}

#[derive(Clone)]
pub struct PlayerInfo {
    pub id: u32,
    pub name: String,
    pub ping: u32,
    pub is_local: bool,
}

#[derive(Clone)]
struct ChatMessage {
    author: String,
    text: String,
    timestamp: String,
    is_system: bool,
}

static STATE: OnceLock<MultiplayerUIState> = OnceLock::new();

fn state() -> &'static MultiplayerUIState {
    STATE.get_or_init(|| MultiplayerUIState {
        show_connect_menu: AtomicBool::new(false),
        show_player_list: AtomicBool::new(false),
        show_chat: AtomicBool::new(false),
        chat_input_focused: AtomicBool::new(false),
        connection_data: Mutex::new(ConnectionData {
            server_ip: "127.0.0.1".to_string(),
            server_port: "7788".to_string(),
            nickname: "Player".to_string(),
            is_connected: false,
            connection_status: "Не подключен".to_string(),
        }),
        players: Mutex::new(Vec::new()),
        chat_messages: Mutex::new(Vec::new()),
        chat_input: Mutex::new(String::new()),
    })
}

// =============================================================================
//  Публичный API
// =============================================================================

/// Нужен ли захват мыши — когда открыто любое интерактивное окно.
///
/// Вызывается из render thread перед сборкой egui RawInput.
pub fn wants_mouse() -> bool {
    let s = state();
    s.show_connect_menu.load(Ordering::Relaxed)
        || s.show_player_list.load(Ordering::Relaxed)
        || s.show_chat.load(Ordering::Relaxed)
}

/// Заблокировать/разблокировать управление игроком в зависимости от состояния UI.
pub fn sync_player_controls() {
    use sdk::game::Player;
    let should_lock = wants_mouse();
    let Some(player) = Player::get_active() else { return; };
    if !player.is_ready() { return; }
    // lock_controls проверяет текущее состояние внутри — вызов идемпотентен
    player.lock_controls(should_lock);
}

/// Обработка горячих клавиш для мультиплеер UI.
pub fn handle_hotkeys() {
    let s = state();

    // F2 — меню подключения (edge detection через just_pressed)
    if egui_input::just_pressed(VK_F2) {
        let current = s.show_connect_menu.load(Ordering::Relaxed);
        s.show_connect_menu.store(!current, Ordering::Relaxed);
    }

    // F3 — список игроков
    if egui_input::just_pressed(VK_F3) {
        let current = s.show_player_list.load(Ordering::Relaxed);
        s.show_player_list.store(!current, Ordering::Relaxed);
    }

    // T — открыть чат (только если не в другом окне и чат не открыт)
    if egui_input::just_pressed(VK_T)
        && !s.show_connect_menu.load(Ordering::Relaxed)
        && !s.chat_input_focused.load(Ordering::Relaxed)
    {
        s.show_chat.store(true, Ordering::Relaxed);
        s.chat_input_focused.store(true, Ordering::Relaxed);
        // Сбрасываем очередь — чтобы 'T' не попала в поле ввода
        egui_input::flush_wndproc_queue();
    }

    // ESC — закрыть активное окно
    if egui_input::just_pressed(VK_ESCAPE) {
        if s.chat_input_focused.load(Ordering::Relaxed) {
            s.show_chat.store(false, Ordering::Relaxed);
            s.chat_input_focused.store(false, Ordering::Relaxed);
        } else if s.show_connect_menu.load(Ordering::Relaxed) {
            s.show_connect_menu.store(false, Ordering::Relaxed);
        } else if s.show_player_list.load(Ordering::Relaxed) {
            s.show_player_list.store(false, Ordering::Relaxed);
        }
    }
}

/// Добавить игрока в список.
pub fn add_player(id: u32, name: String, ping: u32, is_local: bool) {
    if let Ok(mut players) = state().players.lock() {
        players.push(PlayerInfo {
            id,
            name,
            ping,
            is_local,
        });
    }
}

/// Удалить игрока из списка.
pub fn remove_player(id: u32) {
    if let Ok(mut players) = state().players.lock() {
        players.retain(|p| p.id != id);
    }
}

/// Обновить пинг игрока.
pub fn update_player_ping(id: u32, ping: u32) {
    if let Ok(mut players) = state().players.lock() {
        if let Some(player) = players.iter_mut().find(|p| p.id == id) {
            player.ping = ping;
        }
    }
}

/// Добавить сообщение в чат.
pub fn add_chat_message(author: String, text: String) {
    if let Ok(mut messages) = state().chat_messages.lock() {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        messages.push(ChatMessage {
            author,
            text,
            timestamp,
            is_system: false,
        });

        // Ограничиваем историю чата 100 сообщениями
        if messages.len() > 100 {
            messages.remove(0);
        }
    }
}

/// Добавить системное сообщение в чат.
pub fn add_system_message(text: String) {
    let notification_text = format!("СИСТЕМА: {}", text);

    if let Ok(mut messages) = state().chat_messages.lock() {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        messages.push(ChatMessage {
            author: "СИСТЕМА".to_string(),
            text: text.clone(),
            timestamp,
            is_system: true,
        });

        if messages.len() > 100 {
            messages.remove(0);
        }
    }

    // Также показываем как уведомление
    super::state::show_notification(&notification_text);
}

/// Установить статус подключения.
pub fn set_connection_status(connected: bool, status: String) {
    if let Ok(mut data) = state().connection_data.lock() {
        data.is_connected = connected;
        data.connection_status = status.clone();
    }

    if connected {
        super::state::show_notification("Подключено к серверу!");
    } else {
        super::state::show_notification(&status);
    }
}

/// Очистить список игроков.
pub fn clear_players() {
    if let Ok(mut players) = state().players.lock() {
        players.clear();
    }
}

// =============================================================================
//  Рендер UI
// =============================================================================

/// Рисует все окна мультиплеера.
pub fn draw_multiplayer_ui(ctx: &egui::Context) {
    let s = state();

    // Меню подключения
    if s.show_connect_menu.load(Ordering::Relaxed) {
        draw_connect_menu(ctx);
    }

    // Список игроков (компактный, справа сверху)
    if s.show_player_list.load(Ordering::Relaxed) {
        draw_player_list(ctx);
    }

    // Чат (слева снизу)
    if s.show_chat.load(Ordering::Relaxed) {
        draw_chat(ctx);
    }

    // Статус подключения (справа снизу, всегда видим если подключены)
    if let Ok(data) = s.connection_data.lock() {
        if data.is_connected {
            draw_connection_status(ctx, &data.connection_status);
        }
    }
}

// =============================================================================
//  Окно подключения
// =============================================================================

fn draw_connect_menu(ctx: &egui::Context) {
    egui::Window::new("Подключение к серверу")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            let s = state();
            let mut data = match s.connection_data.lock() {
                Ok(d) => d,
                Err(_) => return,
            };

            ui.set_min_width(350.0);

            ui.vertical_centered(|ui| {
                ui.heading("Mafia II: Defencho Edition Multiplayer");
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(10.0);

            // Поля ввода
            ui.horizontal(|ui| {
                ui.label("IP адрес:");
                ui.add(
                    TextEdit::singleline(&mut data.server_ip)
                        .desired_width(200.0)
                        .hint_text("127.0.0.1"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Порт:      ");
                ui.add(
                    TextEdit::singleline(&mut data.server_port)
                        .desired_width(200.0)
                        .hint_text("7777"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Никнейм:");
                ui.add(
                    TextEdit::singleline(&mut data.nickname)
                        .desired_width(200.0)
                        .hint_text("Player"),
                );
            });

            ui.add_space(10.0);

            // Статус
            let status_color = if data.is_connected {
                Color32::from_rgb(100, 255, 100)
            } else if data.connection_status.contains("Подключение") {
                Color32::from_rgb(255, 220, 120)
            } else {
                Color32::from_rgb(255, 100, 100)
            };

            ui.horizontal(|ui| {
                ui.label("Статус:");
                ui.label(
                    RichText::new(&data.connection_status)
                        .color(status_color)
                        .strong(),
                );
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Кнопки
            ui.horizontal(|ui| {
                if !data.is_connected {
                    if ui.button("Подключиться").clicked() {
                        let port: u16 = data.server_port.parse().unwrap_or(protocol::DEFAULT_PORT);
                        let ip = data.server_ip.clone();
                        let nick = data.nickname.clone();
                        drop(data);
                        network::connect(&ip, port, &nick);
                        s.show_connect_menu.store(false, Ordering::Relaxed);
                        return;
                    }
                } else {
                    if ui.button("Отключиться").clicked() {
                        drop(data);
                        network::disconnect();
                        s.show_connect_menu.store(false, Ordering::Relaxed);
                        return;
                    }
                }

                if ui.button("Закрыть").clicked() {
                    s.show_connect_menu.store(false, Ordering::Relaxed);
                }
            });

            ui.add_space(5.0);

            // Подсказки
            ui.separator();
            ui.label(
                RichText::new("F2 - меню | F3 - игроки | T - чат")
                    .size(11.0)
                    .color(Color32::GRAY),
            );
        });
}

// =============================================================================
//  Список игроков
// =============================================================================

fn draw_player_list(ctx: &egui::Context) {
    egui::Window::new("Игроки онлайн")
        .anchor(Align2::RIGHT_TOP, Vec2::new(-10.0, 10.0))
        .resizable(false)
        .collapsible(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            let s = state();
            let players = match s.players.lock() {
                Ok(p) => p,
                Err(_) => return,
            };

            if players.is_empty() {
                ui.label(
                    RichText::new("Нет игроков онлайн")
                        .color(Color32::GRAY)
                        .italics(),
                );
                return;
            }

            ui.label(format!("Всего: {}", players.len()));
            ui.separator();

            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for player in players.iter() {
                    ui.horizontal(|ui| {
                        // Иконка локального игрока
                        if player.is_local {
                            ui.label(RichText::new("[ВЫ]").color(Color32::GOLD));
                        }

                        // Имя игрока
                        ui.label(&player.name);

                        // Пинг
                        let ping_color = match player.ping {
                            0..=50 => Color32::from_rgb(100, 255, 100),
                            51..=100 => Color32::from_rgb(255, 200, 100),
                            _ => Color32::from_rgb(255, 100, 100),
                        };

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{}ms", player.ping))
                                    .color(ping_color)
                                    .monospace(),
                            );
                        });
                    });
                }
            });
        });
}

// =============================================================================
//  Чат
// =============================================================================

fn draw_chat(ctx: &egui::Context) {
    egui::Window::new("Чат")
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(10.0, -10.0))
        .resizable(false)
        .collapsible(false)
        .default_width(400.0)
        .default_height(300.0)
        .show(ctx, |ui| {
            let s = state();

            // История сообщений
            ScrollArea::vertical()
                .max_height(220.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let messages = match s.chat_messages.lock() {
                        Ok(m) => m,
                        Err(_) => return,
                    };

                    if messages.is_empty() {
                        ui.label(
                            RichText::new("Нет сообщений")
                                .color(Color32::GRAY)
                                .italics(),
                        );
                        return;
                    }

                    for msg in messages.iter() {
                        ui.horizontal(|ui| {
                            // Время
                            ui.label(
                                RichText::new(&msg.timestamp)
                                    .size(10.0)
                                    .color(Color32::DARK_GRAY)
                                    .monospace(),
                            );

                            // Автор и текст
                            if msg.is_system {
                                ui.label(
                                    RichText::new(format!("[{}] {}", msg.author, msg.text))
                                        .color(Color32::from_rgb(255, 200, 100))
                                        .strong(),
                                );
                            } else {
                                ui.label(
                                    RichText::new(format!("{}: ", msg.author))
                                        .color(Color32::from_rgb(100, 180, 255))
                                        .strong(),
                                );
                                ui.label(&msg.text);
                            }
                        });
                    }
                });

            ui.separator();

            // Поле ввода
            // TODO: Подумать выглядит сыровато но всё равно
            let mut input = match s.chat_input.lock() {
                Ok(i) => i,
                Err(_) => return,
            };

            let response = ui.add(
                TextEdit::singleline(&mut *input)
                    .desired_width(ui.available_width())
                    .hint_text("Введите сообщение... (Enter - отправить, ESC - закрыть)"),
            );

            // Автофокус при открытии
            if s.chat_input_focused.load(Ordering::Relaxed) {
                response.request_focus();
            }

            let enter_pressed = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let enter_lost = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if enter_pressed || enter_lost {
                if !input.trim().is_empty() {
                    let text = input.trim().to_string();
                    input.clear();
                    drop(input);
                    // Показываем локально сразу, не ждём эхо от сервера
                    let nickname = state()
                        .connection_data
                        .lock()
                        .map(|d| d.nickname.clone())
                        .unwrap_or_else(|_| "Я".to_string());
                    add_chat_message(nickname, text.clone());
                    network::send_chat_message(text);
                    s.show_chat.store(false, Ordering::Relaxed);
                    s.chat_input_focused.store(false, Ordering::Relaxed);
                    return;
                }
                s.show_chat.store(false, Ordering::Relaxed);
                s.chat_input_focused.store(false, Ordering::Relaxed);
            }
        });
}

// =============================================================================
//  Статус подключения
// =============================================================================

fn draw_connection_status(ctx: &egui::Context, status: &str) {
    egui::Area::new(egui::Id::new("connection_status"))
        .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-10.0, -10.0))
        .interactable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("[ОНЛАЙН]").color(Color32::from_rgb(100, 255, 100)));
                ui.label(
                    RichText::new(status)
                        .size(12.0)
                        .color(Color32::from_rgb(200, 200, 200)),
                );
            });
        });
}
