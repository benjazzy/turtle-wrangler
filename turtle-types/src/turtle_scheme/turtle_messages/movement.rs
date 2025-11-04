use crate::turtle_scheme::turtle_messages::{Command, TurtleResult};
use crate::turtle_scheme::{Position, turtle_messages::CommandError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum MovementError {
    #[serde(alias = "Out of fuel")]
    #[error("Out of fuel")]
    OutOfFuel,

    #[serde(alias = "MovementObstructed")]
    #[error("Movement obstructed")]
    MovementObstructed,
}

impl CommandError for MovementError {}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "forward")]
pub struct Forward {}

impl Command for Forward {
    type Response = TurtleResult<Position, MovementError>;
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
