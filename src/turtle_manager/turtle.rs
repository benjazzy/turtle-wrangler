use sqlx::SqlitePool;

use tracing::info;

use crate::db::turtle_operations::TurtleDB;
use crate::{
    scheme::{Coordinates, Heading},
    turtle_scheme::{RequestType, ResponseType, TurtleCommand},
};

use super::turtle_status::TurtleStatus;

#[derive(Debug)]
pub struct DisconnectedError;

#[derive(Debug, Clone)]
pub struct Turtle {
    name: &'static str,
    connection: TurtleStatus,
    db: TurtleDB<'static>,
}

impl Turtle {
    pub fn new(connection: TurtleStatus, pool: SqlitePool) -> Self {
        let name = connection.get_name();
        Turtle {
            name,
            connection,
            db: TurtleDB::new(name, pool),
        }
    }

    pub async fn status(&self) -> String {
        format!(
            "{}:\t\tStatus: {} {}",
            self.name,
            self.connection,
            self.db.status().await
        )
    }

    pub fn get_connection_mut(&mut self) -> &mut TurtleStatus {
        &mut self.connection
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_db(&self) -> &TurtleDB {
        &self.db
    }

    pub async fn send(&self, command: TurtleCommand) -> Result<(), DisconnectedError> {
        if let TurtleStatus::Connected { connection, .. } = &self.connection {
            connection.send(command).await;
        } else {
            return Err(DisconnectedError);
        }

        Ok(())
    }

    pub async fn request(&self, request: RequestType) -> Result<ResponseType, ()> {
        if let TurtleStatus::Connected { connection, .. } = &self.connection {
            connection.request(request).await
        } else {
            Err(())
        }
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

        if let TurtleStatus::Connected { connection, .. } = &self.connection {
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
