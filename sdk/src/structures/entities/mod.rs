//! Entity-related structures: base entity hierarchy, humanoids, vehicles, AI.

pub mod entity;
pub mod player;
pub mod car;
pub mod vehicle;
pub mod script_entity;
pub mod ai;
pub mod human_inventory;
pub mod human_components;
pub mod human_messages;
pub mod inventory_components;

pub use entity::{
    CActor, CEntity, CEntityDBRecord, CEntityGuid, CScriptWrapper, CScriptWrapperManager,
    CServiceIdentity, CTypeDescriptor, CWrapperFactory,
};
pub use player::{CHuman, CHumanNPC, CPlayer, CPlayerSub45C, CarWrapper};
pub use car::{CCar, CCarDamageSub1, CCarVehicle};
pub use vehicle::{Vehicle, VehicleWrapper};
pub use script_entity::{CScriptEntity, CScriptEntityChildEx};
pub use ai::{
    CAIController, CAITask, CHumanAIController, CHumanAIResources, CHumanAIState,
    CHumanStateVariables, CMafiaNavAgent,
};
pub use human_components::{
    CFrameColors, CHumanHeadController, CHumanWeaponController, CPlayerEmitter,
};
pub use human_inventory::CHumanInventory;
pub use human_messages::{human_message_id, DAMAGE_MSG_ID};
pub use inventory_components::{
    CInventory, CInventorySlot, CInventoryResource, CWeaponItem, CHumanThrowInventory,
};
