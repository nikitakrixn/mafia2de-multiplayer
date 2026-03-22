//! repr(C) структуры движка Mafia II: Definitive Edition.
//!
//! Все структуры восстановлены из IDA Pro и проверены
//! compile-time ассертами на смещения полей.

mod callbacks;
mod car;
mod entity;
mod game_world;
mod garage;
mod inventory;
mod messages;
mod player;
mod police_script_owner;
mod render;
mod script_entity;
mod tables;
mod vehicle;

// Entity system (DB records, wrappers, factories)
pub use entity::{
    CActorFields, CEntity, CEntityDBRecord, CEntityGuid, CScriptWrapper, CScriptWrapperManager,
    CServiceIdentity, CTypeDescriptor, CWrapperFactory,
};

// Корневые типы
pub use game_world::GameManager;
pub use player::{CHuman, CHumanNPC, CPlayer};

// Инвентарь и деньги
pub use inventory::{Inventory, InventoryData, InventorySlot, MoneyValue};

// Транспорт
pub use car::{CCar, CarData};
pub use vehicle::{Vehicle, VehicleWrapper};

// Гараж
pub use garage::{CGarage, CGarageManager};

// Таблицы данных
pub use tables::TableManager;

// Callback-система
pub use callbacks::{
    CallbackEventDesc, CallbackFunctionEntry, DispatchContext, DispatchTimer, GameCallbackManager,
    PendingFunctionOp,
};

// Entity/human сообщения
pub use messages::{
    DamageMessage, DamageMessagePayload, DeathMessage, DeathMessagePayload, EntityMessageHeader,
    StanceMessage, StanceMessagePayload, WeaponMessage, WeaponMessagePayload,
};

// DX11 рендер
pub use render::{CRenderDeviceD3D11, RenderInitConfig, SwapChainManager, SwapChainWrapper};

// ScriptEntity family
pub use script_entity::{CPoliceScriptChild, CScriptEntity};

// Police-script owner singleton
pub use police_script_owner::{PoliceScriptOwner, PoliceScriptOwnerNode};
