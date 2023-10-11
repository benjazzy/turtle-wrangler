use std::any::Any;
use std::collections::HashMap;

use actix::prelude::*;
use tracing::{debug, error, warn};

use crate::notifications::{Note, Notification, NotificationRouter, Notify, Warning};
use crate::turtle::{turtle_connection, turtle_sender, Close};
use crate::turtle_notifications::{
    ConnectionClosed, TurtleNotification, TurtleNotificationData, TurtleNotificationFilter,
};
use crate::turtle_scheme::TurtleEvents;

use super::turtle_connection::{SetMessageHandler, TurtleConnection, WebsocketMessage};
use super::turtle_sender::TurtleSenderInner;

pub struct TurtleReceiver {
    name: String,
    connection: Addr<TurtleConnection>,
    router: Addr<NotificationRouter>,
    sender: Addr<TurtleSenderInner>,
}

impl TurtleReceiver {
    pub fn new(
        name: impl Into<String>,
        connection: Addr<TurtleConnection>,
        router: Addr<NotificationRouter>,
        sender: Addr<TurtleSenderInner>,
    ) -> Self {
        TurtleReceiver {
            name: name.into(),
            connection,
            router,
            sender,
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
        self.connection.do_send(Close);
        debug!("TurtleReceiver closed");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveMessage(WebsocketMessage);

impl Handler<ReceiveMessage> for TurtleReceiver {
    type Result = ();

    fn handle(&mut self, msg: ReceiveMessage, ctx: &mut Self::Context) -> Self::Result {
        debug!("Got message from {} {:?}", self.name, msg.0);

        let notification = match msg.0 {
            WebsocketMessage::Text(message) => {
                let result = serde_json::from_str::<TurtleEvents>(message.as_str());
                let event = match result {
                    Ok(event) => event,
                    Err(e) => {
                        warn!("Problem deserializing turtle event {e}");
                        return;
                    }
                };

                let result = match &event {
                    TurtleEvents::Ready => {
                        self.sender.try_send(turtle_sender::Ready).map_err(|_| {})
                    }
                    TurtleEvents::Ok { id } => self
                        .sender
                        .try_send(turtle_sender::SetOk(*id))
                        .map_err(|_| {}),
                    TurtleEvents::Response { response } => self
                        .sender
                        .try_send(turtle_sender::NotifyResponse(response.clone()))
                        .map_err(|_| {}),
                    _ => Ok(()),
                };

                if result.is_err() {
                    warn!(
                        "Problem sending ok, ready, or response to turtle sender inner for {}",
                        self.name
                    );
                }

                Notification::Note(Note::TurtleEvent(self.name.clone(), event))
            }
            WebsocketMessage::Close => {
                Notification::Warning(Warning::TurtleClosed(self.name.clone()))
            }
        };

        let router = self.router.clone();
        let fut = fut::wrap_future(router.send(Notify(notification))).map(
            |result, _actor: &mut Self, _ctx| {
                if let Err(err) = result {
                    match err {
                        MailboxError::Closed => {
                            error!("Router closed before receiver");
                        }
                        MailboxError::Timeout => {
                            warn!("Router mailbox timed out");
                        }
                    }
                }
            },
        );

        ctx.spawn(fut);
    }
}

impl Handler<Close> for TurtleReceiver {
    type Result = ();

    fn handle(&mut self, msg: Close, ctx: &mut Self::Context) -> Self::Result {
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
