use crate::turtle_scheme::turtle_messages::{Command, Query};
use crate::turtle_scheme::{Fuel, inventory::InventoryItem, inventory::TurtleInventory};
use serde::Serialize;

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

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "refuel")]
pub struct Refuel {}

impl Command for Refuel {
    type Response = Result<Fuel, String>;
}
