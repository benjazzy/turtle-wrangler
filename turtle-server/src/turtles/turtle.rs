use kameo::{
    Reply,
    actor::{ActorRef, Spawn},
};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::{debug, error, warn};
use turtle_sender::{LockSender, UnlockSender};
use turtle_types::turtle_scheme::turtle_messages::{Command, Query, TurtleInformation};

mod task;
mod task_tracker;
mod turtle_receiver;
mod turtle_sender;

use turtle_receiver::RegisterRequest;
pub use turtle_receiver::TurtleReceiver;
pub use turtle_sender::{SendCommand, TurtleSender};

use crate::turtles::turtle::{
    task::TurtleTask,
    task_tracker::{RegisterTask, TaskTracker},
};

pub trait Queryable {
    async fn query<Q>(&self, query: Q) -> Result<Q::Response, TurtleRequestError>
    where
        Q: Query + Send + 'static;
}

#[derive(Debug, Error)]
pub enum TurtleRequestError {
    #[error("Internal error communicating with the turtle {0}")]
    InternalError(String),

    #[error("Problem reserializing response {0}")]
    DeserializeError(#[from] serde_json::error::Error),
}

impl<M, E> From<kameo::error::SendError<M, E>> for TurtleRequestError
where
    E: std::fmt::Display,
{
    fn from(value: kameo::error::SendError<M, E>) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<oneshot::error::RecvError> for TurtleRequestError {
    fn from(value: oneshot::error::RecvError) -> Self {
        Self::InternalError(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum TurtleNotification {
    Note(TurtleNote),
    Warning(TurtleWarning),
}

#[derive(Debug, Clone)]
pub enum TurtleNote {
    TurtleConnected(Turtle),
    TurtleInfo {
        name: Arc<str>,
        info: TurtleInformation,
    },
}
#[derive(Debug, Clone)]
pub enum TurtleWarning {
    TurtleDisconnected(Arc<str>),
}

#[derive(Debug, Clone, Reply)]
pub struct Turtle {
    name: Arc<str>,
    sender: ActorRef<TurtleSender>,
    receiver: ActorRef<TurtleReceiver>,
}

impl Turtle {
    pub fn new(
        name: Arc<str>,
        sender: ActorRef<TurtleSender>,
        receiver: ActorRef<TurtleReceiver>,
    ) -> Turtle {
        Turtle {
            name,
            sender,
            receiver,
        }
    }

    pub fn name(&self) -> &Arc<str> {
        &self.name
    }

    pub fn close(&self) {
        self.sender.kill();
        self.receiver.kill();
    }

    async fn request<C>(&self, request: C) -> Result<C::Response, TurtleRequestError>
    where
        C: Command + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let id = self.receiver.ask(RegisterRequest(tx)).await?;

        self.sender.tell(SendCommand(id, request)).await?;

        let message = rx.await?;

        debug!("Got message {message}");

        serde_json::from_value(message.clone()).map_err(Into::into)
    }

    pub async fn query<Q>(&self, query: Q) -> Result<Q::Response, TurtleRequestError>
    where
        Q: Query + Send + 'static,
    {
        let result = self.request(query).await;

        if let Err(TurtleRequestError::InternalError(e)) = &result {
            error!(
                "Actor for {} had an error while sending a query. Closing the connection: {e}",
                self.name
            );
            self.close();
        }

        result
    }

    pub async fn lock(&self) -> LockedTurtle {
        debug!("Trying to lock {}", self.name);
        self.sender.ask(LockSender).await;
        debug!("{} is done locking", self.name);
        LockedTurtle(self.clone())
    }
}

impl Queryable for Turtle {
    async fn query<Q>(&self, query: Q) -> Result<Q::Response, TurtleRequestError>
    where
        Q: Query + Send + 'static,
    {
        Turtle::query(self, query).await
    }
}

#[derive(Debug, Reply)]
pub struct LockedTurtle(Turtle);

impl LockedTurtle {
    pub async fn command<C>(&self, command: C) -> Result<C::Response, TurtleRequestError>
    where
        C: Command + Send + 'static,
    {
        let result = self.0.request(command).await;

        if let Err(TurtleRequestError::InternalError(e)) = &result {
            error!(
                "Actor for {} had an error while sending a command. Closing the connection: {e}",
                self.0.name
            );
            self.0.close();
        }

        result
    }

    pub async fn start_task<T>(self, task: T) -> T::Return
    where
        T: TurtleTask,
    {
        let tracker = TaskTracker::spawn_default();
        let tasky_turtle = TaskyTurtle {
            turtle: self,
            tracker,
        };

        task.execute(&tasky_turtle).await
    }
}

impl Queryable for LockedTurtle {
    async fn query<Q>(&self, query: Q) -> Result<Q::Response, TurtleRequestError>
    where
        Q: Query + Send + 'static,
    {
        LockedTurtle::command(&self, query).await
    }
}

impl Drop for LockedTurtle {
    fn drop(&mut self) {
        debug!("Unlocking {}", self.0.name);
        let sender = self.0.sender.clone();
        tokio::spawn(async move {
            sender.tell(UnlockSender).await;
        });
    }
}

pub struct TaskyTurtle {
    turtle: LockedTurtle,
    tracker: ActorRef<TaskTracker>,
}

impl TaskyTurtle {
    pub async fn run_task<T>(&self, task: T) -> T::Return
    where
        T: TurtleTask,
    {
        self.tracker
            .tell(RegisterTask {
                task_name: task.task_name().into(),
            })
            .await;
        task.execute(self).await
    }

    pub fn get_name(&self) -> &str {
        &self.turtle.0.name
    }

    pub async fn command<C>(&self, command: C) -> Result<C::Response, TurtleRequestError>
    where
        C: Command + Send + 'static,
    {
        let result = self.turtle.0.request(command).await;

        if let Err(TurtleRequestError::InternalError(e)) = &result {
            error!(
                "Actor for {} had an error while sending a command. Closing the connection: {e}",
                self.turtle.0.name
            );
            self.turtle.0.close();
        }

        result
    }
}
