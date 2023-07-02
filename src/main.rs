mod acceptor;
mod turtle_manager;

use std::{io, sync::Arc, time::Duration};

use tokio::{
    io::BufReader,
    runtime::Handle,
    sync::{oneshot, Mutex},
};
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

    let (tx, rx) = oneshot::channel();
    let manager = turtle_manager.clone();

    let handle = Handle::current();
    std::thread::spawn(move || read_input(tx, manager, handle));
    rx.await.unwrap();

    acceptor.close().await;
    turtle_manager.close().await;
}

fn read_input(
    close_tx: oneshot::Sender<()>,
    turtle_manager: TurtleManagerHandle,
    async_handle: Handle,
) {
    println!("Q to quit");
    let mut buffer = String::new();
    while buffer.to_uppercase() != "Q" {
        buffer.clear();
        print!("> ");

        io::stdin().read_line(&mut buffer).unwrap();

        if buffer.to_uppercase() != "Q" {
            let turtle_manager = turtle_manager.clone();
            let buffer = buffer.clone();

            async_handle.spawn(async move { turtle_manager.broadcast(buffer).await });
        }
    }

    close_tx.send(()).unwrap();
}
