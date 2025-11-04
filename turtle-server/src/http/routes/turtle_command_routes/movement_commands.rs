use crate::http::routes::turtle_command_routes::TurtleCommandError;
use crate::http::turtle_extractor::{TurtleExtractor, TurtleManagerState};
use crate::utils::FlattenExt;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tracing::debug;
use turtle_types::turtle_scheme::turtle_messages::{
    Backward, Down, Forward, MovementError, TurnLeft, TurnRight, Up,
};
use turtle_types::turtle_scheme::Position;

#[axum::debug_handler]
pub async fn forward(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Position>, TurtleCommandError<MovementError>> {
    let result = turtle
        .lock()
        .await
        .command(Forward {})
        .await
        .map(Into::into)
        .nested_flatten();

    result.map(Json)
}

pub async fn backward(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Position>, StatusCode> {
    turtle
        .lock()
        .await
        .command(Backward {})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn turn_left(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Position>, StatusCode> {
    turtle
        .lock()
        .await
        .command(TurnLeft {})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn turn_right(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Position>, StatusCode> {
    turtle
        .lock()
        .await
        .command(TurnRight {})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn up(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Position>, StatusCode> {
    turtle
        .lock()
        .await
        .command(Up {})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn down(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<Position>, StatusCode> {
    turtle
        .lock()
        .await
        .command(Down {})
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}
