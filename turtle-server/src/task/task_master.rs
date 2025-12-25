use std::collections::HashMap;

use kameo::prelude::*;
use tokio::task::JoinHandle;
use tracing::error;

use crate::{task::TurtleTask, turtles::Queryable};

fn run<T, Q>(
    master: ActorRef<TaskMaster>,
    id: u64,
    turtle: Q,
    task: T,
    sender: Option<ReplySender<T::Return>>,
) -> JoinHandle<()>
where
    T: TurtleTask<Turtle = Q> + Send + 'static,
    <T as TurtleTask>::Return: Send + Reply,
    Q: Queryable + Send + 'static,
{
    tokio::spawn(async move {
        let result = task.execute(turtle, &master).await;
        if let Err(e) = master.tell(CompleteTask { id }).await {
            error!("Problem informing Task Master of task {id}'s completion {e}",);
        }
        if let Some(sender) = sender {
            sender.send(result);
        }
    })
}

#[derive(Debug, Default, Actor)]
pub struct TaskMaster {
    task_handles: HashMap<u64, JoinHandle<()>>,
    next_id: u64,
}

#[messages]
impl TaskMaster {
    #[message]
    pub fn complete_task(&mut self, id: u64) {
        let Some(handle) = self.task_handles.remove(&id) else {
            error!("Task Master got complete task message for unknown id {id}");
            return;
        };

        handle.abort();
    }
}

pub struct RunTask<Task, Q>(pub Task, pub Q);

impl<Task, Q> Message<RunTask<Task, Q>> for TaskMaster
where
    Task: TurtleTask<Turtle = Q> + Send + 'static,
    <Task as TurtleTask>::Return: kameo::Reply<Value = <Task as TurtleTask>::Return>,
    Q: Queryable + Send + 'static,
{
    type Reply = DelegatedReply<Task::Return>;

    async fn handle(
        &mut self,
        RunTask(task, turtle): RunTask<Task, Q>,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let id = self.next_id;
        self.next_id += 1;

        let (delegated_reply, reply_sender) = ctx.reply_sender();
        let handle = run(ctx.actor_ref().clone(), id, turtle, task, reply_sender);
        self.task_handles.insert(id, handle);

        delegated_reply
    }
}
