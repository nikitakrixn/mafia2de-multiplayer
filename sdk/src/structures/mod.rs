//! repr(C) структуры движка Mafia II: Definitive Edition.

pub mod std_vector;
pub mod vtables;

mod application;
mod actors_pack;
mod mission;
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
pub use vtables::car::CCarVTable;
pub use vtables::player::CHumanVTable;

pub use entity::{
    CActor, CEntity, CEntityDBRecord, CEntityGuid, CScriptWrapper, CScriptWrapperManager,
    CServiceIdentity, CTypeDescriptor, CWrapperFactory,
};

pub use application::CApplication;
pub use actors_pack::CActorsPack;
pub use mission::CMission;
pub use car::{CCar, CCarDamageSub1, CCarVehicle};
pub use game_world::{
    ENTITY_SLOT_COUNT, ENTITY_SLOT_PLAYER, EntitySlot, GameManager, GameStateFlag,
};
pub use garage::{CGarage, CGarageManager};
pub use inventory::{Inventory, InventoryData, InventorySlot, MoneyValue};
pub use player::{CHuman, CHumanNPC, CPlayer, CPlayerSub45C};
pub use tables::TableManager;
pub use vehicle::{Vehicle, VehicleWrapper};

pub use callbacks::{
    CallbackEventDesc, CallbackFunctionEntry, DispatchContext, DispatchTimer, GameCallbackManager,
    PendingFunctionOp,
};

pub use messages::{
    DamageMessage, DamageMessagePayload, DeathMessage, DeathMessagePayload, EntityMessageHeader,
    StanceMessage, StanceMessagePayload, WeaponMessage, WeaponMessagePayload,
};

pub use police_script_owner::{PoliceScriptOwner, PoliceScriptOwnerNode};
pub use render::{CRenderDeviceD3D11, RenderInitConfig, SwapChainManager, SwapChainWrapper};
pub use script_entity::{CScriptEntity, CScriptEntityChildEx};
