#![feature(async_closure)]

/// Acceptor handles listening for incoming tcp connections and upgrading them to a websocket
/// connection.
mod acceptor;

/// Blocks contains all the blocks that turtle_wrangler is aware of and their associated data.
mod blocks;

mod client_manager;

mod client_scheme;

mod command_interpreter;

mod db;

/// Manages turtle websocket connections.
mod turtle_manager;

/// Messages that can be sent to and from a turtle websocket connection.
mod turtle_scheme;

mod scheme;

use tokio::{runtime::Handle, sync::oneshot};

use crate::client_manager::ClientManagerHandle;
use tracing::{error, info};
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
    info!("Starting Turtle Wrangler");

    let db_path = match std::env::var("DB") {
        Ok(p) => p,
        Err(_) => {
            error!("DB environment variable not set. Exiting");
            return;
        }
    };

    let pool = match db::setup_database(db_path.as_str()).await {
        Ok(p) => p,
        Err(e) => {
            error!("Problem setting up database {e}");
            return;
        }
    };

    let turtle_manager = TurtleManagerHandle::new(pool.clone());
    let turtle_acceptor =
        acceptor::AcceptorHandle::new_websocket("0.0.0.0:8080".to_string(), turtle_manager.clone());

    let client_manager = ClientManagerHandle::new();
    let client_acceptor = acceptor::AcceptorHandle::new_client(
        "0.0.0.0:8081".to_string(),
        client_manager.clone(),
        turtle_manager.clone(),
        pool.clone(),
    );

    let (tx, rx) = oneshot::channel();
    let manager = turtle_manager.clone();

    let handle = Handle::current();
    std::thread::spawn(move || command_interpreter::read_input(tx, manager, handle, pool));
    rx.await.unwrap();

    turtle_acceptor.close().await;
    turtle_manager.close().await;
    client_acceptor.close().await;
    client_manager.close().await;
}
