use crate::turtle_scheme::TurtleEvents;

#[derive(Debug, Clone)]
pub struct TurtleConnectionMessage<'a> {
    pub name: &'a str,
    pub message_type: ConnectionMessageType,
}

#[derive(Debug, Clone)]
pub enum ConnectionMessageType {
    TurtleEvent(TurtleEvents),
    Connected,
    Disconnected,
}
