use axum::extract::{ConnectInfo, Request, State, WebSocketUpgrade};
use axum::response::{IntoResponse, Response};
use axum_extra::{headers, TypedHeader};
use std::net::SocketAddr;
use axum::middleware::Next;
use axum::{body, middleware, Router};
use axum::body::Body;
use axum::http::header;
use axum::routing::get;
use axum_extra::headers::HeaderValue;
use tower_http::services::ServeDir;
use tracing::{debug, info};
use crate::http::turtle_state::TurtleState;
use crate::turtles::identify_turtle;

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
    State(TurtleState { pub_sub, database, .. }): State<TurtleState>,
) -> impl IntoResponse {
    let user_agent = if let Some(user_agent) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    debug!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |ws| async move {
        identify_turtle(ws, pub_sub, database).await;
    })
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

pub fn router() -> Router<TurtleState> {
    let scripts_dir = std::env::var("SCRIPTS_DIR").unwrap_or("../scripts".to_string());
    info!("Serving scripts from {scripts_dir}");
    Router::new()
        .nest_service("/scripts", ServeDir::new(scripts_dir))
        .route("/startup.lua", get(get_startup_script))
        .layer(middleware::from_fn(substitute_url))
        .route("/ws", get(ws_handler))
}