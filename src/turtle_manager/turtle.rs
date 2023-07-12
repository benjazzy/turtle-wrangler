use colored::Colorize;

use crate::{
    scheme::{Fuel, Heading, Position},
    turtle_scheme::TurtleCommand,
};

use super::turtle_status::TurtleStatus;

#[derive(Debug)]
pub struct DisconnectedError;

#[derive(Debug, Clone)]
pub struct TurtleState {
    pub position: Option<Position>,
    pub heading: Option<Heading>,
    pub fuel: Option<Fuel>,
}

impl TurtleState {
    pub fn new() -> Self {
        TurtleState {
            position: None,
            heading: None,
            fuel: None,
        }
    }
}

impl std::fmt::Display for TurtleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position = match self.position {
            Some(p) => p.to_string(),
            None => "Unknown".yellow().to_string(),
        };
        let heading = match self.heading {
            Some(h) => h.to_string(),
            None => "Unknown".yellow().to_string(),
        };
        let fuel = match self.fuel {
            Some(f) => f.to_string(),
            None => "Unknown".yellow().to_string(),
        };

        write!(
            f,
            "Position: {} Heading: {}, Fuel: {}",
            position, heading, fuel
        )
    }
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
            state: TurtleState::new(),
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

    pub fn update_position(&mut self, position: Position) {
        self.state.position = Some(position);
    }

    pub fn update_heading(&mut self, heading: Heading) {
        self.state.heading = Some(heading);
    }

    pub fn update_fuel(&mut self, fuel: Fuel) {
        self.state.fuel = Some(fuel);
    }
}

impl std::fmt::Display for Turtle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:\t\tStatus: {} {}",
            self.name, self.connection, self.state
        )
    }
}

impl std::fmt::Display for DisconnectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to run command. Turtle is disconnected")
    }
}

impl std::error::Error for DisconnectedError {}
