//! Типизированные vtable-описания для основных классов движка.
//!
//! Каждый модуль содержит `#[repr(C)]` struct, описывающий layout
//! виртуальной таблицы конкретного класса. Смещения проверяются
//! compile-time ассертами.

pub mod application;
pub mod car;
pub mod game_manager;
pub mod player;
