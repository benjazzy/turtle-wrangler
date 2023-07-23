use sqlx::SqlitePool;
use tokio::sync::mpsc;

use tracing::info;

use crate::db::turtle_operations::TurtleDB;
use crate::scheme::Direction;
use crate::turtle_manager::TurtleConnectionMessage;
use crate::turtle_scheme::TurtleEvents;
use crate::{
    scheme::{Coordinates, Heading},
    turtle_scheme::{RequestType, ResponseType, TurtleCommand},
};

use super::turtle_connection_status::TurtleConnectionStatus;

#[derive(Debug)]
pub struct DisconnectedError;

pub enum TurtleStatus {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct Turtle {
    name: &'static str,
    connection: TurtleConnectionStatus,
    db: TurtleDB<'static>,
}

impl Turtle {
    pub fn new(connection: TurtleConnectionStatus, pool: SqlitePool) -> Self {
        let name = connection.get_name();
        Turtle {
            name,
            connection,
            db: TurtleDB::new(name, pool),
        }
    }

    pub fn get_status(&self) -> TurtleStatus {
        match &self.connection {
            TurtleConnectionStatus::Connected { .. } => TurtleStatus::Connected,
            TurtleConnectionStatus::Disconnected(_) => TurtleStatus::Disconnected,
        }
    }

    pub async fn status_string(&self) -> String {
        format!(
            "{}:\t\tStatus: {} {}",
            self.name,
            self.connection,
            self.db.status().await
        )
    }

    pub fn get_connection_mut(&mut self) -> &mut TurtleConnectionStatus {
        &mut self.connection
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_db(&self) -> &TurtleDB {
        &self.db
    }

    pub async fn client_subscribe(
        &self,
        tx: mpsc::UnboundedSender<TurtleConnectionMessage<'static>>,
    ) -> Result<(), DisconnectedError> {
        if let TurtleConnectionStatus::Connected { connection, .. } = &self.connection {
            connection.client_subscribe(tx).await;
        } else {
            return Err(DisconnectedError);
        }

        Ok(())
    }

    pub async fn send(&self, command: TurtleCommand) -> Result<(), DisconnectedError> {
        if let TurtleConnectionStatus::Connected { connection, .. } = &self.connection {
            connection.send(command).await;
        } else {
            return Err(DisconnectedError);
        }

        Ok(())
    }

    pub async fn request(&self, request: RequestType) -> Result<ResponseType, ()> {
        if let TurtleConnectionStatus::Connected { connection, .. } = &self.connection {
            connection.request(request).await
        } else {
            Err(())
        }
    }

    pub async fn move_turtle(&self, direction: Direction) -> Result<(), DisconnectedError> {
        if let TurtleConnectionStatus::Connected { connection, .. } = &self.connection {
            connection.send(TurtleCommand::Move { direction }).await;
        } else {
            return Err(DisconnectedError);
        }

        Ok(())
    }

    pub async fn send_position_update(&self) {
        let position = match self.db.get_coordinates().await {
            Some(p) => p,
            None => Coordinates { x: 0, y: 0, z: 0 },
        };

        let heading = match self.db.get_heading().await {
            Some(h) => h,
            None => Heading::North,
        };

        info!(
            "Updating {}'s heading and position to {position}, {heading}",
            self.name
        );

        if let TurtleConnectionStatus::Connected { connection, .. } = &self.connection {
            if let Ok(lock) = connection.lock().await {
                lock.send_position_update(position, heading).await;
            }
        }
    }
}

impl std::fmt::Display for DisconnectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to run command. Turtle is disconnected")
    }
}

impl std::error::Error for DisconnectedError {}
