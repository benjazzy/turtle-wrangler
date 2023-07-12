use tokio::sync::oneshot;

use crate::{
    scheme::{Fuel, Heading, Position},
    turtle_scheme::TurtleCommand,
};

use super::{turtle::Turtle, unknown_turtle_connection::UnknownTurtleConnection};

/// Types of messages that can be sent from a TurtleManagerHandle to a TurtleManagerInner.
pub enum TurtleManagerMessage {
    /// Tells the inner to shutdown. The Sender allows the handle to wait for the inner to close.
    Close(oneshot::Sender<()>),

    /// Sends a new turtle connection to the inner.
    UnknownTurtle(UnknownTurtleConnection),

    /// Disconnects a turtle by name.
    Disconnect(String),

    /// Broadcasts a message.
    Broadcast(TurtleCommand),

    /// Gets the status of the connections as a formatted string.
    Status(oneshot::Sender<String>),

    GetTurtle {
        name: String,
        tx: oneshot::Sender<Option<Turtle>>,
    },

    UpdatePosition {
        name: String,
        position: Position,
    },

    UpdateHeading {
        name: String,
        heading: Heading,
    },

    UpdateFuel {
        name: String,
        fuel: Fuel,
    },
}
