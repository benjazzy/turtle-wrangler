use crate::scheme::{Coordinates, Fuel, Heading, TurtleType};
use colored::Colorize;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Row, SqlitePool};
use tracing::error;

#[derive(Debug, Clone)]
pub struct TurtleDB<'a> {
    name: &'a str,
    pool: SqlitePool,
}

impl<'a> TurtleDB<'a> {
    pub fn new(name: &'a str, pool: SqlitePool) -> Self {
        TurtleDB { name, pool }
    }

    ////////////////////////////////////////////////////
    // General
    ////////////////////////////////////////////////////

    pub async fn get_type(&self) -> Option<TurtleType> {
        let row = sqlx::query("SELECT type FROM turtles WHERE name = ?")
            .bind(self.name)
            .fetch_one(&self.pool)
            .await
            .ok()?;

        TurtleType::from_str(row.try_get(0).ok()?)
    }

    pub async fn status(&self) -> String {
        let coordinates = match self.get_coordinates().await {
            Some(p) => p.to_string(),
            None => "Unknown".yellow().to_string(),
        };
        let heading = match self.get_heading().await {
            Some(h) => h.as_str().to_string(),
            None => "Unknown".yellow().to_string(),
        };
        let fuel = match self.get_fuel().await {
            Some(f) => f.to_string(),
            None => "Unknown".yellow().to_string(),
        };

        format!("Position: {coordinates} Heading: {heading}, Fuel: {fuel}")
    }

    ////////////////////////////////////////////////////
    // Position
    ////////////////////////////////////////////////////

    pub async fn get_coordinates(&self) -> Option<Coordinates> {
        sqlx::query_as::<_, Coordinates>("SELECT x, y, z FROM turtles WHERE name = ?")
            .bind(self.name)
            .fetch_one(&self.pool)
            .await
            .ok()
    }

    pub async fn set_coordinates(
        &self,
        coordinates: Coordinates,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE turtles SET x = ?, y = ?, z = ? WHERE name = ?")
            .bind(coordinates.x)
            .bind(coordinates.y)
            .bind(coordinates.z)
            .bind(self.name)
            .execute(&self.pool)
            .await
    }

    pub async fn get_heading(&self) -> Option<Heading> {
        let heading = sqlx::query("SELECT heading FROM turtles WHERE name = ?")
            .bind(self.name)
            .fetch_one(&self.pool)
            .await
            .ok()?;

        Heading::from_str(heading.try_get(0).ok()?)
    }

    pub async fn set_heading(&self, heading: Heading) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE turtles SET heading = ? WHERE name = ?")
            .bind(heading.as_str())
            .bind(self.name)
            .execute(&self.pool)
            .await
    }

    ////////////////////////////////////////////////////
    // Fuel
    ////////////////////////////////////////////////////

    pub async fn get_fuel_level(&self) -> Option<u32> {
        let level_row = sqlx::query("SELECT fuel FROM turtles WHERE name = ?")
            .bind(self.name)
            .fetch_one(&self.pool)
            .await
            .ok()?;

        level_row.try_get(0).ok()
    }

    pub async fn get_max_fuel(&self) -> Option<u32> {
        Some(self.get_type().await?.get_max_fuel())
    }

    pub async fn get_fuel(&self) -> Option<Fuel> {
        let max = self.get_max_fuel().await?;

        let level_row = sqlx::query("SELECT fuel FROM turtles WHERE name = ?")
            .bind(self.name)
            .fetch_one(&self.pool)
            .await
            .ok()?;

        let level: u32 = level_row.try_get(0).ok()?;

        Some(Fuel { level, max })
    }

    pub async fn set_fuel(&self, fuel: u32) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE turtles SET fuel = ? WHERE name = ?")
            .bind(fuel)
            .bind(self.name)
            .execute(&self.pool)
            .await
    }
}

pub async fn turtle_exists(name: &str, pool: &SqlitePool) -> bool {
    let row = sqlx::query("SELECT name FROM turtles WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await;

    match row {
        Ok(r) => !r.is_empty(),
        Err(_e) => {
            error!("Problem checking if turtle {name} exists");
            false
        }
    }
}

pub async fn add_turtle(
    name: &str,
    coordinates: Coordinates,
    heading: Heading,
    turtle_type: TurtleType,
    pool: &SqlitePool,
) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query(
        "INSERT INTO turtles\
        (name, x, y, z, heading, type, fuel) \
        VALUES (?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(name)
    .bind(coordinates.x)
    .bind(coordinates.y)
    .bind(coordinates.z)
    .bind(heading.as_str())
    .bind(turtle_type.as_str())
    .execute(pool)
    .await
}
