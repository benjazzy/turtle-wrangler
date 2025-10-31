use axum::http::StatusCode;
use axum::extract::{FromRequestParts, State};
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::get;
use kameo::actor::ActorRef;
use kameo_actors::pubsub::PubSub;
use sea_orm::DatabaseConnection;
use tracing::{error, info};
use turtle_types::turtle_scheme;
use turtle_types::turtle_scheme::turtle_messages::Refuel;
use crate::http::routes::turtle_command_routes::turtle_extractor::{TurtleExtractor};
use crate::turtles::{TurtleManager, TurtleNotification};

mod turtle_extractor;
pub use turtle_extractor::TurtleManagerState;

#[axum::debug_handler]
async fn refuel(TurtleExtractor(turtle): TurtleExtractor, State(_): State<TurtleManagerState>) -> Result<Json<turtle_scheme::Fuel>, StatusCode> {
    info!("Got refuel request for {}", turtle.name());
    let result = turtle.lock().await.command(Refuel {}).await.map_err(|_| {
        error!("Problem refueling turtle {}", turtle.name());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result.unwrap()))
}
pub fn router(
) -> Router<TurtleManagerState> {
    Router::new().route("/{name}/refuel", get(refuel))
}
