use crate::turtles::{
    identify_turtle, GetConnectedTurtles, GetTurtle, TurtleManager, TurtleNotification,
};
use axum::extract::{ws, ConnectInfo, Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::{headers, TypedHeader};
use kameo::actor::pubsub::PubSub;
use kameo::actor::ActorRef;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};
use turtle_types::turtle_scheme::turtle_messages;
use turtle_types::{client_views, turtle_scheme};

#[derive(Debug, Clone)]
struct TurtleState {
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    db: DatabaseConnection,
}

#[axum::debug_handler]
async fn get_startup_script() -> impl IntoResponse {
    const STARTUP_SCRIPT: &str = r#"
local ok = shell.execute("wget", "run", "$ip$")
os.sleep(5)
os.reboot()
"#;

    STARTUP_SCRIPT.replace("$ip$", "http://127.0.0.1:8080/scripts/run.lua")
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(TurtleState { pub_sub, db }): State<TurtleState>,
) -> impl IntoResponse {
    let user_agent = if let Some(user_agent) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    debug!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |ws| async move {
        identify_turtle(ws, pub_sub, db).await;
    })
}

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

async fn ping(
    Path(turtle_name): Path<String>,
    Query(query): Query<HashMap<String, u64>>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<String, StatusCode> {
    println!("NAME: {turtle_name}");
    let id = *query.get("id").unwrap();
    let turtle = manager
        .ask(GetTurtle {
            name: turtle_name.into(),
        })
        .await
        .unwrap()
        .unwrap();

    let turtle_messages::Pong { id } = turtle.query(turtle_messages::Ping { id }).await.unwrap();

    Ok(id.to_string())
}

async fn reboot(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<&'static str, StatusCode> {
    let turtle = manager
        .ask(GetTurtle {
            name: turtle_name.into(),
        })
        .await
        .unwrap()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    turtle
        .lock()
        .await
        .command(turtle_messages::Reboot { id: 0 })
        .await;

    Ok("OK")
}

pub fn router(
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    manager: ActorRef<TurtleManager>,
    db: DatabaseConnection,
) -> Router {
    let scripts_dir = std::env::var("SCRIPTS_DIR").unwrap_or("../scripts".to_string());
    info!("Serving scripts from {scripts_dir}");
    Router::new()
        .nest_service("/scripts", ServeDir::new(scripts_dir))
        .route("/startup.lua", get(get_startup_script))
        .route("/ws", get(ws_handler))
        .with_state(TurtleState { pub_sub, db })
        .route("/turtles", get(get_turtles))
        .route("/turtle/{name}/ping", get(ping))
        .route("/turtle/{name}/reboot", get(reboot))
        .with_state(manager)
}
