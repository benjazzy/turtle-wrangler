use crate::http::turtle_extractor::TurtleExtractor;
pub use crate::http::turtle_extractor::TurtleManagerState;
use crate::http::turtle_state::TurtleState;
use crate::turtles::{LockedTurtle, TaskyTurtle, Turtle, TurtleRequestError};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use kameo::actor::ActorRef;
use serde::Deserialize;
use thiserror::Error;
use tracing::error;
use turtle_types::turtle_scheme::turtle_messages::{self, CommandError, TurnLeft};
use turtle_types::turtle_scheme::turtle_messages::{Inspect, Ping, Reboot};

mod dig_commands;
mod inventory_commands;
mod movement_commands;

#[derive(Debug, Error)]
enum TurtleCommandError<Inner: CommandError> {
    #[error("Problem sending requet to turtle {0}")]
    ReqeustError(#[from] TurtleRequestError),

    #[error("Turtle returned an error running the command: {0}")]
    CommandError(#[from] Inner),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
enum Infallible {}

impl CommandError for Infallible {}

impl<Inner: CommandError> IntoResponse for TurtleCommandError<Inner> {
    fn into_response(self) -> axum::response::Response {
        match self {
            TurtleCommandError::ReqeustError(turtle_request_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                turtle_request_error.to_string(),
            )
                .into_response(),
            TurtleCommandError::CommandError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}

pub async fn inspect(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<turtle_messages::Inspection>, StatusCode> {
    let inspection = turtle.lock().await.command(Inspect {}).await.map_err(|_| {
        error!("Problem getting inspection from trutle {}", turtle.name());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(inspection))
}

#[derive(Debug, Deserialize)]
struct PingQuery {
    id: u64,
}

async fn ping(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(PingQuery { id }): Query<PingQuery>,
) -> Result<String, StatusCode> {
    let pong = turtle
        .lock()
        .await
        .command(Ping { id })
        .await
        .map_err(|_| {
            error!("Problem pinging turtle {}", turtle.name());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(pong.id.to_string())
}

async fn reboot(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<&'static str, StatusCode> {
    turtle
        .lock()
        .await
        .command(Reboot { id: 0 })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok("OK")
}

#[axum::debug_handler]
async fn start_task(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<&'static str, StatusCode> {
    let locked_turtle = turtle.lock().await;
    locked_turtle
        .start_task(async |turtle: &TaskyTurtle| {
            for _ in 0..100 {
                turtle.command(TurnLeft {}).await;
            }
        })
        .await;

    Ok("DONE")
}

pub fn router() -> Router<TurtleState> {
    Router::new()
        .route("/turtle/{name}/inspect", get(inspect))
        .route("/turtle/{name}/ping", get(ping))
        .route("/turtle/{name}/reboot", get(reboot))
        .route("/turtle/{name}/dig", get(dig_commands::dig))
        .route("/turtle/{name}/digUp", get(dig_commands::dig_up))
        .route("/turtle/{name}/digDown", get(dig_commands::dig_down))
        .route("/turtle/{name}/refuel", get(inventory_commands::refuel))
        .route(
            "/turtle/{name}/select_slot",
            put(inventory_commands::select_slot),
        )
        .route("/turtle/{name}/forward", get(movement_commands::forward))
        .route("/turtle/{name}/backward", get(movement_commands::backward))
        .route("/turtle/{name}/turnLeft", get(movement_commands::turn_left))
        .route(
            "/turtle/{name}/turnRight",
            get(movement_commands::turn_right),
        )
        .route("/turtle/{name}/up", get(movement_commands::up))
        .route("/turtle/{name}/down", get(movement_commands::down))
        .route("/turtle/{name}/task", get(start_task))
}
