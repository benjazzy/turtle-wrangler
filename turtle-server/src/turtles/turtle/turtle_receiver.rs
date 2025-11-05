use crate::turtles::turtle::turtle_sender::TurtleSender;
use crate::turtles::turtle::{turtle_sender, TurtleNote, TurtleNotification, TurtleWarning};
use axum::extract::ws;
use futures::stream::SplitStream;
use kameo::actor::{ActorRef, WeakActorRef};
use kameo::error::ActorStopReason;
use kameo::message::{Context, Message, StreamMessage};
use kameo::Actor;
use kameo_actors::pubsub::{PubSub, Publish};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use turtle_types::turtle_scheme::turtle_messages::TurtleInformation;

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
        #[serde(default)]
        response: serde_json::Value,
    },
    Info {
        info: TurtleInformation,
    },
}

pub struct UninitializedTurtleReceiver {
    id: u64,
    name: Arc<str>,
    connection: SplitStream<ws::WebSocket>,
    outstanding_requests: HashMap<u64, oneshot::Sender<serde_json::Value>>,
    sender: ActorRef<TurtleSender>,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    next_id: u64,
    db: DatabaseConnection,
}

pub struct TurtleReceiver {
    id: u64,
    name: Arc<str>,
    connection: StreamHandle,
    outstanding_requests: HashMap<u64, oneshot::Sender<serde_json::Value>>,
    sender: ActorRef<TurtleSender>,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    next_id: u64,
    db: DatabaseConnection,
}

impl TurtleReceiver {
    pub fn new(
        id: u64,
        name: Arc<str>,
        sender: ActorRef<TurtleSender>,
        connection: SplitStream<ws::WebSocket>,
        pub_sub: ActorRef<PubSub<TurtleNotification>>,
        db: DatabaseConnection,
    ) -> UninitializedTurtleReceiver {
        UninitializedTurtleReceiver {
            name,
            id,
            connection,
            outstanding_requests: HashMap::new(),
            sender,
            pub_sub,
            next_id: 0,
            db,
        }
    }

    async fn handle_message(&mut self, message: ws::Message) {
        match message {
            ws::Message::Text(text) => {
                debug!("Received message from {}: {}", self.name, text.as_str());

                let Ok(Some(entity)) = turtle_entities::turtle::Entity::find_by_id(self.id as i32)
                    .one(&self.db)
                    .await
                else {
                    error!(
                        "Unable to find {} in database with id {}",
                        &self.name, self.id
                    );
                    return;
                };

                let mut active_model = entity.into_active_model();

                active_model.last_seen = Set(chrono::Utc::now());

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
                        match &info {
                            TurtleInformation::Report {
                                fuel,
                                heading,
                                position,
                                inventory,
                            } => {
                                active_model.fuel = Set(fuel.level as i32);
                                active_model.heading = Set(*heading);
                                active_model.x = Set(position.x as i32);
                                active_model.y = Set(position.y as i32);
                                active_model.z = Set(position.z as i32);
                                active_model.inventory = Set(inventory.clone());
                            }
                        }

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

                if let Err(e) = active_model.update(&self.db).await {
                    error!("Problem updating db {e}");
                }
            }
            _ => warn!("Got invalid message from {}", self.name),
        }
    }
}

impl Actor for TurtleReceiver {
    type Args = UninitializedTurtleReceiver;
    type Error = kameo::error::Infallible;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let UninitializedTurtleReceiver {
            id,
            name,
            connection,
            outstanding_requests,
            sender,
            pub_sub,
            next_id,
            db,
        } = args;

        let connection = actor_ref.attach_stream(connection, (), ());

        Ok(TurtleReceiver {
            id,
            name,
            connection,
            outstanding_requests,
            sender,
            pub_sub,
            next_id,
            db,
        })
    }

    async fn on_stop(
        &mut self,
        actor_ref: WeakActorRef<Self>,
        reason: ActorStopReason,
    ) -> Result<(), Self::Error> {
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
        ctx: &mut Context<Self, Self::Reply>,
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
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let id = self.next_id;
        self.next_id += 1;
        self.outstanding_requests.insert(id, tx);

        id
    }
}
