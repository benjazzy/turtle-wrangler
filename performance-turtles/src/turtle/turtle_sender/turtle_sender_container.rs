use std::collections::VecDeque;

use actix::prelude::*;
use futures_util::{FutureExt, TryFutureExt};
use tokio::sync::oneshot;
use tracing::{error, warn};

use crate::turtle::Close;
use crate::turtle_scheme;

use super::turtle_sender_inner::{SendCommand, SendRequest};
use super::{LockedTurtleSender, TurtleSenderInner};

enum LockResult {
    Lock(LockedTurtleSender),
    Wait(oneshot::Receiver<LockedTurtleSender>),
}

pub enum LockedItem {
    Command(SendCommand),
    Request(
        SendRequest,
        oneshot::Sender<Result<turtle_scheme::ResponseType, anyhow::Error>>,
    ),
    Lock(oneshot::Sender<LockedTurtleSender>),
    Close,
}

pub enum TurtleSenderContainer {
    Normal(Addr<TurtleSenderInner>),
    Locked(VecDeque<LockedItem>),
}

impl TurtleSenderContainer {
    pub fn new(sender: Addr<TurtleSenderInner>) -> Self {
        TurtleSenderContainer::Normal(sender)
    }

    fn lock(&mut self, ctx: &mut Context<Self>) -> LockResult {
        let mut old = std::mem::replace(self, TurtleSenderContainer::Locked(VecDeque::new()));

        match old {
            TurtleSenderContainer::Normal(inner) => {
                let lock = self.do_lock(ctx, inner);

                LockResult::Lock(lock)
            }
            TurtleSenderContainer::Locked(mut queue) => {
                let (tx, rx) = oneshot::channel();

                queue.push_back(LockedItem::Lock(tx));

                LockResult::Wait(rx)
            }
        }
    }

    fn do_lock(
        &mut self,
        ctx: &mut Context<Self>,
        inner: Addr<TurtleSenderInner>,
    ) -> LockedTurtleSender {
        LockedTurtleSender::new(inner, ctx.address().recipient())
    }

    fn unlock(&mut self, ctx: &mut Context<Self>, inner: Addr<TurtleSenderInner>) {
        if let TurtleSenderContainer::Locked(queue) = self {
            while let Some(item) = queue.pop_front() {
                match item {
                    LockedItem::Command(command) => {
                        if let Err(e) = inner.try_send(command) {
                            warn!("Problem sending command to turtle sender inner {e}");
                        }
                    }
                    LockedItem::Request(request, tx) => {
                        let inner = inner.clone();
                        let fut = fut::wrap_future::<_, Self>(async move {
                            let result = inner.send(request).await;
                            match result {
                                Ok(r) => tx.send(r),
                                Err(e) => tx.send(Err(anyhow::Error::new(e))),
                            };
                        });

                        ctx.spawn(fut);
                    }
                    LockedItem::Lock(tx) => {
                        let lock = self.do_lock(ctx, inner);
                        tx.send(lock);
                        return;
                    }
                    LockedItem::Close => {
                        inner.do_send(Close);
                        ctx.stop();
                    }
                }
            }
        }
    }
}

impl Actor for TurtleSenderContainer {
    type Context = Context<Self>;
}

impl Handler<SendCommand> for TurtleSenderContainer {
    type Result = ();

    fn handle(&mut self, msg: SendCommand, ctx: &mut Self::Context) -> Self::Result {
        match self {
            TurtleSenderContainer::Normal(sender) => {
                if let Err(e) = sender.try_send(msg) {
                    error!("Problem passing command on to inner {e}");
                }
            }
            TurtleSenderContainer::Locked(queue) => queue.push_back(LockedItem::Command(msg)),
        }
    }
}

impl Handler<SendRequest> for TurtleSenderContainer {
    type Result = ResponseFuture<Result<turtle_scheme::ResponseType, anyhow::Error>>;

    fn handle(&mut self, msg: SendRequest, ctx: &mut Self::Context) -> Self::Result {
        match self {
            TurtleSenderContainer::Normal(sender) => {
                Box::pin(sender.send(msg).map(|result| result?))
            }
            TurtleSenderContainer::Locked(queue) => {
                let (tx, rx) = oneshot::channel();

                queue.push_back(LockedItem::Request(msg, tx));

                Box::pin(rx.map(|result| match result {
                    Ok(r) => r,
                    Err(e) => Err(anyhow::Error::new(e)),
                }))
            }
        }
    }
}

pub struct Lock;

impl Message for Lock {
    type Result = Result<LockedTurtleSender, oneshot::error::RecvError>;
}

impl Handler<Lock> for TurtleSenderContainer {
    type Result = ResponseFuture<Result<LockedTurtleSender, oneshot::error::RecvError>>;

    fn handle(&mut self, _msg: Lock, ctx: &mut Self::Context) -> Self::Result {
        match self.lock(ctx) {
            LockResult::Lock(lock) => Box::pin(async move { Ok(lock) }),
            LockResult::Wait(rx) => Box::pin(rx),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DoUnlock(pub Addr<TurtleSenderInner>);

impl Handler<DoUnlock> for TurtleSenderContainer {
    type Result = ();

    fn handle(&mut self, msg: DoUnlock, ctx: &mut Self::Context) -> Self::Result {
        self.unlock(ctx, msg.0)
    }
}

impl Handler<Close> for TurtleSenderContainer {
    type Result = ();

    fn handle(&mut self, msg: Close, ctx: &mut Self::Context) -> Self::Result {
        match self {
            TurtleSenderContainer::Normal(inner) => {
                inner.do_send(Close);
                ctx.stop();
            }

            TurtleSenderContainer::Locked(queue) => {
                queue.push_front(LockedItem::Close);
            }
        }
    }
}
