use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Different types of Minecraft blocks we recognize.
/// Other is used for any block we don't recognize.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    /// Indicates that there is no block only air.
    Air,

    /// Another normal turtle.
    #[serde(rename = "computercraft:turtle_normal")]
    TurtleNormal {
        #[serde(flatten)]
        data: BlockData<TurtleBlockState>,
    },

    /// Another advanced turtle.
    #[serde(rename = "computercraft:turtle_advanced")]
    TurtleAdvanced {
        #[serde(flatten)]
        data: BlockData<TurtleBlockState>,
    },

    /// Any block that is not in this list.
    Other {
        #[serde(flatten)]
        data: BlockData<HashMap<String, String>>,
    },
}

/// Data that a block can contain.
/// Type S is the type of the state data.
/// For Block::Other S is a HashMap<String, String>.
/// Otherwise each block can have its own state type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockData<S> {
    /// The Minecraft identifier of the block.
    /// I.E. "minecraft:andesite"
    pub name: String,

    /// Minecraft state data of the block.
    pub state: S,

    /// Minecraft tags of the block.
    pub tags: HashMap<String, bool>,
}

/// The Minecraft state that a turtle can have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurtleBlockState {
    /// Direction the turtle is facing.
    pub facing: String,

    /// Whether the turtle is waterlogged or not.
    pub waterlogged: bool,
}
