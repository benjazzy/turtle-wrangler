use serde::{Deserialize, Serialize};
use crate::scheme;
use crate::scheme::Direction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    GetTurtles,
    Move { name: String, direction: Direction },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Turtles{ turtles: Vec<scheme::Turtle> },
}