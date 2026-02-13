use crate::turtles::{TaskMaster, TurtleManager, TurtleNotification};
use axum::extract::FromRef;
use kameo::actor::ActorRef;
use kameo_actors::pubsub::PubSub;
use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct TurtleState {
    pub database: DatabaseConnection,
    pub turtle_manager: ActorRef<TurtleManager>,
    pub task_master: ActorRef<TaskMaster>,
    pub pub_sub: ActorRef<PubSub<TurtleNotification>>,
}
