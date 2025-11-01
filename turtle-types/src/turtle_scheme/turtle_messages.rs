mod dig;
mod inspect;
mod inventory;
mod movement;
mod ping;
mod reboot;

use crate::turtle_scheme::{Coordinates, Fuel, Heading, TurtleInventory};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use dig::*;
pub use inspect::*;
pub use inventory::*;
pub use movement::*;
pub use ping::*;
pub use reboot::*;

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

pub trait Command: Serialize {
    type Response: DeserializeOwned + Send + 'static;
}

pub trait Query: Command {}
