use serde::{Deserialize, Serialize};
use crate::turtle_scheme::turtle_messages::{Command, Query};

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(tag = "type", rename = "ping")]
pub struct Ping {
    pub id: u64,
}

impl Command for Ping {
    type Response = Pong;
}

impl Query for Ping {}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(tag = "type", rename = "pong")]
pub struct Pong {
    pub id: u64,
}