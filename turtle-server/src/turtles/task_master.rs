use std::{collections::HashMap, sync::Arc};

use kameo::{Actor, actor::ActorRef, messages, prelude::Message};
use tracing::info;

use crate::turtles::{TurtleNotification, task::TaskName, turtle::TurtleNote};

use super::turtle::task_tracker::TaskTracker;

#[derive(Debug, Default, Actor)]
pub struct TaskMaster {
    running_trackers: HashMap<Arc<str>, ActorRef<TaskTracker>>,
}

#[messages]
impl TaskMaster {
    #[message]
    pub async fn get_tracker(&self, turtle_name: Arc<str>) -> Option<ActorRef<TaskTracker>> {
        self.running_trackers.get(&turtle_name).cloned()
    }
}

impl Message<TurtleNotification> for TaskMaster {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: TurtleNotification,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        #[allow(clippy::collapsible_if)]
        if let TurtleNotification::Note(TurtleNote::TaskStarted(turtle_name, tracker)) = msg {
            if self
                .running_trackers
                .insert(turtle_name.clone(), tracker)
                .is_some()
            {
                tracing::error!(
                    "{turtle_name} tried to register a new tracker while one was still pending. Dropping old tracker."
                );
            }
            info!("Registered new task for {}", turtle_name);
        }
    }
}
