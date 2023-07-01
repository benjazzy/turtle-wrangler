mod acceptor;
mod turtle_manager;

use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mons=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    start().await;
}

async fn start() {
    info!("Starting Turtle Wragler");

    let senders = Arc::new(Mutex::new(Vec::new()));
    let receivers = Arc::new(Mutex::new(Vec::new()));
    let acceptor = acceptor::AcceptorHandle::new(
        "127.0.0.1:8080".to_string(),
        senders.clone(),
        receivers.clone(),
    );

    tokio::time::sleep(Duration::from_secs(10)).await;
    senders
        .lock()
        .await
        .get(0)
        .unwrap()
        .send("Test message".to_string())
        .await;

    tokio::time::sleep(Duration::from_secs(1)).await;
}
