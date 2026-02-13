mod turtle_command_routes;
mod turtle_connect_routes;
mod turtle_query_routes;
mod turtle_task_routes;

use crate::http::routes::turtle_command_routes::TurtleManagerState;
use crate::turtles::{
    GetConnectedTurtles, GetTurtle, TaskMaster, Turtle, TurtleManager, TurtleNotification,
    identify_turtle,
};
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, Query, Request, State, WebSocketUpgrade};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router, body};
use kameo::actor::ActorRef;
use kameo_actors::pubsub::PubSub;
use sea_orm::DatabaseConnection;
use tracing::{error, info};
use turtle_types::client_views;

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
    task_master: ActorRef<TaskMaster>,
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
        .merge(turtle_task_routes::router())
        .with_state(super::turtle_state::TurtleState {
            turtle_manager: manager,
            database: db,
            task_master,
            pub_sub,
        })
}
