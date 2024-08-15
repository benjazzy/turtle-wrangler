mod turtle_connection;

use std::collections::HashMap;

use actix::prelude::*;

use crate::turtle::Turtle;

pub struct TurtleManager {
    turtles: HashMap<String, Turtle>,
}

impl TurtleManager {
    pub fn new() -> Self {
        TurtleManager {
            turtles: HashMap::new(),
        }
    }
}

impl Actor for TurtleManager {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterTurtle(pub Turtle);

impl Handler<RegisterTurtle> for TurtleManager {
    type Result = ();

    fn handle(&mut self, msg: RegisterTurtle, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(turtle) = self.turtles.insert(msg.0.name().to_string(), msg.0) {
            turtle.close();
        }
    }
}
