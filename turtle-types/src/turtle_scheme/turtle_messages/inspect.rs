use crate::turtle_scheme::turtle_messages::{Command, Query};
use crate::turtle_scheme::{Block, Position};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "inspect")]
pub struct Inspect {}

impl Command for Inspect {
    type Response = Inspection;
}

impl Query for Inspect {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename = "inspection")]
pub struct Inspection {
    turtle_position: Position,
    above: Block,
    below: Block,
    front: Block,
}
