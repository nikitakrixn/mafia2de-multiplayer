//! repr(C) структуры движка Mafia II: Definitive Edition.

pub mod std_vector;
pub mod vtables;

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

pub use std_vector::StdVector;
pub use vtables::CPlayerVTable;

pub use entity::{
    CActor, CEntity, CEntityDBRecord, CEntityGuid, CScriptWrapper,
    CScriptWrapperManager, CServiceIdentity, CTypeDescriptor, CWrapperFactory,
};

pub use game_world::GameManager;
pub use player::{CHuman, CHumanNPC, CPlayer, CPlayerSub45C};
pub use inventory::{Inventory, InventoryData, InventorySlot, MoneyValue};
pub use car::{CCar, CCarDamageSub1, CCarVehicle};
pub use vehicle::{Vehicle, VehicleWrapper};
pub use garage::{CGarage, CGarageManager};
pub use tables::TableManager;

pub use callbacks::{
    CallbackEventDesc, CallbackFunctionEntry, DispatchContext, DispatchTimer,
    GameCallbackManager, PendingFunctionOp,
};

pub use messages::{
    DamageMessage, DamageMessagePayload, DeathMessage, DeathMessagePayload,
    EntityMessageHeader, StanceMessage, StanceMessagePayload, WeaponMessage,
    WeaponMessagePayload,
};

pub use render::{CRenderDeviceD3D11, RenderInitConfig, SwapChainManager, SwapChainWrapper};
pub use script_entity::{CScriptEntity, CScriptEntityChildEx};
pub use police_script_owner::{PoliceScriptOwner, PoliceScriptOwnerNode};