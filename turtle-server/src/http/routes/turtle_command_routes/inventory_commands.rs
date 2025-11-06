use crate::http::routes::turtle_command_routes::{Infallible, TurtleCommandError};
use crate::http::turtle_extractor::{TurtleExtractor, TurtleManagerState};
use crate::utils::FlattenExt;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use tracing::{error, info};
use turtle_types::turtle_scheme;
use turtle_types::turtle_scheme::turtle_messages::{Refuel, RefuelError, SelectSlot};
use turtle_types::turtle_scheme::InventoryItem;

pub async fn refuel(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<turtle_scheme::Fuel>, TurtleCommandError<RefuelError>> {
    info!("Got refuel request for {}", turtle.name());
    let result = turtle
        .lock()
        .await
        .command(Refuel {})
        .await
        .map(Into::into)
        .nested_flatten();

    result.map(Json)
}

#[derive(Deserialize)]
pub struct SlotQuery {
    slot: u8,
}

pub async fn select_slot(
    Query(SlotQuery { slot }): Query<SlotQuery>,
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Option<InventoryItem>>, TurtleCommandError<Infallible>> {
    info!("Got refuel request for {}", turtle.name());

    turtle
        .lock()
        .await
        .command(SelectSlot { slot })
        .await
        .map_err(Into::into)
        .map(Json)
}
