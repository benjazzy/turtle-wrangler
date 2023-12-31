//TurtleManager.

/// Communicates with a TurtleManagerInner.
mod turtle_manager_handle;

/// The logic behind managing all the connected turtle websockets.
mod turtle_manager_inner;

/// Messages that can be sent from a TurtleManagerHandle to a TurtleManagerHandle.
mod turtle_manager_message;

// TurtleSender

/// Communicates with a TurtleSenderInner.
mod turtle_sender_handle;

/// The logic behind sending messages to a turtle.
mod turtle_sender_inner;

/// Messages that can be sent from a TurtleSenderHandle to a TurtleSenderInner.
mod turtle_sender_message;

// TurtleReceiver

/// Communicates with a TurtleReceiverInner.
mod turtle_receiver_handle;

/// The logic behind receiving messages from a turtle and passing them on to where they need to go.
mod turtle_receiver_inner;

/// Messages that can be sent from a TurtleReceiverHandle to a TurtleReceiverInner.
mod turtle_receiver_message;

// TurtleConnection

/// Contains both the turtle sender and receiver handle.
mod turtle_connection;

/// Status of the turtle. Either Connected or disconnected.
/// Contains the TurtleConnection.
mod turtle_connection_status;

/// Turtle that has not identified itself.
/// Can be turned into a TurtleConnection.
mod unknown_turtle_connection;

mod turtle_connection_message;

mod turtle;

// Exports

pub use turtle::Turtle;
pub use turtle_connection_message::{ConnectionMessageType, TurtleConnectionMessage};
pub use turtle_manager_handle::TurtleManagerHandle;
pub use turtle_receiver_handle::TurtleReceiverHandle;
pub use turtle_sender_handle::TurtleSenderHandle;
pub use unknown_turtle_connection::UnknownTurtleConnection;
