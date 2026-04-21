//! Типизированные vtable-описания для основных классов движка.
//!
//! Каждый модуль содержит `#[repr(C)]` struct, описывающий layout
//! виртуальной таблицы конкретного класса. Смещения проверяются
//! compile-time ассертами.

pub mod ai;
pub mod application;
pub mod actors_pack;
pub mod c_sys_input;
pub mod car;
pub mod game_input_module;
pub mod game_manager;
pub mod mission;
pub mod player;
