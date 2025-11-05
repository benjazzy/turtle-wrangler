use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

mod helper {
    // Serde has trouble with the turtle sending an empty inventory as a map "{}" or with fewer
    // than 16 items. This helper types are able to deserialize them while keeping the main
    // TurtleInventory type relatively clean.
    use serde::Deserialize;

    use crate::turtle_scheme::{InventoryItem, TurtleInventory};

    #[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
    pub enum ItemsHelper {
        Empty {},
        Items(Vec<Option<InventoryItem>>),
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct InventoryHelper {
        pub selected_slot: u8,
        pub items: ItemsHelper,
    }

    impl TryFrom<InventoryHelper> for TurtleInventory {
        type Error = &'static str;

        fn try_from(
            InventoryHelper {
                selected_slot,
                items,
            }: InventoryHelper,
        ) -> Result<Self, Self::Error> {
            let items = match items {
                ItemsHelper::Empty {} => Default::default(),
                ItemsHelper::Items(items) => items.into_iter().enumerate().try_fold(
                    Default::default(),
                    |mut acc: [Option<InventoryItem>; 16], (i, item)| {
                        let Some(slot) = acc.get_mut(i) else {
                            return Err("Inventory list too long");
                        };

                        *slot = item;

                        Ok(acc)
                    },
                )?,
            };

            let inv = TurtleInventory {
                selected_slot,
                items,
            };

            Ok(inv)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, FromJsonQueryResult)]
pub struct InventoryItem {
    pub name: Box<str>,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromJsonQueryResult, Default, PartialEq, Eq)]
#[serde(try_from = "helper::InventoryHelper")]
pub struct TurtleInventory {
    pub selected_slot: u8,
    // #[serde(deserialize_with = "de::deserialize_inventory")]
    pub items: [Option<InventoryItem>; 16],
}
