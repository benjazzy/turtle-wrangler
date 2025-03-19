use crate::turtle_scheme::{Coordinates, Fuel, Heading, TurtleStatus, TurtleType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurtleReport {
    pub name: Box<str>,
    pub status: TurtleStatus,
    pub coordinates: Coordinates,
    pub heading: Heading,
    pub turtle_type: TurtleType,
    pub fuel: Fuel,
}
