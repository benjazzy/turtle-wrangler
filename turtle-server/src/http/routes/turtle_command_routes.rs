use crate::http::turtle_extractor::TurtleExtractor;
use crate::turtles::{TurtleManager, TurtleNotification};
use axum::extract::{FromRequestParts, Query, State};
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use kameo::actor::ActorRef;
use kameo_actors::pubsub::PubSub;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::future::Future;
use tracing::{error, info};
use turtle_types::turtle_scheme;
use turtle_types::turtle_scheme::turtle_messages::{Command, DigDown, Inspect, Refuel, SelectSlot};
use turtle_types::turtle_scheme::InventoryItem;

pub use crate::http::turtle_extractor::TurtleManagerState;

fn send_command<C: Command>(
    command: C,
) -> impl AsyncFn(TurtleExtractor, State<TurtleManagerState>) {
    async move |TurtleExtractor(turtle), _| {
        let result = turtle.lock().await.command(Refuel {}).await.map_err(|_| {
            error!("Problem refueling turtle {}", turtle.name());
            StatusCode::INTERNAL_SERVER_ERROR
        });

        todo!()
    }
}

async fn refuel(
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
struct SlotQuery {
    slot: u8,
}

async fn select_slot(
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

pub fn router() -> Router<TurtleManagerState> {
    Router::new()
        .route("/{name}/inspect", get(send_command(Inspect {})))
        .route("/{name}/refuel", get(refuel))
        .route("{name}/select_slot", put(select_slot))
}
