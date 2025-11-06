use crate::turtle_scheme::turtle_messages::{Command, CommandError};
use crate::turtle_scheme::{ToolSide, turtle_messages::TurtleResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Error)]
pub enum DigError {
    #[serde(alias = "No tool to dig with")]
    #[error("No tool to dig with")]
    NoTool,

    #[serde(alias = "Nothing to dig here")]
    #[error("Nothing to dig here")]
    NothingToDig,

    #[serde(alias = "Cannot break unbreakable block")]
    #[error("Cannon break unbreakable block")]
    UnbreakableBlock,
}

impl CommandError for DigError {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "dig")]
pub struct Dig {
    pub side: ToolSide,
}

impl Command for Dig {
    type Response = TurtleResult<bool, DigError>;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "dig_up")]
pub struct DigUp {
    pub side: ToolSide,
}

impl Command for DigUp {
    type Response = TurtleResult<bool, DigError>;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "dig_down")]
pub struct DigDown {
    pub side: ToolSide,
}

impl Command for DigDown {
    type Response = TurtleResult<bool, DigError>;
}
