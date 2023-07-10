use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::turtle_scheme::TurtleCommand;

use super::{
    turtle::Turtle, turtle_manager_message::TurtleManagerMessage, turtle_status::TurtleStatus,
    unknown_turtle_connection::UnknownTurtleConnection, TurtleManagerHandle,
};

/// Contains the logic behind managing the turtle websocket connections.
pub struct TurtleManagerInner {
    /// Receives messages from TurtleManagerHandle.
    rx: mpsc::Receiver<TurtleManagerMessage>,

    /// Clone of this TurtleManagerInner's handle
    /// Given to all turtle connections so they can call disconnect when they shut down.
    own_handle: TurtleManagerHandle,

    /// List of turtles connections that have connected.
    turtles: Vec<Turtle>,
}

impl TurtleManagerInner {
    /// Creates a new TurtleMangerInner. Does not start running until run is called.
    /// Meant to be called by TurtleManagerHandle.
    pub fn new(rx: mpsc::Receiver<TurtleManagerMessage>, own_handle: TurtleManagerHandle) -> Self {
        TurtleManagerInner {
            rx,
            own_handle,
            turtles: Vec::new(),
        }
    }

    /// Starts listening for messages from our handles.
    pub async fn run(mut self) {
        // Gets set to a oneshot when we receive a close message from a handle.
        // Used to notify when we have closed.
        let mut close_tx = None;

        while let Some(message) = self.rx.recv().await {
            match message {
                TurtleManagerMessage::Close(tx) => {
                    close_tx = Some(tx);
                    break;
                }
                TurtleManagerMessage::UnknownTurtle(unknown_turtle) => {
                    self.new_unknown_turtle(unknown_turtle).await;
                }
                TurtleManagerMessage::Disconnect(name) => self.disconnect_turtle(name).await,
                TurtleManagerMessage::Broadcast(command) => self.broadcast(command).await,
                TurtleManagerMessage::Status(tx) => {
                    let names: String = self
                        .turtles
                        .iter()
                        .map(|t| format!("{}\n", t.get_connection()))
                        .collect();
                    let names = names.trim().to_string();

                    if tx.send(names).is_err() {
                        error!("Problem sending status");
                    };
                }
                TurtleManagerMessage::GetTurtle { name, tx } => self.get_turtle(name.as_str(), tx),
            }
        }

        info!("Turtle manager closing");
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    /// Registers a new turtle that has not been identified.
    /// Identifies the turtle and adds it to self.turtles.
    async fn new_unknown_turtle(&mut self, unknown_turtle: UnknownTurtleConnection) {
        if let Some((name, connection)) = unknown_turtle.auth(self.own_handle.clone()).await {
            for turtle in self.turtles.iter_mut() {
                if turtle.get_name() == name {
                    if let Err(e) = turtle.get_connection_mut().connect(connection) {
                        error!("Problem authing turtle {e}");
                        let _ = turtle.get_connection_mut().disconnect().await;
                    }
                    return;
                }
            }

            self.turtles
                .push(Turtle::new(TurtleStatus::Connected { name, connection }));
        };
    }

    // Forces a turtle to disconnect.
    async fn disconnect_turtle(&mut self, name: String) {
        for turtle in self.turtles.iter_mut() {
            if turtle.get_name() == name {
                if let Err(e) = turtle.get_connection_mut().disconnect().await {
                    error!("Problem disconnecting turtle {e}");
                }
                return;
            }
        }

        error!("Turtle named {name} attempted to disconnect without authing");
    }

    /// Send a message to all connected turtles.
    async fn broadcast(&mut self, command: TurtleCommand) {
        for turtle in self.turtles.iter() {
            let _ = turtle.send(command.clone()).await;
        }
    }

    fn get_turtle(&self, name: &str, tx: oneshot::Sender<Option<Turtle>>) {
        let turtle = self.get_turtle_by_name(name);
        tx.send(turtle);
    }

    fn get_turtle_by_name(&self, name: &str) -> Option<Turtle> {
        self.turtles
            .iter()
            .filter(|t| t.get_name() == name)
            .map(Clone::clone)
            .next()
    }
}
