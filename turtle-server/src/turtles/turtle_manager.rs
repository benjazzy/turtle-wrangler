use crate::turtles::turtle::{Turtle, TurtleNote, TurtleNotification, TurtleWarning};
use kameo::actor::pubsub::{PubSub, Subscribe};
use kameo::actor::ActorRef;
use kameo::error::BoxError;
use kameo::mailbox::unbounded::UnboundedMailbox;
use kameo::mailbox::Mailbox;
use kameo::message::{Context, Message};
use kameo::{messages, Actor};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tracing::{debug, info, warn};
use turtle_types::turtle_scheme::TurtleStatus;
use turtle_types::{client_views, turtle_scheme};

pub struct TurtleManager {
    turtles: HashMap<Arc<str>, Turtle>,
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
}

impl TurtleManager {
    pub fn new(pub_sub: ActorRef<PubSub<TurtleNotification>>) -> Self {
        TurtleManager {
            turtles: HashMap::new(),
            pub_sub,
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
    type Mailbox = UnboundedMailbox<Self>;

    async fn on_start(&mut self, actor_ref: ActorRef<Self>) -> Result<(), BoxError> {
        self.pub_sub.tell(Subscribe(actor_ref)).await;

        Ok(())
    }
}

impl Message<TurtleNotification> for TurtleManager {
    type Reply = ();

    async fn handle(
        &mut self,
        notification: TurtleNotification,
        ctx: Context<'_, Self, Self::Reply>,
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
    type Reply = Vec<client_views::TurtleReport>;

    async fn handle(
        &mut self,
        _: GetConnectedTurtles,
        _: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        self.turtles
            .values()
            .map(|t| client_views::TurtleReport {
                name: t.name().as_ref().into(),
                status: TurtleStatus::Connected,
                coordinates: turtle_scheme::Coordinates { x: 0, y: 0, z: 0 },
                heading: turtle_scheme::Heading::North,
                turtle_type: turtle_scheme::TurtleType::Normal,
                fuel: turtle_scheme::Fuel { level: 0, max: 0 },
            })
            .collect()
    }
}
