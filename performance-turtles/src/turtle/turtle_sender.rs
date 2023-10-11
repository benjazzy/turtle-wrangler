mod locked_turtle_sender;
// mod sender_state;
mod turtle_sender_container;
mod turtle_sender_inner;

use actix::prelude::*;
use tracing::warn;

use self::turtle_sender_container::{Lock, TurtleSenderContainer};
use self::turtle_sender_inner::SendCommand;

use super::turtle_connection::TurtleConnection;
use super::Close;
use crate::turtle_scheme::{self};
pub use locked_turtle_sender::LockedTurtleSender;
pub use turtle_sender_inner::{NotifyResponse, Ready, SetOk, TurtleSenderInner};

#[derive(Clone)]
pub struct TurtleSender {
    sender: Addr<TurtleSenderContainer>,
}

impl TurtleSender {
    pub fn new(inner: Addr<TurtleSenderInner>) -> Self {
        let sender = TurtleSenderContainer::new(inner).start();
        TurtleSender { sender }
    }

    pub async fn lock(&mut self) -> Result<LockedTurtleSender, anyhow::Error> {
        match self.sender.send(Lock).await {
            Ok(r) => r.map_err(anyhow::Error::new),
            Err(e) => Err(anyhow::Error::new(e)),
        }
    }

    pub fn send(&mut self, command: turtle_scheme::TurtleCommand) {
        if let Err(_e) = self.sender.try_send(SendCommand(command)) {
            warn!("Problem sending command to turtle sender container");
        }
    }

    // pub async fn request(&mut self, request: RequestType) -> ResponseType {
    //     self.sender.request(requset).await
    // }

    pub fn close(&self) {
        self.sender.do_send(Close)
    }
}
