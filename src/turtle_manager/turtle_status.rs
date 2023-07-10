use colored::Colorize;

use super::turtle_connection::TurtleConnection;

#[derive(Debug)]
pub struct AlreadyConnectedError {
    name: &'static str,
}

#[derive(Debug)]
pub struct AlreadyDisconnectedError {
    name: &'static str,
}

#[derive(Debug, Clone)]
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

    pub fn connect(&mut self, connection: TurtleConnection) -> Result<(), AlreadyConnectedError> {
        match self {
            TurtleStatus::Connected {
                name,
                connection: _,
            } => {
                return Err(AlreadyConnectedError { name });
            }
            TurtleStatus::Disconnected(name) => {
                *self = TurtleStatus::Connected { name, connection };
            }
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), AlreadyDisconnectedError> {
        match self {
            TurtleStatus::Connected { name, connection } => {
                connection.close().await;
                *self = TurtleStatus::Disconnected(name);
            }
            TurtleStatus::Disconnected(name) => return Err(AlreadyDisconnectedError { name }),
        }

        Ok(())
    }
}

impl std::fmt::Display for TurtleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            TurtleStatus::Connected { .. } => "Connected:   ".green(),
            TurtleStatus::Disconnected(_) => "Disconnected:".red(),
        };
        write!(f, "{status} {}", self.get_name())
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
