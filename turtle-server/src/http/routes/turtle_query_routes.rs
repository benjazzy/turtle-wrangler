use crate::http::turtle_entity_extractor::TurtleEntityExtractor;
use crate::http::turtle_state::TurtleState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use turtle_types::turtle_scheme::TurtleInventory;

async fn get_inventory(
    TurtleEntityExtractor(turtle): TurtleEntityExtractor,
    State(_): State<TurtleState>,
) -> Result<Json<TurtleInventory>, StatusCode> {
    Ok(Json(turtle.inventory))
}

pub fn router() -> Router<TurtleState> {
    Router::new().route("/turtle/{name}/inventory", get(get_inventory))
}
