pub mod types;
pub mod game;
pub mod resource_manager;
pub mod physical_processor;
pub mod translocator;
pub mod free_raid;
pub mod globals;
pub mod object_list;

// Реэкспортируем основные структуры для удобства использования
pub use types::*;
pub use game::*;
pub use resource_manager::*;
pub use physical_processor::*;
pub use translocator::*;
pub use free_raid::*;
pub use globals::*;
pub use object_list::*;
