use axum::extract::{ws, ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum_extra::{headers, TypedHeader};
use kameo::message::{Context, Message};
use kameo::Actor;
use kameo_actors::pubsub::PubSub;
use sea_orm::{ConnectOptions, Database};
use std::net::SocketAddr;
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
        _: &mut Context<Self, Self::Reply>,
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

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./turtles.sqlite?mode=rwc".to_owned());
    let mut db_opt = ConnectOptions::new(database_url);
    db_opt.sqlx_logging(false);
    let db = Database::connect(db_opt).await?;
    db.get_schema_registry("turtle-entities::*")
        .sync(&db)
        .await?;

    let actor_ref = HelloWorldActor::spawn(HelloWorldActor);
    actor_ref.tell(Greet(String::from("Hello, World!"))).await?;

    let pub_sub = PubSub::spawn(PubSub::new(kameo_actors::DeliveryStrategy::Guaranteed));
    let turtle_manager = TurtleManager::spawn(TurtleManager::new(pub_sub.clone(), db.clone()));

    turtle_wrangler::http::run(pub_sub, turtle_manager, db.clone()).await;

    db.close().await?;

    Ok(())
}
