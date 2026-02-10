use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use futures::TryFutureExt;
use tokio::task::JoinError;
use tracing::warn;
use turtle_types::turtle_scheme::{Coordinates, ToolSide};

use crate::{
    http::{
        turtle_extractor::{TurtleExtractor, TurtleManagerState},
        turtle_state::TurtleState,
    },
    turtles::task::TunnelTo,
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

pub fn router() -> Router<TurtleState> {
    Router::new().route("/turtle/{name}/task/tunnel", get(start_task))
}
