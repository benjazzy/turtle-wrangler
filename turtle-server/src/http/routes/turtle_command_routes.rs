use crate::http::turtle_extractor::TurtleExtractor;
pub use crate::http::turtle_extractor::TurtleManagerState;
use crate::http::turtle_state::TurtleState;
use axum::extract::{FromRequestParts, Query, State};
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing::{error, info};
use turtle_types::turtle_scheme::turtle_messages;
use turtle_types::turtle_scheme::turtle_messages::{Command, Inspect, Ping};

mod dig_commands;
mod inventory_commands;
mod movement_commands;

pub async fn inspect(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<Json<turtle_messages::Inspection>, StatusCode> {
    let inspection = turtle.lock().await.command(Inspect {}).await.map_err(|_| {
        error!("Problem refueling turtle {}", turtle.name());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(inspection))
}

#[derive(Debug, Deserialize)]
struct PingQuery {
    id: u64,
}

pub async fn ping(
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
    Query(PingQuery { id }): Query<PingQuery>
) -> Result<String, StatusCode> {
    let pong = turtle.lock().await.command(Ping { id }).await.map_err(|_| {
        error!("Problem refueling turtle {}", turtle.name());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(pong.id.to_string())
}

pub fn router() -> Router<TurtleState> {
    Router::new()
        // .route("/{name}/dig", get(dig_commands::dig))
        // .route("/{name}/digUp", get(dig_commands::dig_up))
        // .route("/{name}/digDown", get(dig_commands::dig_down))
        // .route("/{name}/inspect", get(inspect))
        // .route("/{name}/refuel", get(inventory_commands::refuel))
        // .route("/{name}/select_slot", put(inventory_commands::select_slot))
        .route("/turtle/{name}/forward", get(movement_commands::forward))
        .route("/turtle/{name}/backward", get(movement_commands::backward))
        .route("/turtle/{name}/turnLeft", get(movement_commands::turn_left))
        .route("/turtle/{name}/turnRight", get(movement_commands::turn_right))
        .route("/turtle/{name}/up", get(movement_commands::up))
        .route("/turtle/{name}/down", get(movement_commands::down))
}
