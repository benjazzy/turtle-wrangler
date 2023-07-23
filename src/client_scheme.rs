use crate::scheme;
use crate::scheme::Direction;
use crate::turtle_scheme::TurtleEvents;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    GetTurtles,
    Move { name: String, direction: Direction },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Turtles { turtles: Vec<scheme::Turtle> },
    TurtleEvent { name: String, event: TurtleEvents },
    TurtleConnected { name: String },
    TurtleDisconnected { name: String },
}
