use axum::extract::ws;
use futures::stream::SplitSink;
use futures::SinkExt;
use kameo::Actor;
use kameo::{message::Context, messages};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{debug, error};
use turtle_types::turtle_scheme::turtle_messages::Command;

#[derive(Debug, Clone, Copy, thiserror::Error)]
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

#[derive(Debug, thiserror::Error)]
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

impl From<axum::Error> for TurtleSendError {
    fn from(_: axum::Error) -> Self {
        TurtleSendError::ConnectionClosed
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

#[derive(Debug, Actor)]
pub struct TurtleSender {
    name: Arc<str>,
    connection: SplitSink<ws::WebSocket, ws::Message>,
    state: LockState,
    message_queue: VecDeque<TurtleMessage>,
    lock_queue: VecDeque<oneshot::Sender<()>>,
}

impl TurtleSender {
    pub fn new(name: Arc<str>, connection: SplitSink<ws::WebSocket, ws::Message>) -> Self {
        TurtleSender {
            name,
            connection,
            state: LockState::Unlocked(SenderState::WaitingForReady),
            message_queue: VecDeque::new(),
            lock_queue: VecDeque::new(),
        }
    }

    pub async fn send(&mut self, msg: TurtleMessage) -> Result<(), TurtleSendError> {
        match &mut self.state {
            LockState::Unlocked(state @ SenderState::Ready)
            | LockState::Locked(state @ SenderState::Ready) => {
                *state = state.send(msg.id).map_err(TurtleSendError::StateError)?;

                self.connection
                    .send(ws::Message::Text(serde_json::to_string(&msg)?.into()))
                    .await?;
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

    pub async fn ready(&mut self) -> Result<(), TurtleSendError> {
        match &mut self.state {
            LockState::Unlocked(sender_state) | LockState::Locked(sender_state) => {
                *sender_state = sender_state.ready().map_err(TurtleSendError::StateError)?;
                if let Some(msg) = self.message_queue.pop_front() {
                    self.send(msg).await?;
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
                    self.state = LockState::Locked(SenderState::Ready);
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

#[messages]
impl TurtleSender {
    #[message]
    pub async fn lock_sender(&mut self) {
        let (tx, rx) = oneshot::channel();
        self.lock(tx);
        rx.await;
        debug!("{} locked", self.name);
    }

    #[message]
    pub async fn unlock_sender(&mut self) {
        self.unlock();
        debug!("{} unlocked", self.name);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SendCommand<C: Command>(pub u64, pub C);

impl<C> kameo::message::Message<SendCommand<C>> for TurtleSender
where
    C: Command + Send + 'static,
{
    type Reply = Result<(), TurtleSendError>;

    async fn handle(
        &mut self,
        SendCommand(id, command): SendCommand<C>,
        ctx: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        self.message_queue.push_back(TurtleMessage {
            id,
            message: serde_json::to_value(command)?,
        });
        let message = self
            .message_queue
            .pop_front()
            .expect("Just pushed command onto queue");

        self.send(message).await
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GotOk(pub u64);

impl kameo::message::Message<GotOk> for TurtleSender {
    type Reply = ();

    async fn handle(
        &mut self,
        GotOk(id): GotOk,
        ctx: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        debug!("Setting {} to OK", self.name);
        if let Err(e) = self.ok(id) {
            error!("State error setting ok state for {}: {e}", self.name);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GotReady;

impl kameo::message::Message<GotReady> for TurtleSender {
    type Reply = ();

    async fn handle(&mut self, _: GotReady, ctx: Context<'_, Self, Self::Reply>) -> Self::Reply {
        debug!("Setting {} to READY", self.name);
        if let Err(e) = self.ready().await {
            error!("State error setting ready state for {}: {e}", self.name);
        }
    }
}
