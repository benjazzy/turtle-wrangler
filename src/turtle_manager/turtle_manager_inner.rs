use sqlx::SqlitePool;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

use crate::turtle_manager::turtle::TurtleStatus;
use crate::turtle_manager::{ConnectionMessageType, TurtleConnectionMessage};
use crate::turtle_scheme::TurtleEvents;
use crate::{
    scheme::{Coordinates, Fuel, Heading},
    turtle_scheme::TurtleCommand,
};

use super::{
    turtle::Turtle, turtle_connection_status::TurtleConnectionStatus,
    turtle_manager_message::TurtleManagerMessage,
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

    client_subscriptions: Vec<mpsc::UnboundedSender<TurtleConnectionMessage<'static>>>,

    pool: SqlitePool,
}

impl TurtleManagerInner {
    /// Creates a new TurtleMangerInner. Does not start running until run is called.
    /// Meant to be called by TurtleManagerHandle.
    pub fn new(
        rx: mpsc::Receiver<TurtleManagerMessage>,
        own_handle: TurtleManagerHandle,
        pool: SqlitePool,
    ) -> Self {
        TurtleManagerInner {
            rx,
            own_handle,
            turtles: Vec::new(),
            client_subscriptions: vec![],
            pool,
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
                    let names_futures = self
                        .turtles
                        .iter()
                        .map(async move |t| t.status_string().await);
                    let names = futures_util::future::join_all(names_futures)
                        .await
                        .join("\n");
                    let names = names.trim().to_string();

                    if tx.send(names).is_err() {
                        error!("Problem sending status");
                    };
                }
                TurtleManagerMessage::GetTurtle { name, tx } => self.get_turtle(name.as_str(), tx),
                TurtleManagerMessage::UpdatePosition { name, position } => {
                    self.update_turtle_position(name, position).await;
                }
                TurtleManagerMessage::UpdateHeading { name, heading } => {
                    self.update_turtle_heading(name, heading).await;
                }
                TurtleManagerMessage::UpdateFuel { name, fuel } => {
                    self.update_turtle_fuel(name, fuel).await;
                }
                TurtleManagerMessage::SendTurtlePosition(name) => {
                    self.send_turtle_position(name).await;
                }
                TurtleManagerMessage::ClientSubscription(tx) => {
                    self.client_subscribe_all(tx).await;
                }
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
            // Send the new turtle the client subscriptions so that the turtle connection can forward events to clients.
            for tx in self.client_subscriptions.iter() {
                debug!("Sending client subscription");
                connection.client_subscribe(tx.clone()).await;
            }

            debug!("Sending connected message to clients");
            Self::send_subs_message(
                &mut self.client_subscriptions,
                TurtleConnectionMessage {
                    name,
                    message_type: ConnectionMessageType::Connected,
                },
            );

            for turtle in self.turtles.iter_mut() {
                if turtle.get_name() == name {
                    if let Err(e) = turtle.get_connection_mut().connect(connection) {
                        error!("Problem authing turtle {e}");
                        let _ = turtle.get_connection_mut().disconnect().await;
                    }
                    return;
                }
            }

            self.turtles.push(Turtle::new(
                TurtleConnectionStatus::Connected { name, connection },
                self.pool.clone(),
            ));
        };
    }

    async fn client_subscribe_all(
        &mut self,
        tx: mpsc::UnboundedSender<TurtleConnectionMessage<'static>>,
    ) {
        self.client_subscriptions.push(tx.clone());

        for turtle in self.turtles.iter() {
            let _ = turtle.client_subscribe(tx.clone()).await;
        }

        for turtle in self.turtles.iter() {
            let message_type = match turtle.get_status() {
                TurtleStatus::Connected => ConnectionMessageType::Connected,
                TurtleStatus::Disconnected => ConnectionMessageType::Disconnected,
            };

            let message = TurtleConnectionMessage {
                name: turtle.get_name(),
                message_type,
            };
            Self::send_subs_message(&mut self.client_subscriptions, message);
        }
    }

    // Forces a turtle to disconnect.
    async fn disconnect_turtle(&mut self, name: String) {
        for turtle in self.turtles.iter_mut() {
            if turtle.get_name() == name {
                if let Err(e) = turtle.get_connection_mut().disconnect().await {
                    error!("Problem disconnecting turtle {e}");
                }
                Self::send_subs_message(
                    &mut self.client_subscriptions,
                    TurtleConnectionMessage {
                        name: turtle.get_name(),
                        message_type: ConnectionMessageType::Disconnected,
                    },
                );
                return;
            }
        }

        error!("Turtle named {name} attempted to disconnect without authing");
    }

    fn send_subs_message(
        client_subscriptions: &mut Vec<mpsc::UnboundedSender<TurtleConnectionMessage<'static>>>,
        message: TurtleConnectionMessage<'static>,
    ) {
        let mut subs_to_remove = vec![];
        client_subscriptions.iter().enumerate().for_each(|(i, s)| {
            if s.send(message.clone()).is_err() {
                subs_to_remove.push(i);
            }
        });
        subs_to_remove.iter().for_each(|i| {
            client_subscriptions.remove(*i);
        });
    }

    /// Send a message to all connected turtles.
    async fn broadcast(&mut self, command: TurtleCommand) {
        for turtle in self.turtles.iter() {
            let _ = turtle.send(command.clone()).await;
        }
    }

    fn get_turtle(&self, name: &str, tx: oneshot::Sender<Option<Turtle>>) {
        let turtle = self.get_turtle_by_name(name);
        let _ = tx.send(turtle);
    }

    fn get_turtle_by_name(&self, name: &str) -> Option<Turtle> {
        self.turtles
            .iter()
            .filter(|t| t.get_name() == name)
            .map(Clone::clone)
            .next()
    }

    fn get_turtle_mut_ref(&mut self, name: &str) -> Option<&mut Turtle> {
        self.turtles.iter_mut().find(|t| t.get_name() == name)
    }

    async fn update_turtle_position(&mut self, name: String, position: Coordinates) {
        if let Some(turtle) = self.get_turtle_mut_ref(name.as_str()) {
            if let Err(e) = turtle.get_db().set_coordinates(position).await {
                error!("Problem updating turtle position in db {e}");
            }
        }
    }

    async fn update_turtle_heading(&mut self, name: String, heading: Heading) {
        if let Some(turtle) = self.get_turtle_mut_ref(name.as_str()) {
            if let Err(e) = turtle.get_db().set_heading(heading).await {
                error!("Problem updating turtle heading in db {e}");
            }
        }
    }

    async fn update_turtle_fuel(&mut self, name: String, fuel: Fuel) {
        if let Some(turtle) = self.get_turtle_mut_ref(name.as_str()) {
            if let Err(e) = turtle.get_db().set_fuel(fuel.level).await {
                error!("Problem updating turtle fuel in db {e}");
            }
        }
    }

    async fn send_turtle_position(&self, name: String) {
        if let Some(turtle) = self.get_turtle_by_name(name.as_str()) {
            turtle.send_position_update().await;
        }
    }
}
