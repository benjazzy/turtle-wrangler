mod acceptor;
mod turtle_manager;

use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::turtle_manager::TurtleManagerHandle;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "turtle_wrangler=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    start().await;
    info!("Shutting down");
}

async fn start() {
    info!("Starting Turtle Wragler");

    let turtle_manager = TurtleManagerHandle::new();

    let acceptor =
        acceptor::AcceptorHandle::new("127.0.0.1:8080".to_string(), turtle_manager.clone());

    tokio::time::sleep(Duration::from_secs(10)).await;

    turtle_manager.broadcast("Hello Turtle".to_string()).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    acceptor.close().await;
    turtle_manager.close().await;
}
