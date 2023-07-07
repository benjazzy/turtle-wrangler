/// Acceptor handles listening for incoming tcp connections and upgrading them to a websocket
/// connection.
mod acceptor;

/// Blocks contains all the blocks that turtle_wrangler is aware of and their associated data.
mod blocks;

/// Manages turtle websocket connections.
mod turtle_manager;

/// Messages that can be sent to and from a turtle websocket connection.
mod turtle_scheme;

use std::io;

use tokio::{runtime::Handle, sync::oneshot};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{turtle_manager::TurtleManagerHandle, turtle_scheme::TurtleCommand};

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
        let trimmed_buffer = buffer.trim_end();

        let command = if let Some(c) = trimmed_buffer.chars().next() {
            c
        } else {
            error!("Invalid command");
            continue;
        };

        match command.to_ascii_uppercase() {
            'R' => {
                let turtle_manager = turtle_manager.clone();
                let turtle_command_string = if let Some(c) = trimmed_buffer.split(' ').nth(1) {
                    c.to_string()
                } else {
                    error!("Invalid command to run");
                    continue;
                };

                let turtle_command = match turtle_command_string.to_uppercase().as_str() {
                    "FORWARD" => TurtleCommand::Forward,
                    "BACK" => TurtleCommand::Back,
                    "TURNLEFT" => TurtleCommand::TurnLeft,
                    "TURNRIGHT" => TurtleCommand::TurnRight,
                    "REBOOT" => TurtleCommand::Reboot,
                    "INSPECT" => TurtleCommand::Inspect,
                    _ => {
                        error!("Unknown turtle command");
                        continue;
                    }
                };
                async_handle.spawn(async move {
                    turtle_manager
                        .broadcast(serde_json::to_string(&vec![turtle_command]).unwrap())
                        .await
                });
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
                let name = match trimmed_buffer.split_whitespace().nth(1) {
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
