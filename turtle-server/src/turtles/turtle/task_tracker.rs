use std::{collections::HashSet, sync::Arc};

use kameo::{Actor, Reply, messages, prelude::Context};
use tokio::task::AbortHandle;
use tracing::{debug, info, warn};

use crate::turtles::turtle::task::TaskName;

#[derive(Debug, Actor)]
pub struct TaskTracker {
    turtle_name: Arc<str>,
    next_id: u64,
    tasks: Vec<RunningTask>,
    pending_completion: HashSet<u64>,
    handle: AbortHandle,
}

impl TaskTracker {
    pub fn new(turtle_name: Arc<str>, handle: AbortHandle) -> Self {
        TaskTracker {
            turtle_name,
            next_id: 0,
            tasks: Vec::new(),
            pending_completion: HashSet::new(),
            handle,
        }
    }

    fn complete(&mut self, id: u64) {
        if self.pending_completion.remove(&id) {
            return;
        }

        let Some(index) = self.tasks.iter().rposition(|t| t.id == id) else {
            warn!("{} got a completion for an unkown task", self.turtle_name);
            return;
        };

        for completed in self.tasks.drain(index..) {
            debug!("{} completed task {}", self.turtle_name, completed.name);

            if completed.id != id {
                self.pending_completion.insert(completed.id);
            }
        }
    }
}

#[messages]
impl TaskTracker {
    #[message]
    pub fn register_task(&mut self, name: TaskName) -> u64 {
        info!("Registering new task {name}");
        let id = self.next_id;
        self.next_id += 1;

        let task = RunningTask { id, name };
        self.tasks.push(task);

        id
    }

    #[message(ctx)]
    pub fn notify_completed(&mut self, id: u64, ctx: &mut Context<Self, ()>) {
        info!(
            "Completing task {:?} Pending: {:?}",
            self.tasks, self.pending_completion
        );
        self.complete(id);
        if self.tasks.is_empty() && self.pending_completion.is_empty() {
            ctx.stop();
        }
    }

    #[message]
    pub fn running_tasks(&self) -> TaskList {
        TaskList(Box::from_iter(self.tasks.iter().map(|t| (t.id, t.name))))
    }

    #[message]
    pub fn cancel(&mut self) {
        self.handle.abort();
    }
}

#[derive(Debug)]
struct RunningTask {
    id: u64,
    name: TaskName,
}

#[derive(Debug, Reply, Clone)]
pub struct TaskList(Box<[(u64, TaskName)]>);

impl AsRef<[(u64, TaskName)]> for TaskList {
    fn as_ref(&self) -> &[(u64, TaskName)] {
        &self.0
    }
}

impl std::fmt::Display for TaskList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        for tasks in self.0.windows(2) {
            let (_, name) = tasks[0];
            write!(f, "{name}/")?;
        }

        let (_, last) = self
            .0
            .last()
            .expect("Checked that task list is not empty before");

        write!(f, "{last}")
    }
}
