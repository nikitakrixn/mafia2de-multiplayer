//! VTable layouts for entity hierarchy: humanoids, vehicles, AI, inventory.

pub mod player;
pub mod car;
pub mod ai;
pub mod inventory;

pub use player::CHumanVTable;
pub use car::CCarVTable;
pub use ai::{
    CAIControllerVTable, CAITaskVTable, CHumanAIControllerVTable, CHumanAIResourcesVTable,
    CHumanAIStateVTable, CMafiaNavAgentVTable,
};
pub use inventory::{
    CHumanInventoryVTable, CHumanThrowInventoryVTable, CInventoryResourceVTable,
    CInventorySlotVTable, CInventoryVTable, CWeaponItemVTable,
};
