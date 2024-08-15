use sqlx::SqlitePool;
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::turtle_manager::TurtleConnectionMessage;
use crate::turtle_scheme::TurtleEvents;
use crate::{
    scheme::{Coordinates, Fuel, Heading},
    turtle_scheme::TurtleCommand,
};

use super::{
    turtle::Turtle, turtle_manager_inner::TurtleManagerInner,
    turtle_manager_message::TurtleManagerMessage,
    unknown_turtle_connection::UnknownTurtleConnection,
};

/// Handle for communicating with a TurtleManagerInner.
#[derive(Debug, Clone)]
pub struct TurtleManagerHandle {
    /// Sender to send TurtleManagerMessages to a TurtleManagerInner.
    tx: mpsc::Sender<TurtleManagerMessage>,
}

impl TurtleManagerHandle {
    /// Creates a new TurtleManagerInner and starts it.
    /// Returns a handle to communicate to the TurtleManagerInner.
    pub fn new(pool: SqlitePool) -> Self {
        let (tx, rx) = mpsc::channel(100);

        let inner = TurtleManagerInner::new(rx, TurtleManagerHandle { tx: tx.clone() }, pool);
        tokio::spawn(inner.run());

        TurtleManagerHandle { tx }
    }

    /// Closes the TurtleManager.
    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(TurtleManagerMessage::Close(tx)).await.is_err() {
            error!("Problem closing turtle manager");
        }

        let _ = rx.await;
    }

    /// Called by an Acceptor when a new turtle connects.
    pub async fn new_unknown_turtle(&self, turtle: UnknownTurtleConnection) {
        if self
            .tx
            .send(TurtleManagerMessage::UnknownTurtle(turtle))
            .await
            .is_err()
        {
            error!("Problem sending new turtle to turtle manager");
        }
    }

    /// Disconnects a turtle.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the turtle to disconnect.
    pub async fn disconnect(&self, name: impl Into<String>) {
        if self
            .tx
            .send(TurtleManagerMessage::Disconnect(name.into()))
            .await
            .is_err()
        {
            error!("Problem sending disconnect to turtle manager");
        }
    }

    /// Sends a message to all connected turtles.
    ///
    /// # Arguments
    ///
    /// * `message` - Message to send to all turtles.
    pub async fn broadcast(&self, command: TurtleCommand) {
        if self
            .tx
            .send(TurtleManagerMessage::Broadcast(command))
            .await
            .is_err()
        {
            error!("Problem sending broadcast message to turtle manager");
        }
    }

    /// Gets the status of all turtles.
    /// Returns None if the TurtleManagerInner fails to send the status.
    pub async fn get_status(&self) -> Option<String> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(TurtleManagerMessage::Status(tx))
            .await
            .is_err()
        {
            error!("Problem sending list message to turtle manager");
            return None;
        }

        rx.await.ok()
    }

    pub async fn get_turtle(&self, name: impl Into<String>) -> Option<Turtle> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(TurtleManagerMessage::GetTurtle {
                tx,
                name: name.into(),
            })
            .await
            .is_err()
        {
            error!("Problem sending GetTurtle message to turtle manager");
            return None;
        }

        rx.await
            .map_err(|_| {
                error!("There was an error getting the turtle back from TurtleManagerInner");
            })
            .ok()
            .flatten()
    }

    pub async fn update_turtle_position(&self, name: impl Into<String>, position: Coordinates) {
        if self
            .tx
            .send(TurtleManagerMessage::UpdatePosition {
                name: name.into(),
                position,
            })
            .await
            .is_err()
        {
            error!("Problem sending turtle position update to turtle manager");
        }
    }

    pub async fn update_turtle_heading(&self, name: impl Into<String>, heading: Heading) {
        if self
            .tx
            .send(TurtleManagerMessage::UpdateHeading {
                name: name.into(),
                heading,
            })
            .await
            .is_err()
        {
            error!("Problem sending turtle heading update to turtle manager");
        }
    }

    pub async fn update_turtle_fuel(&self, name: impl Into<String>, fuel: Fuel) {
        if self
            .tx
            .send(TurtleManagerMessage::UpdateFuel {
                name: name.into(),
                fuel,
            })
            .await
            .is_err()
        {
            error!("Problem sending turtle fuel update to turtle manager");
        }
    }

    pub async fn send_turtle_position(&self, name: impl Into<String>) {
        if self
            .tx
            .send(TurtleManagerMessage::SendTurtlePosition(name.into()))
            .await
            .is_err()
        {
            error!("Problem sending send turtle position to turtle manager");
        }
    }

    pub async fn client_subscribe(
        &self,
        tx: mpsc::UnboundedSender<TurtleConnectionMessage<'static>>,
    ) {
        if self
            .tx
            .send(TurtleManagerMessage::ClientSubscription(tx))
            .await
            .is_err()
        {
            error!("Problem sending client subscription to turtle manager");
        }
    }
}
