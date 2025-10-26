use crate::entities;
use crate::turtles::turtle::{Turtle, TurtleNote, TurtleNotification, TurtleWarning};
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use kameo::{messages, Actor};
use kameo_actors::pubsub::{PubSub, Subscribe};
use migration::IntoIden;
use sea_orm::prelude::{DateTime, DateTimeUtc, Uuid};
use sea_orm::ActiveValue::{self, Set};
use sea_orm::{
    sea_query, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityOrSelect, EntityTrait,
    IntoActiveModel, QueryFilter,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use turtle_types::turtle_scheme::{Heading, TurtleStatus, TurtleType};
use turtle_types::{client_views, turtle_scheme};

pub struct TurtleManager {
    turtles: HashMap<Arc<str>, Turtle>,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
    db: DatabaseConnection,
}

impl TurtleManager {
    pub fn new(pub_sub: ActorRef<PubSub<TurtleNotification>>, db: DatabaseConnection) -> Self {
        TurtleManager {
            turtles: HashMap::new(),
            pub_sub,
            db,
        }
    }
}

#[messages]
impl TurtleManager {
    #[message]
    pub async fn get_turtle(&mut self, name: Arc<str>) -> Option<Turtle> {
        self.turtles.get(&name).cloned()
    }
}

impl Actor for TurtleManager {
    type Error = kameo::error::Infallible;
    type Args = Self;

    async fn on_start(state: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        state.pub_sub.tell(Subscribe(actor_ref)).await;

        Ok(state)
    }
}

impl Message<TurtleNotification> for TurtleManager {
    type Reply = ();

    async fn handle(
        &mut self,
        notification: TurtleNotification,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match notification {
            TurtleNotification::Note(TurtleNote::TurtleConnected(turtle)) => {
                info!("New connection from {}", turtle.name());

                if let Some(old_turtle) = self.turtles.insert(turtle.name().clone(), turtle) {
                    old_turtle.close();
                }
            }
            TurtleNotification::Warning(TurtleWarning::TurtleDisconnected(name)) => {
                warn!("{name}'s connection was closed");
                if let Some(turtle) = self.turtles.remove(&name) {
                    turtle.close();
                }
            }
            TurtleNotification::Note(TurtleNote::TurtleInfo { name, info }) => {
                debug!("Got info from {name}: {info:?}");
            }
            _ => {}
        }
    }
}

pub struct GetConnectedTurtles;

impl Message<GetConnectedTurtles> for TurtleManager {
    type Reply = Result<Vec<client_views::TurtleReport>, sea_orm::DbErr>;

    async fn handle(
        &mut self,
        _: GetConnectedTurtles,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let turtles = entities::turtles::Entity::find().all(&self.db).await?;

        let reports = turtles
            .into_iter()
            .map(|t| {
                let status = if self.turtles.contains_key(t.name.as_str()) {
                    TurtleStatus::Connected
                } else {
                    TurtleStatus::Disconnected
                };

                client_views::TurtleReport {
                    name: t.name.into(),
                    status,
                    coordinates: turtle_scheme::Coordinates {
                        x: t.x as i64,
                        y: t.y as i64,
                        z: t.z as i64,
                    },
                    heading: t.heading,
                    turtle_type: t.turtle_type,
                    fuel: turtle_scheme::Fuel {
                        level: t.fuel as u32,
                        max: 0,
                    },
                }
            })
            .collect::<Vec<_>>();

        Ok(reports)
    }
}
