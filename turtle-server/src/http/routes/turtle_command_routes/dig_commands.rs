use crate::http::turtle_extractor::{TurtleExtractor, TurtleManagerState};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use turtle_types::turtle_scheme::turtle_messages::{Dig, DigDown, DigUp};
use turtle_types::turtle_scheme::ToolSide;

pub async fn dig(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, StatusCode> {
    info!("Got refuel request for {}", turtle.name());
    let success = turtle
        .lock()
        .await
        .command(Dig { side })
        .await
        .map_err(|_| {
            error!("Problem refueling turtle {}", turtle.name());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(success))
}

pub async fn dig_up(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, StatusCode> {
    info!("Got refuel request for {}", turtle.name());
    let success = turtle
        .lock()
        .await
        .command(DigUp { side })
        .await
        .map_err(|_| {
            error!("Problem refueling turtle {}", turtle.name());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(success))
}

pub async fn dig_down(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, StatusCode> {
    info!("Got refuel request for {}", turtle.name());
    let success = turtle
        .lock()
        .await
        .command(DigDown { side })
        .await
        .map_err(|_| {
            error!("Problem refueling turtle {}", turtle.name());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(success))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DigSide {
    side: ToolSide,
}
