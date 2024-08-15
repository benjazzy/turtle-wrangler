use serde::{Deserialize, Serialize};

use crate::scheme::{Coordinates, Direction, Heading};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Command { command: TurtleCommand },
    Request { id: u64, request: Request },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestType {
    Inspect,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub id: u64,
    pub request: RequestType,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TurtleCommand {
    Request(Request),
    Move {
        direction: Direction,
    },
    Forward,
    Back,
    TurnLeft,
    TurnRight,
    Reboot,
    Inspect,
    UpdatePosition {
        coords: Coordinates,
        heading: Heading,
    },
}
