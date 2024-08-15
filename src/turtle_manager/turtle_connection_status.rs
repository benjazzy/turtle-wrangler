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
pub enum TurtleConnectionStatus {
    Connected {
        name: &'static str,
        connection: TurtleConnection,
    },
    Disconnected(&'static str),
}

impl TurtleConnectionStatus {
    pub fn get_name(&self) -> &'static str {
        match self {
            TurtleConnectionStatus::Connected { name, .. } => name,
            TurtleConnectionStatus::Disconnected(name) => name,
        }
    }

    pub fn connect(&mut self, connection: TurtleConnection) -> Result<(), AlreadyConnectedError> {
        match self {
            TurtleConnectionStatus::Connected {
                name,
                connection: _,
            } => {
                return Err(AlreadyConnectedError { name });
            }
            TurtleConnectionStatus::Disconnected(name) => {
                *self = TurtleConnectionStatus::Connected { name, connection };
            }
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), AlreadyDisconnectedError> {
        match self {
            TurtleConnectionStatus::Connected { name, connection } => {
                connection.close().await;
                *self = TurtleConnectionStatus::Disconnected(name);
            }
            TurtleConnectionStatus::Disconnected(name) => {
                return Err(AlreadyDisconnectedError { name })
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for TurtleConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            TurtleConnectionStatus::Connected { .. } => "Connected   ".green(),
            TurtleConnectionStatus::Disconnected(_) => "Disconnected".red(),
        };
        // write!(f, "{status} {}", self.get_name())
        write!(f, "{status}")
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
