use crate::turtle::turtle_sender::turtle_sender_inner::TurtleSenderInner;
use crate::turtle::turtle_sender::{turtle_sender_inner, LockedTurtleSender};
use crate::turtle_scheme;
use actix::prelude::*;
use tokio::sync::oneshot;
use tracing::{error, warn};

use self::locked_state::{LockedState, QueueMessage};

use super::TurtleLockedError;

/// Used by the turtle sender to track wether the sender is locked.
/// Normally SenderState is unlocked and will pass any message on to the sender inner.
/// When lock is called the state will be switched to Locked and a LockedTurtleSender will be
/// returned with our inner. Any future sends will be added to a queue until the LockedTurtleSender
/// is dropped.
#[derive(Clone)]
pub enum SenderState {
    Normal(Addr<TurtleSenderInner>),
    Locked(Addr<LockedState>),
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
                let (tx, rx) = oneshot::channel();
                let locked_state = LockedState::new(rx).start();
                let locked = LockedTurtleSender::new(sender, tx);
                *self = SenderState::Locked(locked_state);

                Ok(locked)
            }
            SenderState::Locked(_) => {
                *self = state;

                Err(())
            }
            SenderState::Transitioning => {
                error!("Sender was in invalid state Transitioning when lock was called");

                Err(())
            }
        }
    }

    pub async fn send(&mut self, message: turtle_scheme::Message) {
        match self {
            SenderState::Normal(sender) => {
                let result = sender.try_send(turtle_sender_inner::SendMessage(message));
                if result.is_err() {
                    warn!("Tried to send message with closed or full TurtleSenderInner");
                }
            }
            SenderState::Locked(state) => match state.send(QueueMessage(message)).await {
                Ok(Some(inner)) => {
                    *self = SenderState::Normal(inner);
                }
                Err(_) => {
                    warn!("Unable to queue turtle message");
                }
                _ => {}
            },
            SenderState::Transitioning => {
                panic!("Send was in invalid state Transition when send was called")
            }
        }
    }

    // pub async fn request(&mut self, request: turtle_scheme::RequestType) -> Result<turtle_scheme::ResponseType, MailboxError> {
    //     match self {
    //         SenderState::Normal(sender) => {
    //
    //         }
    //     }
    // }

    pub fn close(&self) -> Result<(), TurtleLockedError> {
        if let SenderState::Normal(sender) = self {
            sender.do_send(turtle_sender_inner::CloseSenderInner);

            return Ok(());
        }

        Err(TurtleLockedError)
    }
}

mod locked_state {
    use actix::prelude::*;

    use tracing::error;

    use super::super::turtle_sender_inner::SendMessage;

    use crate::turtle_scheme;

    use crate::turtle::turtle_sender::turtle_sender_inner::TurtleSenderInner;

    use tokio::sync::oneshot;

    pub enum LockedState {
        Starting(oneshot::Receiver<Addr<TurtleSenderInner>>),
        Locked(Vec<turtle_scheme::Message>),
        Unlocking(Addr<TurtleSenderInner>),
    }

    impl LockedState {
        pub fn new(unlock_channel: oneshot::Receiver<Addr<TurtleSenderInner>>) -> Self {
            LockedState::Starting(unlock_channel)
        }
    }

    impl Actor for LockedState {
        type Context = Context<Self>;

        fn started(&mut self, _ctx: &mut Self::Context) {
            if let LockedState::Starting(unlock_channel) =
                std::mem::replace(self, LockedState::Locked(Vec::new()))
            {
                let fut =
                    fut::wrap_future(unlock_channel).map(|result, actor, _ctx| match result {
                        Ok(inner) => {
                            if let LockedState::Locked(queue) =
                                std::mem::replace(actor, LockedState::Unlocking(inner.clone()))
                            {
                                for message in queue {
                                    inner.do_send(SendMessage(message));
                                }
                            } else {
                                error!("Unlock channel completed while not in locked state");
                            }
                        }
                        Err(_) => {
                            error!("Failed to get TurtleSenderInner back from LockedTurtleSender");
                        }
                    });
            } else {
                error!("Started called while LockedState was not in LockedState::Starting");
            }
        }
    }

    pub(crate) struct QueueMessage(pub turtle_scheme::Message);

    impl Message for QueueMessage {
        type Result = Option<Addr<TurtleSenderInner>>;
    }

    impl Handler<QueueMessage> for LockedState {
        type Result = Option<Addr<TurtleSenderInner>>;

        fn handle(&mut self, msg: QueueMessage, ctx: &mut Self::Context) -> Self::Result {
            match self {
                LockedState::Locked(queue) => {
                    queue.push(msg.0);

                    None
                }
                LockedState::Unlocking(inner) => {
                    ctx.stop();
                    inner.do_send(SendMessage(msg.0));

                    Some(inner.clone())
                }
                LockedState::Starting(_) => {
                    error!("Tried to queue message while LockedState was still starting");

                    None
                }
            }
        }
    }
}
