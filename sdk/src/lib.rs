//! # Mafia II: DE SDK
//!
//! ## Модули
//!
//! - [`addresses`] — все RVA-адреса, смещения, константы (из IDA Pro)
//! - [`structures`] — `repr(C)` структуры игры
//! - [`game`] — высокоуровневый API (Player, Vehicle, Garage...)
//! - [`memory`] — чтение/запись памяти
//! - [`patterns`] — сканер сигнатур
//!
//! ## Пример
//!
//! ```ignore
//! use sdk::game::Player;
//! use sdk::addresses::constants::weapons;
//!
//! let player = Player::get_active().unwrap();
//! player.add_weapon(weapons::THOMPSON_1928, 200);
//! player.add_money(10_000);
//! ```

pub mod addresses;
pub mod structures;
pub mod game;
pub mod memory;
pub mod patterns;