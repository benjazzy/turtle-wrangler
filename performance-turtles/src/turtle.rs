pub mod turtle_connection;
pub mod turtle_identifier;
mod turtle_receiver;
mod turtle_sender;
mod unknown_turtle_connection;

use actix::prelude::*;

use self::turtle_receiver::{CloseReceiver, TurtleReceiver};
use self::turtle_sender::{TurtleLockedError, TurtleSender};

#[derive(Clone)]
pub struct Turtle {
    sender: TurtleSender,
    receiver: Addr<TurtleReceiver>,
    name: String,
}

impl Turtle {
    pub fn new(sender: TurtleSender, receiver: Addr<TurtleReceiver>, name: String) -> Self {
        Turtle {
            sender,
            receiver,
            name,
        }
    }

    pub fn sender(&mut self) -> &mut TurtleSender {
        &mut self.sender
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn close(&self) -> Result<(), TurtleLockedError> {
        self.sender.close()?;
        self.receiver.do_send(CloseReceiver);

        Ok(())
    }

    pub fn force_close(&self) {
        self.receiver.do_send(CloseReceiver);
    }
}
