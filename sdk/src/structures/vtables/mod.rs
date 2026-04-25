//! Типизированные vtable-описания для основных классов движка.
//!
//! Каждый модуль содержит `#[repr(C)]` struct, описывающий layout
//! виртуальной таблицы конкретного класса. Смещения проверяются
//! compile-time ассертами.

// Entity vtables (humanoids, vehicles, AI) — в отдельном подмодуле
pub mod entities;

// Системные vtables (не entity)
pub mod application;
pub mod actors_pack;
pub mod c_sys_input;
pub mod game_input_module;
pub mod game_manager;
pub mod mission;
