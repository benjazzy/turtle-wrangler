mod dig;
mod inspect;
mod inventory;
mod movement;
mod ping;
mod reboot;

use crate::turtle_scheme::{Coordinates, Fuel, Heading, inventory::TurtleInventory};
use serde::de::{self, DeserializeOwned};
use serde::{Deserialize, Serialize};

pub use dig::*;
pub use inspect::*;
pub use inventory::*;
pub use movement::*;
pub use ping::*;
pub use reboot::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurtleResult<O, E> {
    Ok(O),
    Err(E),
}

impl<'de, O, E> Deserialize<'de> for TurtleResult<O, E>
where
    O: Deserialize<'de>,
    E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = serde_json::Map::deserialize(deserializer)?;
        let value = map
            .remove("success")
            .ok_or_else(|| de::Error::missing_field("success"))?;

        let success: bool = serde_json::from_value(value).map_err(de::Error::custom)?;
        let rest = map
            .remove("message")
            .ok_or_else(|| de::Error::missing_field("success"))?;
        // let rest = Value::Object(map);

        if success {
            O::deserialize(rest)
                .map(TurtleResult::Ok)
                .map_err(de::Error::custom)
        } else {
            E::deserialize(rest)
                .map(TurtleResult::Err)
                .map_err(de::Error::custom)
        }
    }
}

impl<O, E> From<TurtleResult<O, E>> for std::result::Result<O, E> {
    fn from(value: TurtleResult<O, E>) -> Self {
        match value {
            TurtleResult::Ok(o) => Ok(o),
            TurtleResult::Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "info_type")]
#[serde(rename_all = "lowercase")]
pub enum TurtleInformation {
    Report {
        fuel: Fuel,
        heading: Heading,
        position: Coordinates,
        inventory: TurtleInventory,
    },
}

pub trait CommandError: std::error::Error {}

pub trait Command: Serialize {
    type Response: DeserializeOwned + Send + 'static;
}

pub trait Query: Command {}
