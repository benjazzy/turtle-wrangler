use std::sync::Arc;

use kameo::{messages, prelude::Context, Actor};
use tracing::error;

#[derive(Debug, Actor, Default)]
pub struct TaskTracker {
    task_names: Vec<Arc<str>>,
}

impl TaskTracker {
    pub fn new() -> Self {
        let task_names = Vec::new();
        Self { task_names }
    }
}

#[messages]
impl TaskTracker {
    #[message]
    pub async fn register_task(&mut self, task_name: Arc<str>) {
        self.task_names.push(task_name);
    }

    #[message(ctx)]
    pub async fn notify_completed(&mut self, task_name: Arc<str>, ctx: &mut Context<Self, ()>) {
        match self.task_names.pop() {
            Some(completed) if completed != task_name => {
                error!("Task tracker got completion notification for incorrect task. Notified {task_name}, Completed {completed}");
            }
            Some(_) => {}
            None => {
                ctx.stop();
            }
        }
    }

    #[message(ctx)]
    pub async fn cancel(&mut self, ctx: &mut Context<Self, ()>) {
        ctx.stop();
    }
}

struct RunningTask;
