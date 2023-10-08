use actix::prelude::*;
use tracing::warn;
use crate::turtle::turtle_sender::{LockedTurtleSender, turtle_sender_inner};
use crate::turtle::turtle_sender::turtle_sender_inner::TurtleSenderInner;
use crate::turtle_scheme;

/// Used by the turtle sender to track wether the sender is locked.
/// Normally SenderState is unlocked and will pass any message on to the sender inner.
/// When lock is called the state will be switched to Locked and a LockedTurtleSender will be
/// returned with our inner. Any future sends will be added to a queue until the LockedTurtleSender
/// is dropped.
pub enum SenderState {
    Normal(Addr<TurtleSenderInner>),
    Locked {
        queue: Vec<turtle_scheme::Message>,
        unlock_channel: tokio::sync::oneshot::Receiver<Addr<TurtleSenderInner>>,
    },
    Transitioning,
}

impl SenderState {
    pub fn new(sender: Addr<TurtleSenderInner>) -> Self {
        SenderState::Normal(sender)
    }

    pub fn lock(&mut self) -> Result<LockedTurtleSender, ()> {
        let state = std::mem::replace(self, SenderState::Transitioning);
        match state {
            SenderState::Normal(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let locked = LockedTurtleSender::new(sender, tx);
                *self = SenderState::Locked {
                    queue: Vec::new(),
                    unlock_channel: rx,
                };

                Ok(locked)
            }
            SenderState::Locked {
                queue,
                unlock_channel,
            } => {
                *self = SenderState::Locked {
                    queue,
                    unlock_channel,
                };

                Err(())
            }
            SenderState::Transitioning => {
                panic!("Sender was in invalid state Transitioning when lock was called")
            }
        }
    }

    pub fn send(&mut self, message: turtle_scheme::Message) {
        match self {
            SenderState::Normal(sender) => {
                let result = sender.try_send(turtle_sender_inner::SendMessage(message));
                if result.is_err() {
                    warn!("Tried to send message with closed or full TurtleSenderInner");
                }
            }
            SenderState::Locked {
                queue,
                unlock_channel,
            } => {
                if let Ok(sender) = unlock_channel.try_recv() {
                    if let SenderState::Locked {
                        queue,
                        unlock_channel: _,
                    } = std::mem::replace(self, SenderState::Normal(sender.clone()))
                    {
                        for message in queue {
                            sender.do_send(turtle_sender_inner::SendMessage(message));
                        }
                    }
                } else {
                    queue.push(message);
                }
            }
            SenderState::Transitioning => {
                panic!("Send was in invalid state Transition when send was called")
            }
        }
    }
}
