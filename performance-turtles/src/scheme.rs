use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "f")]
    Forward,

    #[serde(rename = "b")]
    Back,

    #[serde(rename = "l")]
    Left,

    #[serde(rename = "r")]
    Right,

    #[serde(rename = "u")]
    Up,

    #[serde(rename = "d")]
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Coordinates {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heading {
    #[serde(rename = "n")]
    North,

    #[serde(rename = "s")]
    South,

    #[serde(rename = "e")]
    East,

    #[serde(rename = "w")]
    West,
}

impl Heading {
    const NORTH: &'static str = "n";
    const SOUTH: &'static str = "s";
    const EAST: &'static str = "e";
    const WEST: &'static str = "w";

    pub fn as_str(&self) -> &'static str {
        match self {
            Heading::North => Self::NORTH,
            Heading::South => Self::SOUTH,
            Heading::East => Self::EAST,
            Heading::West => Self::WEST,
        }
    }

    pub fn from_str(s: &str) -> Option<Heading> {
        match s {
            Self::NORTH => Some(Heading::North),
            Self::SOUTH => Some(Heading::South),
            Self::EAST => Some(Heading::East),
            Self::WEST => Some(Heading::West),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fuel {
    pub level: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurtleType {
    Normal,
    Advanced,
}

impl TurtleType {
    const NORMAL_NAME: &'static str = "normal";
    const ADVANCED: &'static str = "advanced";
    pub const NORMAL_FUEL: u32 = 20000;
    pub const ADVANCED_FUEL: u32 = 100000;

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            Self::NORMAL_NAME => Some(TurtleType::Normal),
            Self::ADVANCED => Some(TurtleType::Advanced),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TurtleType::Normal => Self::NORMAL_NAME,
            TurtleType::Advanced => Self::ADVANCED,
        }
    }

    pub fn get_max_fuel(&self) -> u32 {
        match self {
            TurtleType::Normal => Self::NORMAL_FUEL,
            TurtleType::Advanced => Self::ADVANCED_FUEL,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Turtle {
    pub name: String,
    pub coordinates: Coordinates,
    pub heading: Heading,
    pub turtle_type: TurtleType,
    pub fuel: Fuel,
}

// pub struct TurtleData {
//     pub name: String,
//     pub turtle_type: TurtleType,
// }

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}, z: {}", self.x, self.y, self.z)
    }
}

impl std::fmt::Display for Heading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let h = match self {
            Heading::North => "North",
            Heading::South => "South",
            Heading::East => "East",
            Heading::West => "West",
        };

        write!(f, "{h}")
    }
}

impl std::fmt::Display for Fuel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.level, self.max)
    }
}
