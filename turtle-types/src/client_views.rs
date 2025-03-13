use serde::{Deserialize, Serialize};
use crate::turtle_scheme::{Coordinates, Fuel, Heading, TurtleStatus, TurtleType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurtleReport {
    pub name: Box<str>,
    pub status: TurtleStatus,
    pub coordinates: Coordinates,
    pub heading: Heading,
    pub turtle_type: TurtleType,
    pub fuel: Fuel,
}