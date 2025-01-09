use crate::scheme::{Coordinates, Fuel, Heading};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "info_type")]
#[serde(rename_all = "lowercase")]
pub enum TurtleInformation {
    Report {
        fuel: Fuel,
        heading: Heading,
        position: Coordinates,
    },
}

pub trait Command: Serialize {
    type Response: DeserializeOwned + Send + 'static;
}

pub trait Query: Command {}

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
