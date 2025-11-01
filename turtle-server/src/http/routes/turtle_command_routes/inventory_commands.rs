use crate::http::turtle_extractor::{TurtleExtractor, TurtleManagerState};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use tracing::{error, info};
use turtle_types::turtle_scheme;
use turtle_types::turtle_scheme::turtle_messages::{Inspect, Refuel, SelectSlot};
use turtle_types::turtle_scheme::{turtle_messages, InventoryItem};

pub async fn refuel(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<turtle_scheme::Fuel>, StatusCode> {
    info!("Got refuel request for {}", turtle.name());
    let result = turtle.lock().await.command(Refuel {}).await.map_err(|_| {
        error!("Problem refueling turtle {}", turtle.name());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result.unwrap()))
}

#[derive(Deserialize)]
pub struct SlotQuery {
    slot: u8,
}

pub async fn select_slot(
    Query(SlotQuery { slot }): Query<SlotQuery>,
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<InventoryItem>, StatusCode> {
    info!("Got refuel request for {}", turtle.name());
    let result = turtle
        .lock()
        .await
        .command(SelectSlot { slot })
        .await
        .map_err(|_| {
            error!("Problem refueling turtle {}", turtle.name());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(result.unwrap()))
}
