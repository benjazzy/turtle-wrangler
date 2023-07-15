use serde::{Deserialize, Serialize};

use crate::{
    blocks::Block,
    scheme::{Fuel, Heading, Position},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TurtleEvents {
    Response {
        response: Response,
    },
    Report {
        position: Position,
        heading: Heading,
        fuel: Fuel,
    },
    GetPosition,
    Inspection {
        block: crate::blocks::Block,
    },
    Ok {
        id: u64,
    },
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub response: ResponseType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseType {
    Inspection { block: Block },
    Pong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}
