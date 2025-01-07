mod turtle_commands;
mod turtle_events;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
// pub use turtle_commands::{Message, Request, RequestType, TurtleCommand};
// pub use turtle_events::{Response, ResponseType, TurtleEvents};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "notification_type")]
pub enum TurtleInformation {
    Fuel { fuel: u64 },
}

pub trait Command {
    type Response: DeserializeOwned;
}

pub trait Query: Command {}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct Ping {
    id: u64,
}

impl Command for Ping {
    type Response = Pong;
}

impl Query for Ping {}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct Pong {
    id: u64,
}
