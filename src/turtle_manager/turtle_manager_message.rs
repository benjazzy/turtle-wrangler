use tokio::sync::oneshot;

use super::unknown_turtle_connection::UnknownTurtleConnection;

pub enum TurtleManagerMessage {
    Close(oneshot::Sender<()>),
    UnknownTurtle(UnknownTurtleConnection),
    Disconnect(String),
    Broadcast(String),
    Status(oneshot::Sender<String>),
}
