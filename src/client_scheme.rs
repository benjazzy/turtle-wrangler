use serde::{Deserialize, Serialize};
use crate::scheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    GetTurtles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Reply {
    Turtles{ turtles: Vec<scheme::Turtle> },
}