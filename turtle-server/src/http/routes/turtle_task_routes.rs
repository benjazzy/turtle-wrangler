use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use futures::TryFutureExt;
use kameo::actor::ActorRef;
use tokio::task::JoinError;
use tracing::warn;
use turtle_types::turtle_scheme::{Coordinates, ToolSide};

use crate::{
    http::{
        turtle_extractor::{TurtleExtractor, TurtleManagerState},
        turtle_state::TurtleState,
    },
    turtles::{
        GetTracker, TaskMaster,
        task::TunnelTo,
        task_tracker::{RunningTasks, TaskList},
    },
};

async fn start_task(
    Query(coords): Query<Coordinates>,
    TurtleExtractor(turtle): TurtleExtractor,
    State(_): State<TurtleManagerState>,
) -> Result<&'static str, StatusCode> {
    let lock = turtle.lock().await;
    match lock.start_task(TunnelTo(coords, ToolSide::Right)).await {
        Ok(Ok(_)) => Ok("DONE"),
        Ok(Err(e)) => {
            warn!("{} had a problem while tunneling: {e}", e);

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            if e.is_panic() {
                tracing::error!("{} encounterd a panic while tunneling", turtle.name());

                Err(StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                Ok("CANCELED")
            }
        }
    }
}

#[axum::debug_handler]
async fn status(
    Path(turtle_name): Path<Arc<str>>,
    State(TurtleState { task_master, .. }): State<TurtleState>,
) -> Result<String, StatusCode> {
    let tracker = task_master
        .ask(GetTracker {
            turtle_name: turtle_name.clone(),
        })
        .await
        .map_err(|e| {
            tracing::error!("Problem getting tracker for {}: {e}", turtle_name);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let Some(tracker) = tracker else {
        return Ok("None".to_owned());
    };

    let tasks = tracker.ask(RunningTasks).await.map_err(|e| {
        tracing::error!("Problem getting running tasks for {}: {e}", turtle_name);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(tasks.to_string())
}

pub fn router() -> Router<TurtleState> {
    Router::new()
        .route("/turtle/{name}/task/tunnel", get(start_task))
        .route("/turtle/{name}/task/status", get(status))
}
