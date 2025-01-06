pub mod turtle_connection;
pub mod turtle_identifier;
mod turtle_receiver;
mod turtle_sender;
mod unknown_turtle_connection;

use actix::prelude::*;
use serde::Serialize;
use tokio::sync::oneshot;
use turtle_receiver::ResponseListener;
use turtle_sender::{Lock, SendCommand, TurtleSendError};

use crate::turtle_scheme::{Command, Query};

use self::turtle_receiver::TurtleReceiver;
use self::turtle_sender::TurtleSenderActor;

#[derive(Debug, thiserror::Error)]
#[error("Turtle connection closed before it could be locked")]
struct LockError;

impl From<MailboxError> for LockError {
    fn from(_value: MailboxError) -> Self {
        LockError
    }
}

impl From<oneshot::error::RecvError> for LockError {
    fn from(_value: oneshot::error::RecvError) -> Self {
        LockError
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Problem sending message to turtle {0}")]
    SendError(TurtleSendError),
    #[error("Unable to deserialize into response type {0}")]
    DeserializeError(serde_json::Error),
    #[error("Turtle connection closed while waiting for response")]
    ConnectionClosed,
}

impl From<oneshot::error::RecvError> for RequestError {
    fn from(_: oneshot::error::RecvError) -> Self {
        RequestError::ConnectionClosed
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(value: serde_json::Error) -> Self {
        RequestError::DeserializeError(value)
    }
}

impl From<TurtleSendError> for RequestError {
    fn from(value: TurtleSendError) -> Self {
        RequestError::SendError(value)
    }
}

impl From<actix::MailboxError> for RequestError {
    fn from(_value: actix::MailboxError) -> Self {
        RequestError::ConnectionClosed
    }
}

#[derive(Clone)]
pub struct Turtle {
    sender: Addr<TurtleSenderActor>,
    receiver: Addr<TurtleReceiver>,
    name: String,
}

impl Turtle {
    pub fn new(
        sender: Addr<TurtleSenderActor>,
        receiver: Addr<TurtleReceiver>,
        name: String,
    ) -> Self {
        Turtle {
            sender,
            receiver,
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub async fn query<Q>(&self, query: Q) -> Result<Q::Response, RequestError>
    where
        Q: Query + Serialize + Send + 'static,
    {
        self.request(query).await
    }

    async fn request<R>(&self, request: R) -> Result<R::Response, RequestError>
    where
        R: Command + Serialize + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let id = self.receiver.send(ResponseListener(tx)).await?;
        self.sender.send(SendCommand(request, id)).await?;

        Ok(serde_json::from_value(rx.await?)?)
    }

    pub async fn lock(&self) -> Result<TurtleLock, LockError> {
        self.sender.send(Lock).await??;

        Ok(TurtleLock(self.clone()))
    }

    pub fn close(&self) {
        self.sender.do_send(Close);
        self.receiver.do_send(Close);
    }
}

pub struct TurtleLock(Turtle);

impl TurtleLock {
    pub async fn command<C>(&self, command: C) -> Result<C::Response, RequestError>
    where
        C: Command + Serialize + Send + 'static,
    {
        self.0.request(command).await
    }
}

impl Drop for TurtleLock {
    fn drop(&mut self) {
        todo!()
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Close;
