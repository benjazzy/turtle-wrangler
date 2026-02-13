use kameo::{
    Reply,
    actor::{ActorRef, Spawn},
};
use kameo_actors::pubsub::{PubSub, Publish};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use thiserror::Error;
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::{debug, error, info, warn};
use turtle_sender::{LockSender, UnlockSender};
use turtle_types::turtle_scheme::{
    Coordinates, Position,
    turtle_messages::{Command, Query, TurtleInformation},
};

pub mod task;
pub mod task_tracker;
mod turtle_receiver;
mod turtle_sender;

use task_tracker::*;
use turtle_receiver::RegisterRequest;
pub use turtle_receiver::TurtleReceiver;
pub use turtle_sender::{SendCommand, TurtleSender};

use crate::turtles::{
    task_master::{self, TaskMaster},
    turtle::task::TurtleTask,
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

    #[error("Problem reserializing response {0}: {1}")]
    DeserializeError(serde_json::error::Error, String),
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
    TaskStarted(Arc<str>, ActorRef<TaskTracker>),
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
    pub_sub: ActorRef<PubSub<TurtleNotification>>,
}

impl Turtle {
    pub fn new(
        name: Arc<str>,
        sender: ActorRef<TurtleSender>,
        receiver: ActorRef<TurtleReceiver>,
        pub_sub: ActorRef<PubSub<TurtleNotification>>,
    ) -> Turtle {
        Turtle {
            name,
            sender,
            receiver,
            pub_sub,
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

        serde_json::from_value(message.clone())
            .map_err(|e| TurtleRequestError::DeserializeError(e, message.to_string()))
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

    pub async fn start_task<T>(self, task: T) -> Result<T::Return, tokio::task::JoinError>
    where
        T: TurtleTask + Send + 'static,
        T::Return: Send,
    {
        // Sigh
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskyTurtle>();
        let handle = tokio::spawn(async move {
            let Ok(turtle) = rx.await else {
                panic!("Turtle must be sent before the sender is dropped")
            };

            turtle.run_task(task).await
        });

        let tracker =
            TaskTracker::spawn(TaskTracker::new(self.0.name.clone(), handle.abort_handle()));
        self.0
            .pub_sub
            .tell(Publish(TurtleNotification::Note(TurtleNote::TaskStarted(
                self.0.name.clone(),
                tracker.clone(),
            ))))
            .await;

        let tasky_turtle = TaskyTurtle {
            turtle: self,
            tracker,
        };
        tx.send(tasky_turtle);

        handle.await
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
        let name = task.task_name().into();
        let id_result = self.tracker.ask(RegisterTask { name }).send().await;

        let result = task.execute(self).await;

        match id_result {
            Ok(id) => {
                self.tracker.tell(NotifyCompleted { id }).await;
            }
            Err(e) => {
                error!("{} ran a task without a tracker: {e}", self.turtle.0.name);
            }
        }

        result
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
