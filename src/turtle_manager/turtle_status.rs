use super::turtle_connection::TurtleConnection;

#[derive(Debug)]
pub struct AlreadyConnectedError {
    name: &'static str,
    connection: TurtleConnection,
}

#[derive(Debug)]
pub struct AlreadyDisconnectedError {
    name: &'static str,
}

pub enum TurtleStatus {
    Connected {
        name: &'static str,
        connection: TurtleConnection,
    },
    Disconnected(&'static str),
}

impl TurtleStatus {
    pub fn get_name(&self) -> &'static str {
        match self {
            TurtleStatus::Connected { name, .. } => name,
            TurtleStatus::Disconnected(name) => name,
        }
    }

    pub fn connect(self, connection: TurtleConnection) -> Result<Self, AlreadyConnectedError> {
        match self {
            TurtleStatus::Connected { name, connection } => {
                Err(AlreadyConnectedError { name, connection })
            }
            TurtleStatus::Disconnected(name) => Ok(Self::Connected { name, connection }),
        }
    }
}

impl std::fmt::Display for AlreadyConnectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Turtle {} is already connected", self.name)
    }
}

impl std::fmt::Display for AlreadyDisconnectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Turtle {} is already disconnected", self.name)
    }
}

impl std::error::Error for AlreadyConnectedError {}
impl std::error::Error for AlreadyDisconnectedError {}
