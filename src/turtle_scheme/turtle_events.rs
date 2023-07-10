use serde::{Deserialize, Serialize};

use crate::scheme::{Fuel, Heading, Position};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TurtleEvents {
    Report {
        position: Position,
        heading: Heading,
        fuel: Fuel,
    },
    Inspection {
        block: crate::blocks::Block,
    },
    Ok {
        id: u64,
    },
    Ready,
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
