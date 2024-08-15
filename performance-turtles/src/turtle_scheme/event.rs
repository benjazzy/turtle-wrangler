use serde::{Deserialize, Serialize};

use crate::{
    blocks::Block,
    scheme::{Coordinates, Fuel, Heading},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Report {
        position: Coordinates,
        heading: Heading,
        fuel: Fuel,
    },
    GetPosition,
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
