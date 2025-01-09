use axum::extract::{ws, ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::ServiceExt;
use axum_extra::headers::UserAgent;
use axum_extra::{headers, TypedHeader};
use kameo::actor::pubsub::PubSub;
use kameo::message::{Context, Message};
use kameo::Actor;
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::debug;
use tracing_subscriber::prelude::*;
use turtle_wrangler::turtles::TurtleManager;

#[derive(Actor)]
pub struct HelloWorldActor;

pub struct Greet(String);

impl Message<Greet> for HelloWorldActor {
    type Reply = ();

    async fn handle(
        &mut self,
        Greet(greeting): Greet,
        _: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        println!("{greeting}");
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(user_agent) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    debug!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |mut ws| async move {
        ws.send(ws::Message::Text("Hello".into())).await.unwrap();
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let actor_ref = kameo::spawn(HelloWorldActor);
    actor_ref.tell(Greet(String::from("Hello, World!"))).await?;

    let pub_sub = kameo::spawn(PubSub::new());
    let turtle_manager = kameo::spawn(TurtleManager::new(pub_sub.clone()));

    turtle_wrangler::http::run(pub_sub, turtle_manager).await;

    Ok(())
}
