use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "turtle_inventory")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub second: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub third: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub fourth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub fifth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub sixth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub seventh: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub eighth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub ninth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub tenth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub eleventh: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub twelfth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub thirteenth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub fourteenth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub fifteenth: Option<turtle_types::turtle_scheme::InventoryItem>,
    pub sixteenth: Option<turtle_types::turtle_scheme::InventoryItem>,
}