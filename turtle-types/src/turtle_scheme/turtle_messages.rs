use crate::turtle_scheme::{Coordinates, Fuel, Heading, InventoryItem, TurtleInventory};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::{Block, Position, ToolSide};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "info_type")]
#[serde(rename_all = "lowercase")]
pub enum TurtleInformation {
    Report {
        fuel: Fuel,
        heading: Heading,
        position: Coordinates,
        inventory: TurtleInventory,
    },
}

pub trait Command: Serialize {
    type Response: DeserializeOwned + Send + 'static;
}

pub trait Query: Command {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "ping")]
pub struct Ping {
    pub id: u64,
}

impl Command for Ping {
    type Response = Pong;
}

impl Query for Ping {}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(tag = "type", rename = "pong")]
pub struct Pong {
    pub id: u64,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "reboot")]
pub struct Reboot {
    pub id: u64,
}

impl Command for Reboot {
    type Response = u64;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "inspect")]
pub struct Inspect {}

impl Command for Inspect {
    type Response = Inspection;
}

impl Query for Inspect {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "get_inventory")]
pub struct GetInventory {}

impl Command for GetInventory {
    type Response = TurtleInventory;
}

impl Query for GetInventory {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "select_slot")]
pub struct SelectSlot{
    pub slot: u8
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename = "inspection")]
pub struct Inspection {
    turtle_position: Position,
    above: Block,
    below: Block,
    front: Block,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "forward")]
pub struct Forward {}

impl Command for Forward {
    type Response = Position;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "backward")]
pub struct Backward {}

impl Command for Backward {
    type Response = Position;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "turn_left")]
pub struct TurnLeft {}

impl Command for TurnLeft {
    type Response = Position;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "turn_right")]
pub struct TurnRight {}

impl Command for TurnRight {
    type Response = Position;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "up")]
pub struct Up {}

impl Command for Up {
    type Response = Position;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "down")]
pub struct Down {}

impl Command for Down {
    type Response = Position;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "dig")]
pub struct Dig {
    pub side: ToolSide,
}

impl Command for Dig {
    type Response = bool;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "dig_up")]
pub struct DigUp {
    pub side: ToolSide,
}

impl Command for DigUp {
    type Response = bool;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "dig_down")]
pub struct DigDown {
    pub side: ToolSide,
}

impl Command for DigDown {
    type Response = bool;
}
