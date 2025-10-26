use crate::entities;
use crate::turtles::{
    identify_turtle, GetConnectedTurtles, GetTurtle, TurtleManager, TurtleNotification,
};
use axum::body::Body;
use axum::extract::{ws, ConnectInfo, Path, Query, Request, State, WebSocketUpgrade};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{body, Json, Router};
use axum_extra::{headers, TypedHeader};
use futures::StreamExt;
use kameo::actor::pubsub::PubSub;
use kameo::actor::ActorRef;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};
use turtle_types::turtle_scheme::{turtle_messages, ToolSide};
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

    STARTUP_SCRIPT.replace("$ip$", "http://{host}/scripts/run.lua")
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

async fn get_pos(
    State(TurtleState { pub_sub: _, db }): State<TurtleState>,
    Path(turtle_name): Path<String>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
    let turtle = entities::turtles::Entity::find()
        .filter(entities::turtles::Column::Name.eq(turtle_name))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let coordinates = turtle_scheme::Coordinates {
        x: turtle.x as i64,
        y: turtle.y as i64,
        z: turtle.z as i64,
    };

    let position = turtle_scheme::Position {
        coordinates,
        heading: turtle.heading,
    };

    Ok(Json(position))
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

async fn forward(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
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
        .command(turtle_messages::Forward {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn backward(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
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
        .command(turtle_messages::Backward {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn turn_left(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
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
        .command(turtle_messages::TurnLeft {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn turn_right(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
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
        .command(turtle_messages::TurnRight {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn up(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
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
        .command(turtle_messages::Up {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn down(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_scheme::Position>, StatusCode> {
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
        .command(turtle_messages::Down {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[axum::debug_handler]
async fn inspect(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
) -> Result<Json<turtle_messages::Inspection>, StatusCode> {
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
        .command(turtle_messages::Inspect {})
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Debug, Serialize, Deserialize)]
struct DigSide {
    side: ToolSide,
}

#[axum::debug_handler]
async fn dig(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, StatusCode> {
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
        .command(turtle_messages::Dig { side })
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[axum::debug_handler]
async fn dig_up(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, StatusCode> {
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
        .command(turtle_messages::DigUp { side })
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[axum::debug_handler]
async fn dig_down(
    Path(turtle_name): Path<String>,
    State(manager): State<ActorRef<TurtleManager>>,
    Query(DigSide { side }): Query<DigSide>,
) -> Result<Json<bool>, StatusCode> {
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
        .command(turtle_messages::DigDown { side })
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn substitute_url(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let (mut parts, body) = response.into_parts();
    let body = body::to_bytes(body, usize::MAX).await.unwrap();
    let body = String::from_utf8(body.into()).unwrap();
    let host = std::env::var("TURTLE_SERVER_HOST").unwrap_or("127.0.0.1:8080".to_owned());
    let body = body.replace("{host}", &host);
    let len = body.len();
    parts.headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&len.to_string()).unwrap(),
    );

    Response::from_parts(parts, Body::from(body))
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
        .layer(middleware::from_fn(substitute_url))
        .route("/ws", get(ws_handler))
        .route("/turtle/{name}/position", get(get_pos))
        .with_state(TurtleState { pub_sub, db })
        .route("/turtles", get(get_turtles))
        .route("/turtle/{name}/ping", get(ping))
        .route("/turtle/{name}/reboot", get(reboot))
        .route("/turtle/{name}/forward", get(forward))
        .route("/turtle/{name}/backward", get(backward))
        .route("/turtle/{name}/turn_left", get(turn_left))
        .route("/turtle/{name}/turn_right", get(turn_right))
        .route("/turtle/{name}/up", get(up))
        .route("/turtle/{name}/down", get(down))
        .route("/turtle/{name}/inspect", get(inspect))
        .route("/turtle/{name}/dig", get(dig))
        .route("/turtle/{name}/digUp", get(dig_up))
        .route("/turtle/{name}/digDown", get(dig_down))
        .with_state(manager)
}
