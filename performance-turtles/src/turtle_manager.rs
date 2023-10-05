mod turtle_connection;

use actix::prelude::*;

pub struct TurtleManager {

}

impl Actor for TurtleManager {
    type Context = Context<Self>;
}
