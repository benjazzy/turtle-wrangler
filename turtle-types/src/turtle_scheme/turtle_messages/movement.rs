use serde::Serialize;
use crate::turtle_scheme::Position;
use crate::turtle_scheme::turtle_messages::Command;

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