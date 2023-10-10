use std::any::Any;
use std::collections::HashMap;

use actix::prelude::*;
use tracing::{debug, error};

use crate::turtle::turtle_connection;
use crate::turtle_notifications::{
    ConnectionClosed, TurtleNotification, TurtleNotificationData, TurtleNotificationFilter,
};

use super::turtle_connection::{SetMessageHandler, TurtleConnection, WebsocketMessage};

enum NotificationHandlerType {
    Closed(Box<dyn FnMut(ConnectionClosed)>),
}

impl NotificationHandlerType {
    pub fn new<F>(handler: F) -> Self
    where
        F: FnMut(ConnectionClosed) + 'static,
    {
        NotificationHandlerType::Closed(Box::new(handler))
    }
}

pub struct TurtleReceiver {
    name: String,
    connection: Addr<TurtleConnection>,
    listeners: HashMap<String, NotificationHandlerType>,
    next_id: usize,
}

impl TurtleReceiver {
    pub fn new(name: impl Into<String>, connection: Addr<TurtleConnection>) -> Self {
        TurtleReceiver {
            name: name.into(),
            connection,
            listeners: HashMap::new(),
            next_id: 0,
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

// #[derive(Message)]
// #[rtype(result = "usize")]
// pub struct RegisterRecipient<N: TurtleNotificationData + Send + 'static>(Recipient<N>);
//
// impl<N> Handler<RegisterRecipient<N>> for TurtleReceiver
// where
//     N: TurtleNotificationData + Send,
// {
//     type Result = usize;
//
//     fn handle(&mut self, msg: RegisterRecipient<N>, ctx: &mut Self::Context) -> Self::Result {
//         let id = self.next_id;
//         self.next_id += 1;
//
//         let recipient = msg.0.clone();
//         let func = move |notification: N| {
//             recipient.do_send(notification);
//         };
//
//         let handler = NotificationHandlerType::new(func);
//
//         // let
//
//         // if let Some(listener_list) = self.listeners.get_mut(N::NAME) {
//         //     listener_list.push(Box::new(func));
//         // };
//
//         id
//     }
// }
