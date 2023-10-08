use actix::prelude::*;
use tracing::{debug, error};

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
        let addr = ctx.address();
        let result = self.connection.try_send(SetMessageHandler(move |message| {
            addr.do_send(ReceiveMessage(message));
        }));

        if result.is_err() {
            error!(
                "Problem setting connection message handler for {}. Shutting down receiver",
                self.name
            );
            ctx.stop();
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveMessage(WebsocketMessage);

impl Handler<ReceiveMessage> for TurtleReceiver {
    type Result = ();

    fn handle(&mut self, msg: ReceiveMessage, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Got message from {} {:?}", self.name, msg.0);
    }
}
