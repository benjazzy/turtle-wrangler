use std::collections::HashMap;

use sea_orm::{DeriveActiveEnum, DeriveIden, EnumIter, prelude::StringLen, DeriveValueType, FromJsonQueryResult};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "block_type")]
#[serde(rename_all = "snake_case")]
pub enum OptionalBlock {
    Air,
    #[serde(alias = "normal")]
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub name: Box<str>,
    #[serde(default)]
    pub tags: HashMap<Box<str>, bool>,
    #[serde(default)]
    pub state: HashMap<Box<str>, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, FromJsonQueryResult)]
pub struct InventoryItem {
    name: Box<str>,
    count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromJsonQueryResult, Default, PartialEq, Eq)]
pub struct TurtleInventory {
    selected_slot: u8,
    items: [Option<InventoryItem>; 16],
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn check_deserialize_air() {
        let message = r#"{"name":"minecraft:air"}"#;

        let block: Block = serde_json::from_str(message).unwrap();
        assert_eq!(
            block,
            Block {
                name: "minecraft:air".into(),
                tags: Default::default(),
                state: Default::default(),
            }
        );
    }

    #[test]
    fn check_deserialize_block() {
        let message = r#"{
  "state": {
    "age": 7
  },
  "name": "minecraft:wheat",
  "tags": {
    "minecraft:bee_growables": true,
    "minecraft:crops": true,
    "minecraft:mineable/axe": true,
    "computercraft:turtle_hoe_harvestable": true,
    "minecraft:maintains_farmland": true,
    "minecraft:sword_efficient": true
  }
}"#;

        let block: Block = serde_json::from_str(message).unwrap();
        let tags = vec![
            "minecraft:bee_growables".into(),
            "minecraft:crops".into(),
            "minecraft:mineable/axe".into(),
            "computercraft:turtle_hoe_harvestable".into(),
            "minecraft:maintains_farmland".into(),
            "minecraft:sword_efficient".into(),
        ];
        let tags = tags.into_iter().fold(HashMap::new(), |mut acc, t| {
            acc.insert(t, true);

            acc
        });

        let mut state = HashMap::new();
        state.insert("age".into(), json!(7));

        assert_eq!(
            block,
            Block {
                name: "minecraft:wheat".into(),
                state,
                tags,
            }
        );
    }
}
