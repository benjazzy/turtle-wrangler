use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heading {
    #[serde(alias = "n")]
    North,

    #[serde(alias = "s")]
    South,

    #[serde(alias = "e")]
    East,

    #[serde(alias = "w")]
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fuel {
    pub level: u64,
    pub max: u64,
}

impl std::fmt::Display for Position {
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
