use sea_orm::{DeriveActiveEnum, DeriveIden, EnumIter, prelude::StringLen};
use serde::{Deserialize, Serialize};

pub mod turtle_messages;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    DeriveIden,
    EnumIter,
    DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum Heading {
    #[serde(rename = "n")]
    #[sea_orm(string_value = "n")]
    North,

    #[serde(rename = "s")]
    #[sea_orm(string_value = "s")]
    South,

    #[serde(rename = "e")]
    #[sea_orm(string_value = "e")]
    East,

    #[serde(rename = "w")]
    #[sea_orm(string_value = "2")]
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
pub struct Position {
    #[serde(flatten)]
    pub coordinates: Coordinates,
    pub heading: Heading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fuel {
    pub level: u32,
    pub max: u32,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    DeriveIden,
    EnumIter,
    DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum TurtleType {
    #[sea_orm(string_value = "normal")]
    Normal,
    #[sea_orm(string_value = "advanced")]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TurtleStatus {
    Connected,
    Disconnected,
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

pub struct BlockTag {
    pub name: Box<str>,
    pub state: bool,
}

impl<T> From<T> for BlockTag
where
    T: Into<Box<str>>,
{
    fn from(value: T) -> Self {
        BlockTag {
            name: value.into(),
            state: true,
        }
    }
}

pub struct Block {
    pub name: Box<str>,
    pub tags: Vec<BlockTag>,
}
