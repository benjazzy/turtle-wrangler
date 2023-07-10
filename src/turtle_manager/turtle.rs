use crate::{
    scheme::{Fuel, Heading, Position},
    turtle_scheme::TurtleCommand,
};

use super::turtle_status::TurtleStatus;

#[derive(Debug)]
pub struct DisconnectedError;

#[derive(Debug, Clone)]
pub enum TurtleState {
    Unknown,
    Known(TurtleStateData),
}

#[derive(Debug, Clone)]
pub struct TurtleStateData {
    pub position: Position,
    pub heading: Heading,
    pub fuel: Fuel,
}

#[derive(Debug, Clone)]
pub struct Turtle {
    name: &'static str,
    connection: TurtleStatus,
    state: TurtleState,
}

impl Turtle {
    pub fn new(connection: TurtleStatus) -> Self {
        Turtle {
            name: connection.get_name(),
            connection,
            state: TurtleState::Unknown,
        }
    }

    pub fn get_connection(&self) -> &TurtleStatus {
        &self.connection
    }

    pub fn get_connection_mut(&mut self) -> &mut TurtleStatus {
        &mut self.connection
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_state(&self) -> &TurtleState {
        &self.state
    }

    pub async fn send(&self, command: TurtleCommand) -> Result<(), DisconnectedError> {
        if let TurtleStatus::Connected { connection, .. } = &self.connection {
            connection.send(command).await;
        } else {
            return Err(DisconnectedError);
        }

        Ok(())
    }
}

impl std::fmt::Display for DisconnectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to run command. Turtle is disconnected")
    }
}

impl std::error::Error for DisconnectedError {}
