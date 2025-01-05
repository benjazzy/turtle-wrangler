use std::collections::VecDeque;

use actix::Addr;
use actix_web::Result;
use serde::Serialize;
use tokio::sync::oneshot;

use crate::turtle_scheme::{Command, TurtleCommand};

use super::turtle_connection::{self, TurtleConnection};

#[derive(Debug, Clone, Copy)]
pub struct StateError(LockState, StateErrorType);

#[derive(Debug, Clone, Copy)]
pub enum StateErrorType {
    GotWrongOkId,
    GotSendWhenWaitingForOk,
    GotSendWhenWaitingForReady,
    GotOkWhenReady,
    GotOkWhenWaitingForReady,
    GotReadyWhenReady,
    GotReadyWhenOk,
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
enum LockState {
    Unlocked(SenderState),
    Locked(SenderState),
}

#[derive(Debug, Serialize)]
struct TurtleMessage<T> {
    id: u64,
    message: T,
}

#[derive(Debug)]
struct TurtleSenderActor {
    connection: Addr<TurtleConnection>,
    state: LockState,
    lock_queue: VecDeque<oneshot::Sender<()>>,
    message_queue: VecDeque<String>,
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

    async fn do_send(&mut self, msg: String) {
        self.connection
            .send(turtle_connection::SendMessage(msg))
            .await
            .unwrap();
    }

    pub async fn send<M: Command + Serialize>(&mut self, msg: M) -> Result<(), StateErrorType> {
        match &mut self.state {
            LockState::Unlocked(state @ SenderState::Ready)
            | LockState::Locked(state @ SenderState::Ready) => {
                *state = state.send(self.next_id)?;
                let msg = TurtleMessage {
                    id: self.next_id,
                    message: msg,
                };

                self.do_send(serde_json::to_string(&msg).unwrap()).await;
                self.next_id += 1;
            }
            LockState::Unlocked(_) | LockState::Locked(_) => {
                self.message_queue
                    .push_back(serde_json::to_string(&msg).unwrap());
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

    pub async fn ready(&mut self) -> Result<(), StateErrorType> {
        match self.state {
            LockState::Unlocked(state) => {
                if let Some(tx) = self.lock_queue.pop_back() {
                    if tx.send(()).is_ok() {
                        self.state = LockState::Locked(state);
                    }
                }
            }
            LockState::Locked(_) => {}
        }

        self.state = match self.state {
            LockState::Unlocked(mut sender_state) => {
                sender_state = sender_state.ready()?;
                if let Some(msg) = self.message_queue.pop_back() {
                    self.send(msg).await;
                    let id = self.next_id;
                    self.next_id += 1;

                    LockState::Unlocked(SenderState::WaitingForOk(id))
                } else {
                    LockState::Unlocked(sender_state)
                }
            }
            LockState::Locked(mut sender_state) => {
                sender_state = sender_state.ready()?;
                if let Some(tx) = self.lock_queue.pop_back() {
                    if tx.send(()).is_ok() {
                        LockState::Locked(sender_state)
                    } else {
                        tracing::warn!("Sender attempted to send lock after reciever was dropped");

                        LockState::Unlocked(sender_state)
                    }
                } else {
                    LockState::Unlocked(sender_state)
                }
            }
        };

        Ok(())
    }

    pub fn lock(&mut self, tx: oneshot::Sender<()>) {
        match self.state {
            LockState::Unlocked(SenderState::Ready) => {
                tx.send(()).unwrap();
                self.state = LockState::Locked(SenderState::Ready)
            }
            _ => self.lock_queue.push_back(tx),
        }
    }
}
