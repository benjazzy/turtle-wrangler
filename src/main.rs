mod acceptor;
mod turtle_manager;

use std::{io, sync::Arc, time::Duration};

use tokio::{
    io::BufReader,
    runtime::Handle,
    sync::{oneshot, Mutex},
};
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
    let mut buffer = String::new();
    while buffer.to_uppercase() != "Q" {
        buffer.clear();

        io::stdin().read_line(&mut buffer).unwrap();
        let trimed_buffer = buffer.trim_end();

        let command = if let Some(c) = trimed_buffer.chars().nth(0) {
            c
        } else {
            error!("Invalid command");
            continue;
        };

        match command.to_ascii_uppercase() {
            'R' => {
                let turtle_manager = turtle_manager.clone();
                let turtle_command = if let Some(c) = trimed_buffer.split(' ').nth(1) {
                    c.to_string()
                } else {
                    error!("Invalid command to run");
                    continue;
                };
                async_handle.spawn(async move { turtle_manager.broadcast(turtle_command).await });
            }
            'S' => {
                let turtle_manager = turtle_manager.clone();
                async_handle.spawn(async move {
                    println!(
                        "{}",
                        turtle_manager
                            .get_status()
                            .await
                            .unwrap_or("Problem getting status".to_string())
                    );
                });
            }
            'D' => {
                let name = match trimed_buffer.split_whitespace().nth(1) {
                    Some(n) => n.to_string(),
                    None => {
                        error!("Invalid disconnect command entered");
                        continue;
                    }
                };
                let turtle_manager = turtle_manager.clone();

                async_handle.spawn(async move { turtle_manager.disconnect(name).await });
            }

            c if c != 'Q' => info!("Unknown command"),
            _ => {}
        }
    }

    close_tx.send(()).unwrap();
}
