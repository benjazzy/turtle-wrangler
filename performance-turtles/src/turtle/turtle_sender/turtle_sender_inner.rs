use crate::turtle::turtle_connection;
use crate::turtle::turtle_connection::TurtleConnection;
use crate::turtle_scheme;
use crate::turtle_scheme::TurtleCommand;
use actix::prelude::*;
use tracing::debug;

pub struct TurtleSenderInner {
    connection: Addr<TurtleConnection>,
}

impl TurtleSenderInner {
    pub fn new(connection: Addr<TurtleConnection>) -> Self {
        TurtleSenderInner { connection }
    }
}

impl Actor for TurtleSenderInner {
    type Context = Context<Self>;
    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.connection.do_send(turtle_connection::CloseMessage);
        debug!("TurtleSenderInner closed");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub turtle_scheme::Message);

impl Handler<SendMessage> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        let message = serde_json::to_string(&msg.0).expect("Problem serializing turtle message");

        self.connection
            .try_send(turtle_connection::SendMessage(message));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommand(pub TurtleCommand);

impl Handler<SendCommand> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: SendCommand, ctx: &mut Self::Context) -> Self::Result {
        let message = turtle_scheme::Message::Command { command: msg.0 };
        let message = serde_json::to_string(&message).expect("Problem serializing turtle command");

        self.connection
            .try_send(turtle_connection::SendMessage(message));
    }
}
