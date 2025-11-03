use crate::turtles::{TurtleManager, TurtleNotification};
use kameo::actor::ActorRef;
use kameo_actors::pubsub::PubSub;
use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct TurtleState {
    pub database: DatabaseConnection,
    pub turtle_manager: ActorRef<TurtleManager>,
    pub pub_sub: ActorRef<PubSub<TurtleNotification>>,
}
