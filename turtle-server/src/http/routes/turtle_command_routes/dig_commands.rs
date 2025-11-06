use crate::http::routes::turtle_command_routes::TurtleCommandError;
use crate::http::turtle_extractor::{TurtleExtractor, TurtleManagerState};
use crate::utils::FlattenExt;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use turtle_types::turtle_scheme::turtle_messages::{Dig, DigDown, DigError, DigUp};
use turtle_types::turtle_scheme::ToolSide;

pub async fn dig(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, TurtleCommandError<DigError>> {
    info!("Got dig request for {}", turtle.name());
    let success = turtle
        .lock()
        .await
        .command(Dig { side })
        .await
        .map(Into::into)
        .nested_flatten();

    success.map(Json)
}

pub async fn dig_up(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, TurtleCommandError<DigError>> {
    info!("Got dig up request for {}", turtle.name());
    let success = turtle
        .lock()
        .await
        .command(DigUp { side })
        .await
        .map(Into::into)
        .nested_flatten();

    success.map(Json)
}

pub async fn dig_down(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, TurtleCommandError<DigError>> {
    info!("Got dig down request for {}", turtle.name());
    let success = turtle
        .lock()
        .await
        .command(DigDown { side })
        .await
        .map(Into::into)
        .nested_flatten();

    success.map(Json)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DigSide {
    side: ToolSide,
}
