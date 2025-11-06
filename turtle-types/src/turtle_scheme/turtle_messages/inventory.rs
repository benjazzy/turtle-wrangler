use crate::turtle_scheme::turtle_messages::{Command, CommandError, Query, TurtleResult};
use crate::turtle_scheme::{Fuel, inventory::InventoryItem, inventory::TurtleInventory};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "get_inventory")]
pub struct GetInventory {}

impl Command for GetInventory {
    type Response = TurtleInventory;
}

impl Query for GetInventory {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "select_slot")]
pub struct SelectSlot {
    pub slot: u8,
}

impl Command for SelectSlot {
    type Response = Option<InventoryItem>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum RefuelError {
    #[error("Item not combustible")]
    #[serde(alias = "Items not combustible")]
    NotCombustible,

    #[error("No items to combust")]
    #[serde(alias = "No items to combust")]
    NoItems,
}

impl CommandError for RefuelError {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "refuel")]
pub struct Refuel {}

impl Command for Refuel {
    type Response = TurtleResult<Fuel, RefuelError>;
}
