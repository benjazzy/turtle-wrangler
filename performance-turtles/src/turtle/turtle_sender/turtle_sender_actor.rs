use std::{collections::VecDeque, rc::Rc};

use actix::{
    dev::SendError, Actor, ActorFuture, Addr, AsyncContext, Context, Handler, Message,
    ResponseActFuture, ResponseFuture, WrapFuture,
};

use actix_web::Result;

use serde::Serialize;

use thiserror::Error;
use tokio::sync::oneshot;

use crate::{
    turtle::turtle_connection::{self, TurtleConnection},
    turtle_scheme::{Command, Ping},
};

#[derive(Debug, Clone, Copy)]
pub struct StateError(LockState, StateErrorType);

#[derive(Debug, Clone, Copy, Error)]
pub enum StateErrorType {
    #[error("Got wrong id with ok message")]
    GotWrongOkId,
    #[error("Got send when waiting for ok")]
    GotSendWhenWaitingForOk,
    #[error("Got send when waiting for ready")]
    GotSendWhenWaitingForReady,
    #[error("Got ok when ready")]
    GotOkWhenReady,
    #[error("Got ok when waiting for ready")]
    GotOkWhenWaitingForReady,
    #[error("Got ready when already ready")]
    GotReadyWhenReady,
    #[error("Got ready when ok")]
    GotReadyWhenOk,
}

#[derive(Debug, Error)]
pub enum TurtleSendError {
    #[error("Problem serializing message {0}")]
    SerializeError(serde_json::Error),
    #[error("Turtle connection has closed")]
    ConnectionClosed,
    #[error("Attempt to send while turtle was in an invalid state {0}")]
    StateError(StateErrorType),
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
    pub fn ok(self, id: u64) -> Result<Self, StateErrorType> {
        match self {
            SenderState::WaitingForOk(old_id) if old_id == id => Ok(SenderState::WaitingForReady),
            SenderState::WaitingForOk(_) => Err(StateErrorType::GotWrongOkId),
            SenderState::WaitingForReady => Err(StateErrorType::GotOkWhenWaitingForReady),
            SenderState::Ready => Err(StateErrorType::GotOkWhenReady),
        }
    }

    pub fn ready(self) -> Result<Self, StateErrorType> {
        match self {
            SenderState::WaitingForReady => Ok(SenderState::Ready),
            SenderState::Ready => Err(StateErrorType::GotReadyWhenReady),
            SenderState::WaitingForOk(_) => Err(StateErrorType::GotReadyWhenOk),
        }
    }

    pub fn send(self, id: u64) -> Result<Self, StateErrorType> {
        match self {
            SenderState::Ready => Ok(SenderState::WaitingForOk(id)),
            SenderState::WaitingForOk(_) => Err(StateErrorType::GotSendWhenWaitingForOk),
            SenderState::WaitingForReady => Err(StateErrorType::GotSendWhenWaitingForReady),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LockState {
    Unlocked(SenderState),
    Locked(SenderState),
}

#[derive(Debug, Serialize)]
pub struct TurtleMessage<T> {
    pub id: u64,
    pub message: T,
}

#[derive(Debug)]
pub struct TurtleSenderActor {
    connection: Addr<TurtleConnection>,
    state: LockState,
    lock_queue: VecDeque<oneshot::Sender<()>>,
    message_queue: VecDeque<serde_json::Value>,
    next_id: u64,
}

impl TurtleSenderActor {
    pub fn new(connection: Addr<TurtleConnection>) -> Self {
        TurtleSenderActor {
            connection,
            state: LockState::Unlocked(SenderState::Ready),
            lock_queue: VecDeque::new(),
            message_queue: VecDeque::new(),
            next_id: 0,
        }
    }

    pub fn send<M: Serialize>(&mut self, msg: M) -> Result<(), TurtleSendError> {
        match &mut self.state {
            LockState::Unlocked(state @ SenderState::Ready)
            | LockState::Locked(state @ SenderState::Ready) => {
                *state = state
                    .send(self.next_id)
                    .map_err(TurtleSendError::StateError)?;
                let msg = TurtleMessage {
                    id: self.next_id,
                    message: msg,
                };

                // self.do_send(serde_json::to_string(&msg).unwrap()).await;
                self.connection
                    .try_send(turtle_connection::SendMessage(
                        serde_json::to_string(&msg).map_err(TurtleSendError::SerializeError)?,
                    ))
                    .map_err(|_| TurtleSendError::ConnectionClosed)?;
                self.next_id += 1;
            }
            LockState::Unlocked(_) | LockState::Locked(_) => {
                self.message_queue
                    .push_back(serde_json::to_value(&msg).unwrap());
            }
        };

        Ok(())
    }

    pub fn ok(&mut self, id: u64) -> Result<(), StateErrorType> {
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
}

impl Actor for TurtleSenderActor {
    type Context = Context<Self>;
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), TurtleSendError>")]
pub struct SendCommand<C>(pub C);

impl<C> Handler<SendCommand<C>> for TurtleSenderActor
where
    C: Command + Serialize + 'static,
{
    type Result = Result<(), TurtleSendError>;

    fn handle(&mut self, msg: SendCommand<C>, _ctx: &mut Self::Context) -> Self::Result {
        self.send(msg.0)
    }
}
