//! repr(C) структуры движка Mafia II: Definitive Edition.
//!
//! Все структуры восстановлены из IDA Pro и проверены
//! compile-time ассертами на смещения полей.

mod game_world;
mod player;
mod inventory;
mod vehicle;
mod car;
mod garage;
mod tables;
mod callbacks;
mod messages;
mod render;

// Корневые типы
pub use game_world::GameManager;
pub use player::CHuman;

// Инвентарь и деньги
pub use inventory::{Inventory, InventorySlot, InventoryData, MoneyValue};

// Транспорт
pub use vehicle::{Vehicle, VehicleWrapper};
pub use car::{CCar, CarData};

// Гараж
pub use garage::{CGarageManager, CGarage};

// Таблицы данных
pub use tables::TableManager;

// Callback-система (много связанных типов)
pub use callbacks::{
    GameCallbackManager, CallbackEventDesc, CallbackFunctionEntry,
    PendingFunctionOp, DispatchContext, DispatchTimer,
};

// Entity/human сообщения
pub use messages::{
    EntityMessageHeader,
    DamageMessage, DamageMessagePayload,
    DeathMessage, DeathMessagePayload,
    WeaponMessage, WeaponMessagePayload,
    StanceMessage, StanceMessagePayload,
};

// DX11 рендер
pub use render::{
    CRenderDeviceD3D11, RenderInitConfig,
    SwapChainManager, SwapChainWrapper,
};