use crate::turtles::TurtleManager;
use kameo::actor::ActorRef;
use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct TurtleState {
    pub database: DatabaseConnection,
    pub turtle_manager: ActorRef<TurtleManager>,
}
