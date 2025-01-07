use std::collections::HashMap;
use std::sync::Arc;

use actix::prelude::*;
use serde::Deserialize;
use tokio::sync::oneshot;
use tracing::{debug, error, warn};

use crate::notifications::{Note, Notification, NotificationRouter, Notify, Warning};
use crate::turtle::{turtle_sender, Close};
use crate::turtle_scheme::TurtleInformation;

use super::turtle_connection::{SetMessageHandler, TurtleConnection, WebsocketMessage};
use super::turtle_sender::TurtleSenderActor;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum TurtleEvents {
    Ok {
        id: u64,
    },
    Ready,
    Response {
        id: u64,
        response: serde_json::Value,
    },
    Info {
        info: TurtleInformation,
    },
}

pub struct TurtleReceiver {
    name: Arc<str>,
    connection: Addr<TurtleConnection>,
    sender: Addr<TurtleSenderActor>,
    response_listeners: HashMap<u64, oneshot::Sender<serde_json::Value>>,
    router: Addr<NotificationRouter>,
    next_response_id: u64,
}

impl TurtleReceiver {
    pub fn new(
        name: Arc<str>,
        connection: Addr<TurtleConnection>,
        sender: Addr<TurtleSenderActor>,
        router: Addr<NotificationRouter>,
    ) -> Self {
        TurtleReceiver {
            name,
            connection,
            sender,
            response_listeners: HashMap::new(),
            router,
            next_response_id: 0,
        }
    }

    fn send_notification(&self, ctx: &mut Context<Self>, notification: Notification) {
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
    fn stopped(&mut self, _ctx: &mut Self::Context) {
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

        match msg.0 {
            WebsocketMessage::Text(message) => {
                let result = serde_json::from_str::<TurtleEvents>(message.as_str());
                let event = match result {
                    Ok(event) => event,
                    Err(e) => {
                        warn!("Problem deserializing turtle event {e}");
                        return;
                    }
                };

                let result = match event {
                    TurtleEvents::Ready => self
                        .sender
                        .try_send(turtle_sender::SetReady)
                        .map_err(|_| {}),
                    TurtleEvents::Ok { id } => self
                        .sender
                        .try_send(turtle_sender::SetOk(id))
                        .map_err(|_| {}),
                    TurtleEvents::Response { id, response } => {
                        if let Some(tx) = self.response_listeners.remove(&id) {
                            tx.send(response).map_err(|_| {})
                        } else {
                            Err(())
                        }
                    }
                    TurtleEvents::Info { info } => {
                        self.send_notification(
                            ctx,
                            Notification::Note(Note::TurtleInfo(self.name.clone(), info)),
                        );
                        Ok(())
                    }
                };

                if result.is_err() {
                    warn!(
                        "Problem sending ok, ready, or response to turtle sender inner for {}",
                        self.name
                    );
                }
            }
            WebsocketMessage::Close => {
                self.send_notification(
                    ctx,
                    Notification::Warning(Warning::TurtleClosed(self.name.clone())),
                );
            }
        };
    }
}

#[derive(Debug, Message)]
#[rtype(result = "u64")]
pub struct ResponseListener(pub oneshot::Sender<serde_json::Value>);

impl Handler<ResponseListener> for TurtleReceiver {
    type Result = u64;

    fn handle(&mut self, msg: ResponseListener, _ctx: &mut Self::Context) -> Self::Result {
        let id = self.next_response_id;
        self.next_response_id += 1;
        self.response_listeners.insert(id, msg.0);

        id
    }
}

impl Handler<Close> for TurtleReceiver {
    type Result = ();

    fn handle(&mut self, _msg: Close, ctx: &mut Self::Context) -> Self::Result {
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
