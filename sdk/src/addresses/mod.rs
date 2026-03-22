//! Все известные адреса из Mafia II: Definitive Edition.
//!
//! Адреса хранятся как **RVA** (Relative Virtual Address).
//!
//! ```text
//! RVA = IDA_address - IMAGE_BASE
//! runtime_address = module_base + RVA
//! ```

pub mod constants;
pub mod data;
pub mod fields;
pub mod functions;
pub mod globals;
pub mod strings;
pub mod vtables;

/// Имя основного исполняемого модуля.
pub const GAME_MODULE: &str = "Mafia II Definitive Edition.exe";

/// Базовый адрес образа в IDA (стандарт для x64 PE).
pub const IMAGE_BASE: usize = 0x1_4000_0000;

/// Размер модуля
/// TODO: уточнить размер, проверить на разных версиях игры. Временная заглушка.
pub const GAME_MODULE_SIZE: usize = 0x389_8000;
