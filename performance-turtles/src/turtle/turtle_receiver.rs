use actix::prelude::*;
use tracing::{debug, error};

use crate::turtle::turtle_connection;

use super::turtle_connection::{SetMessageHandler, TurtleConnection, WebsocketMessage};

pub struct TurtleReceiver {
    name: String,
    connection: Addr<TurtleConnection>,
}

impl TurtleReceiver {
    pub fn new(name: impl Into<String>, connection: Addr<TurtleConnection>) -> Self {
        TurtleReceiver {
            name: name.into(),
            connection,
        }
    }
}

impl Actor for TurtleReceiver {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let weak = ctx.address().downgrade();
        let result =
            self.connection
                .try_send(SetMessageHandler(move |message| match weak.upgrade() {
                    Some(addr) => {
                        addr.do_send(ReceiveMessage(message));

                        Ok(())
                    }
                    None => {
                        error!("Problem sending message to receiver");

                        Err(())
                    }
                }));

        if result.is_err() {
            error!(
                "Problem setting connection message handler for {}. Shutting down receiver",
                self.name
            );
            ctx.stop();
        }
    }
    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.connection.do_send(turtle_connection::CloseMessage);
        debug!("TurtleReceiver closed");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveMessage(WebsocketMessage);

impl Handler<ReceiveMessage> for TurtleReceiver {
    type Result = ();

    fn handle(&mut self, msg: ReceiveMessage, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Got message from {} {:?}", self.name, msg.0);
        //TODO finish closing the connection.
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CloseReceiver;

impl Handler<CloseReceiver> for TurtleReceiver {
    type Result = ();

    fn handle(&mut self, msg: CloseReceiver, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
