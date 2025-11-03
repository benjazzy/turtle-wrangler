mod turtle_command_routes;
mod turtle_connect_routes;
mod turtle_query_routes;

use crate::http::routes::turtle_command_routes::TurtleManagerState;
use crate::turtles::{
    identify_turtle, GetConnectedTurtles, GetTurtle, Turtle, TurtleManager, TurtleNotification,
};
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, Query, Request, State, WebSocketUpgrade};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{body, Json, Router};
use axum_extra::{headers, TypedHeader};
use futures::StreamExt;
use kameo::actor::ActorRef;
use kameo_actors::pubsub::PubSub;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};
use turtle_types::turtle_scheme::turtle_messages::*;
use turtle_types::turtle_scheme::{turtle_messages, ToolSide};
use turtle_types::{client_views, turtle_scheme};

#[axum::debug_handler]
async fn get_turtles(
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<Vec<client_views::TurtleReport>>, StatusCode> {
    let turtles = manager.ask(GetConnectedTurtles).await.map_err(|e| {
        error!("Problem getting turtles from turtle manager {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(turtles))
}


pub fn router(
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    manager: ActorRef<TurtleManager>,
    db: DatabaseConnection,
) -> Router {
    let scripts_dir = std::env::var("SCRIPTS_DIR").unwrap_or("../scripts".to_string());
    info!("Serving scripts from {scripts_dir}");
    Router::new()
        .route("/turtles", get(get_turtles))
        .with_state(manager.clone())
        .merge(turtle_connect_routes::router())
        .merge(turtle_query_routes::router())
        .merge(turtle_command_routes::router())
        .with_state(super::turtle_state::TurtleState {
            turtle_manager: manager,
            database: db,
            pub_sub,
        })
}
