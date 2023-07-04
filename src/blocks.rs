use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Air,

    #[serde(rename = "computercraft:turtle_normal")]
    TurtleNormal {
        #[serde(flatten)]
        data: BlockData<TurtleState>,
    },

    #[serde(rename = "computercraft:turtle_advanced")]
    TurtleAdvanced {
        #[serde(flatten)]
        data: BlockData<TurtleState>,
    },

    Other {
        #[serde(flatten)]
        data: BlockData<HashMap<String, String>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurtleState {
    pub facing: String,
    pub waterlogged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockData<S> {
    pub name: String,
    pub state: S,
    pub tags: HashMap<String, bool>,
}
