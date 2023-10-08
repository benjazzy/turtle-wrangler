mod locked_turtle_sender;
mod sender_state;
mod turtle_sender_inner;

use actix::prelude::*;
use tracing::warn;

use super::turtle_connection::{self, TurtleConnection};
use crate::turtle_scheme::{self, TurtleCommand};
pub use locked_turtle_sender::LockedTurtleSender;
use sender_state::SenderState;
use turtle_sender_inner::TurtleSenderInner;

#[derive(Debug)]
pub struct TurtleLockedError;

impl std::fmt::Display for TurtleLockedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to close turtle connection because there is a lock"
        )
    }
}

impl std::error::Error for TurtleLockedError {}

#[derive(Clone)]
pub struct TurtleSender {
    sender: SenderState,
}

impl TurtleSender {
    pub fn new(turtle_connection: Addr<TurtleConnection>) -> Self {
        let inner = TurtleSenderInner::new(turtle_connection).start();
        let sender = SenderState::new(inner);
        TurtleSender { sender }
    }

    pub fn lock(&mut self) -> Result<LockedTurtleSender, ()> {
        self.sender.lock()
    }

    pub async fn send(&mut self, command: turtle_scheme::TurtleCommand) {
        self.sender
            .send(turtle_scheme::Message::Command { command })
            .await;
    }

    pub fn close(&self) -> Result<(), TurtleLockedError> {
        self.sender.close()
    }
}
