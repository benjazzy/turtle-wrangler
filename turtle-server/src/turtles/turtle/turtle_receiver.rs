use turtle_types::turtle_scheme::Turtle;
use crate::turtles::turtle::turtle_sender::TurtleSender;
use crate::turtles::turtle::{turtle_sender, TurtleNote, TurtleNotification, TurtleWarning};
use turtle_types::turtle_scheme::turtle_messages::TurtleInformation;
use axum::extract::ws;
use futures::stream::SplitStream;
use kameo::actor::pubsub::{PubSub, Publish};
use kameo::actor::{ActorRef, WeakActorRef};
use kameo::error::{ActorStopReason, BoxError};
use kameo::mailbox::unbounded::UnboundedMailbox;
use kameo::message::{Context, Message, StreamMessage};
use kameo::reply::ReplySender;
use kameo::Actor;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

type StreamHandle = JoinHandle<
    Result<
        SplitStream<ws::WebSocket>,
        kameo::error::SendError<StreamMessage<Result<ws::Message, axum::Error>, (), ()>>,
    >,
>;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
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
    connection: StreamHandle,
    outstanding_requests: HashMap<u64, oneshot::Sender<serde_json::Value>>,
    sender: ActorRef<TurtleSender>,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    next_id: u64,
}

impl TurtleReceiver {
    pub fn new(
        actor_ref: ActorRef<Self>,
        name: Arc<str>,
        sender: ActorRef<TurtleSender>,
        connection: SplitStream<ws::WebSocket>,
        pub_sub: ActorRef<PubSub<TurtleNotification>>,
    ) -> Self {
        let connection = actor_ref.attach_stream(connection, (), ());
        TurtleReceiver {
            name,
            connection,
            outstanding_requests: HashMap::new(),
            sender,
            pub_sub,
            next_id: 0,
        }
    }

    async fn handle_message(&mut self, message: ws::Message) {
        match message {
            ws::Message::Text(text) => {
                debug!("Received message from {}: {}", self.name, text.as_str());
                match serde_json::from_str::<TurtleEvents>(text.as_str()) {
                    Ok(TurtleEvents::Ok { id }) => {
                        if let Err(e) = self.sender.tell(turtle_sender::GotOk(id)).await {
                            error!(
                            "{}'s receiver got error sending ok to sender: {e}",
                            self.name
                        );
                        }
                    }
                    Ok(TurtleEvents::Ready) => {
                        if let Err(e) = self.sender.tell(turtle_sender::GotReady).await {
                            error!(
                            "{}'s receiver got error sending ok to sender: {e}",
                            self.name
                        );
                        }
                    }
                    Ok(TurtleEvents::Response { id, response }) => {
                        if let Some(tx) = self.outstanding_requests.remove(&id) {
                            let _ = tx.send(response);
                        } else {
                            warn!("Got reply for unknown request from {}", self.name);
                        }
                    }
                    Ok(TurtleEvents::Info { info }) => {
                        self.pub_sub
                            .tell(Publish(TurtleNotification::Note(TurtleNote::TurtleInfo {
                                name: self.name.clone(),
                                info,
                            })))
                            .await;
                    }
                    Err(e) => warn!(
                    "Unable to deserialize message from turtle {}: {e}",
                    self.name
                ),
                }
            },
            _ => warn!("Got invalid message from {}", self.name),
        }
    }
}

impl Actor for TurtleReceiver {
    type Mailbox = UnboundedMailbox<Self>;

    async fn on_stop(
        &mut self,
        actor_ref: WeakActorRef<Self>,
        reason: ActorStopReason,
    ) -> Result<(), BoxError> {
        self.connection.abort();

        self.pub_sub
            .tell(Publish(TurtleNotification::Warning(
                TurtleWarning::TurtleDisconnected(self.name.clone()),
            )))
            .await;

        Ok(())
    }
}

impl Message<StreamMessage<Result<ws::Message, axum::Error>, (), ()>> for TurtleReceiver {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: StreamMessage<Result<ws::Message, axum::Error>, (), ()>,
        ctx: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        match msg {
            StreamMessage::Started(_) => {}
            StreamMessage::Finished(_) => ctx.actor_ref().kill(),
            StreamMessage::Next(Ok(message)) => self.handle_message(message).await,
            StreamMessage::Next(Err(e)) => {
                warn!("Turtle receiver {} got error: {e}", self.name);
            }
        }
    }
}

#[derive(Debug)]
pub struct RegisterRequest(pub oneshot::Sender<serde_json::Value>);

impl Message<RegisterRequest> for TurtleReceiver {
    type Reply = u64;

    async fn handle(
        &mut self,
        RegisterRequest(tx): RegisterRequest,
        _: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        let id = self.next_id;
        self.next_id += 1;
        self.outstanding_requests.insert(id, tx);

        id
    }
}
