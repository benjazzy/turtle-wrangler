use std::{collections::VecDeque, sync::Arc};

use actix::{Actor, ActorContext, Addr, Context, Handler, Message, ResponseFuture};

use actix_web::Result;

use serde::Serialize;

use thiserror::Error;
use tokio::sync::oneshot;
use tracing::error;

use crate::{
    turtle::turtle_connection::{self, TurtleConnection},
    turtle_scheme::Command,
};

#[derive(Debug, Clone, Copy, Error)]
pub enum StateError {
    #[error("Got wrong id with ok message")]
    WrongOkId,
    #[error("Got send when waiting for ok")]
    SendWhenWaitingForOk,
    #[error("Got send when waiting for ready")]
    SendWhenWaitingForReady,
    #[error("Got ok when ready")]
    OkWhenReady,
    #[error("Got ok when waiting for ready")]
    OkWhenWaitingForReady,
    #[error("Got ready when already ready")]
    ReadyWhenReady,
    #[error("Got ready when ok")]
    ReadyWhenOk,
}

#[derive(Debug, Error)]
pub enum TurtleSendError {
    #[error("Problem serializing message {0}")]
    SerializeError(serde_json::Error),
    #[error("Turtle connection has closed")]
    ConnectionClosed,
    #[error("Attempt to send while turtle was in an invalid state {0}")]
    StateError(StateError),
}

impl From<serde_json::Error> for TurtleSendError {
    fn from(value: serde_json::Error) -> Self {
        TurtleSendError::SerializeError(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SenderState {
    Ready,
    WaitingForOk(u64),
    WaitingForReady,
}

impl SenderState {
    pub fn ok(self, id: u64) -> Result<Self, StateError> {
        match self {
            SenderState::WaitingForOk(old_id) if old_id == id => Ok(SenderState::WaitingForReady),
            SenderState::WaitingForOk(_) => Err(StateError::WrongOkId),
            SenderState::WaitingForReady => Err(StateError::OkWhenWaitingForReady),
            SenderState::Ready => Err(StateError::OkWhenReady),
        }
    }

    pub fn ready(self) -> Result<Self, StateError> {
        match self {
            SenderState::WaitingForReady => Ok(SenderState::Ready),
            SenderState::Ready => Err(StateError::ReadyWhenReady),
            SenderState::WaitingForOk(_) => Err(StateError::ReadyWhenOk),
        }
    }

    pub fn send(self, id: u64) -> Result<Self, StateError> {
        match self {
            SenderState::Ready => Ok(SenderState::WaitingForOk(id)),
            SenderState::WaitingForOk(_) => Err(StateError::SendWhenWaitingForOk),
            SenderState::WaitingForReady => Err(StateError::SendWhenWaitingForReady),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LockState {
    Unlocked(SenderState),
    Locked(SenderState),
}

#[derive(Debug, Serialize)]
pub struct TurtleMessage {
    pub id: u64,
    pub message: serde_json::Value,
}

#[derive(Debug)]
pub struct TurtleSenderActor {
    connection: Addr<TurtleConnection>,
    state: LockState,
    lock_queue: VecDeque<oneshot::Sender<()>>,
    message_queue: VecDeque<TurtleMessage>,
    name: Arc<str>,
}

impl TurtleSenderActor {
    pub fn new(connection: Addr<TurtleConnection>, name: Arc<str>) -> Self {
        TurtleSenderActor {
            connection,
            state: LockState::Unlocked(SenderState::WaitingForReady),
            lock_queue: VecDeque::new(),
            message_queue: VecDeque::new(),
            name,
        }
    }

    pub fn send(&mut self, msg: TurtleMessage) -> Result<(), TurtleSendError> {
        match &mut self.state {
            LockState::Unlocked(state @ SenderState::Ready)
            | LockState::Locked(state @ SenderState::Ready) => {
                *state = state.send(msg.id).map_err(TurtleSendError::StateError)?;

                self.connection
                    .try_send(turtle_connection::SendMessage(
                        serde_json::to_string(&msg).map_err(TurtleSendError::SerializeError)?,
                    ))
                    .map_err(|_| TurtleSendError::ConnectionClosed)?;
            }
            LockState::Unlocked(_) | LockState::Locked(_) => {
                self.message_queue.push_back(msg);
            }
        };

        Ok(())
    }

    pub fn ok(&mut self, id: u64) -> Result<(), StateError> {
        match &mut self.state {
            LockState::Unlocked(sender_state) | LockState::Locked(sender_state) => {
                *sender_state = sender_state.ok(id)?
            }
        };

        Ok(())
    }

    pub fn ready(&mut self) -> Result<(), TurtleSendError> {
        match &mut self.state {
            LockState::Unlocked(sender_state) | LockState::Locked(sender_state) => {
                *sender_state = sender_state.ready().map_err(TurtleSendError::StateError)?;
                if let Some(msg) = self.message_queue.pop_front() {
                    self.send(msg)?;
                }
            }
        };

        self.try_lock();

        Ok(())
    }

    pub fn lock(&mut self, tx: oneshot::Sender<()>) {
        self.lock_queue.push_back(tx);
        self.try_lock();
    }

    pub fn try_lock(&mut self) {
        match self.state {
            LockState::Unlocked(SenderState::Ready) if !self.lock_queue.is_empty() => {
                let tx = self
                    .lock_queue
                    .pop_front()
                    .expect("Checked that send queue is not empty earlier");
                if tx.send(()).is_ok() {
                    self.state = LockState::Locked(SenderState::WaitingForReady);
                }
            }
            _ => {}
        }
    }

    pub fn unlock(&mut self) {
        match self.state {
            LockState::Locked(state) => self.state = LockState::Unlocked(state),
            LockState::Unlocked(_) => error!(
                "Unlock called for {} while sender was already unlocked",
                self.name
            ),
        }
    }
}

impl Actor for TurtleSenderActor {
    type Context = Context<Self>;
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), TurtleSendError>")]
pub struct SendCommand<C>(pub C, pub u64);

impl<C> Handler<SendCommand<C>> for TurtleSenderActor
where
    C: Command + Serialize + 'static,
{
    type Result = Result<(), TurtleSendError>;

    fn handle(&mut self, msg: SendCommand<C>, _ctx: &mut Self::Context) -> Self::Result {
        let msg = TurtleMessage {
            message: serde_json::to_value(&msg.0).map_err(TurtleSendError::SerializeError)?,
            id: msg.1,
        };
        self.send(msg)
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), oneshot::error::RecvError>")]
pub struct Lock;

impl Handler<Lock> for TurtleSenderActor {
    type Result = ResponseFuture<Result<(), oneshot::error::RecvError>>;

    fn handle(&mut self, _msg: Lock, _ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();
        self.lock(tx);

        Box::pin(rx)
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Unlock;

impl Handler<Unlock> for TurtleSenderActor {
    type Result = ();

    fn handle(&mut self, _msg: Unlock, _ctx: &mut Self::Context) -> Self::Result {
        self.unlock();
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetOk(pub u64);

impl Handler<SetOk> for TurtleSenderActor {
    type Result = ();

    fn handle(&mut self, msg: SetOk, _ctx: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.ok(msg.0) {
            error!("{} ok error {e}", self.name);
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetReady;

impl Handler<SetReady> for TurtleSenderActor {
    type Result = ();

    fn handle(&mut self, _msg: SetReady, _ctx: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.ready() {
            error!("{} ready error {e}", self.name);
        }
    }
}

impl Handler<super::Close> for TurtleSenderActor {
    type Result = ();

    fn handle(&mut self, _msg: super::Close, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
