use serde::Serialize;
use crate::turtle_scheme::ToolSide;
use crate::turtle_scheme::turtle_messages::Command;

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