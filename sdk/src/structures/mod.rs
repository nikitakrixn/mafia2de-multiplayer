//! repr(C) структуры движка Mafia II: Definitive Edition.
//!
//! ## Организация
//!
//! - `entities/` — иерархия entity-классов (гуманоиды, машины, AI, инвентарь)
//! - `vtables/entities/` — vtable layouts для entity-классов
//! - остальные модули — системные структуры (callbacks, render, input, world...)

// Entity hierarchy — гуманоиды, машины, AI, инвентарь и их компоненты
pub mod entities;

pub mod std_vector;
pub mod vtables;

// Системные структуры (не entity)
mod application;
mod actors_pack;
mod mission;
mod entity_hash_table;
mod c_sys_input;
mod callbacks;
mod game_input_module;
mod game_world;
mod garage;
mod messages;
mod police_script_owner;
mod render;
mod tables;

pub use std_vector::StdVector;
pub use vtables::entities::car::CCarVTable;
pub use vtables::entities::player::CHumanVTable;

// Entity компоненты (из нового entities/ подмодуля)
pub use entities::entity::{
    CActor, CEntity, CEntityDBRecord, CEntityGuid, CScriptWrapper, CScriptWrapperManager,
    CServiceIdentity, CTypeDescriptor, CWrapperFactory,
};
pub use entities::player::{CHuman, CHumanNPC, CPlayer, CPlayerSub45C, CarWrapper};
pub use entities::car::{CCar, CCarDamageSub1, CCarVehicle};
pub use entities::vehicle::{Vehicle, VehicleWrapper};
pub use entities::script_entity::{CScriptEntity, CScriptEntityChildEx};
pub use entities::ai::{
    CAIController, CAITask, CHumanAIController, CHumanAIResources, CHumanAIState,
    CHumanStateVariables, CMafiaNavAgent,
};
pub use entities::human_inventory::CHumanInventory;
pub use entities::human_messages::{human_message_id, DAMAGE_MSG_ID};
pub use entities::inventory_components::{
    CInventory, CInventorySlot, CInventoryResource, CWeaponItem, CHumanThrowInventory,
};
pub use entities::human_components::{
    CFrameColors, CPlayerEmitter, CHumanHeadController, CHumanWeaponController,
};

pub use application::CApplication;
pub use actors_pack::CActorsPack;
pub use mission::CMission;
pub use entity_hash_table::EntityHashTable;
pub use c_sys_input::{CSysInput, CSysInputNode};
pub use game_input_module::CGameInputModule;
pub use game_world::{
    ENTITY_SLOT_COUNT, ENTITY_SLOT_PLAYER, EntitySlot, GameManager, GameStateFlag,
};
pub use garage::{CGarage, CGarageManager};
pub use tables::TableManager;

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
