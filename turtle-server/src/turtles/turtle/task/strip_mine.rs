use turtle_types::turtle_scheme::{Coordinates, Position};

use crate::turtles::TaskyTurtle;

pub fn strip_mine(start: Position, dropoff: Coordinates) -> impl AsyncFnOnce(&TaskyTurtle) {
    async |turtle| {}
}
