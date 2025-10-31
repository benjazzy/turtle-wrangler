use kameo::actor::ActorRef;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{debug, error};
use turtle_sender::{LockSender, UnlockSender};
use turtle_types::turtle_scheme::turtle_messages::{Command, Query, TurtleInformation};

mod turtle_receiver;
mod turtle_sender;

use turtle_receiver::RegisterRequest;
pub use turtle_receiver::TurtleReceiver;
pub use turtle_sender::{SendCommand, TurtleSender};

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

#[derive(Debug, Clone)]
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

    async fn request<C>(&self, request: C) -> Result<C::Response, ()>
    where
        C: Command + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let id = if let Ok(id) = self.receiver.ask(RegisterRequest(tx)).await {
            id
        } else {
            self.close();
            return Err(());
        };

        if self.sender.tell(SendCommand(id, request)).await.is_err() {
            self.close();
            return Err(());
        }

        let message = rx.await.map_err(|_| ())?;

        debug!("Got message {message}");

        serde_json::from_value(message.clone()).map_err(|e|{
            error!("Problem deserializing response {e}: {message}");

            ()
        })
    }

    pub async fn query<Q>(&self, query: Q) -> Result<Q::Response, ()>
    where
        Q: Query + Send + 'static,
    {
        self.request(query).await
    }

    pub async fn lock(&self) -> LockedTurtle {
        debug!("Trying to lock {}", self.name);
        self.sender.ask(LockSender).await;
        debug!("{} is done locking", self.name);
        LockedTurtle(self.clone())
    }
}

pub struct LockedTurtle(Turtle);

impl LockedTurtle {
    pub async fn command<C>(&self, command: C) -> Result<C::Response, ()>
    where
        C: Command + Send + 'static,
    {
        self.0.request(command).await
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
