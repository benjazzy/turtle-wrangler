use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;
use actix::fut::wrap_future;
use crate::turtle::turtle_connection::TurtleConnection;
use crate::turtle::{turtle_connection, Close};
use crate::turtle_scheme::{self, Command, Request};
use actix::prelude::*;
use futures_util::TryFutureExt;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::{debug, error, warn};
use turtle_sender_queue::SenderQueue;

enum SenderState {
    Ready,
    WaitingForOk { id: u64, ok_watcher: SpawnHandle, ready_watcher: SpawnHandle },
    WaitingForReady{ ready_watcher: SpawnHandle },
}

enum LockState {
    Unlocked,
    Locking(VecDeque<oneshot::Sender<Addr<TurtleSenderInner>>>),
    Locked(VecDeque<oneshot::Sender<Addr<TurtleSenderInner>>>, Addr<TurtleSenderInner>),
    Locker,
}

pub struct TurtleSenderInner {
    connection: Addr<TurtleConnection>,
    // sent_command: Option<(u64, SpawnHandle)>,
    sender_state: SenderState,
    lock_state: LockState,
    sender_queue: SenderQueue<(serde_json::Value, Pin<Box<dyn Future<Output=()>>>)>,
    next_id: u64,
    outstanding_requests: HashMap<u64, oneshot::Sender<serde_json::Value>>,

    // Any containers needs to make sure there is only
    // one flush called at atime
    flush_sender: Option<oneshot::Sender<()>>,
    name: String,
}

impl TurtleSenderInner {
    pub fn new(connection: Addr<TurtleConnection>, name: String) -> Self {
        TurtleSenderInner {
            connection,
            // sent_command: None,
            sender_state: SenderState::Ready,
            lock_state: LockState::Unlocked,
            sender_queue: SenderQueue::new(),
            next_id: 0,
            outstanding_requests: HashMap::new(),
            flush_sender: None,
            name,
        }
    }

    fn send<C: Command + Serialize>(&mut self, ctx: &mut Context<Self>, command: C, ready_watcher: Pin<Box<dyn Future<Output = ()>>>) {
        let command = match serde_json::to_value(command) {
            Ok(c) => {
                c
            }
            Err(e) => {
                error!("Problem serializing command: {e}");
                return;
            }
        };
        
        match (&self.lock_state, &self.sender_state) {
            (LockState::Unlocked | LockState::Locker, SenderState::Ready) => {
                if C::NEEDS_LOCK
                if let Some((command, timeout_watcher)) = self.sender_queue.send((command, ready_watcher)) {
                    self.do_send(ctx, command, timeout_watcher);
                }
            }
            
            // If we are locked or in the process of locking or are not ready to send 
            // a message then add the message to the queue.
            _ => self.sender_queue.push((command, ready_watcher))
        }
    }

    fn request(
        &mut self,
        ctx: &mut Context<Self>,
        request: impl Request + Serialize,
        tx: oneshot::Sender<serde_json::Value>,
    ) {
        #[derive(Serialize)]
        struct SentRequest<T: Request> {
            pub id: u64,
            pub request: T,
        }

        impl<T> Command for SentRequest<T> where T: Request {
            const NEEDS_LOCK: bool = T::NEEDS_LOCK;
        }

        let id = self.next_id;
        self.next_id += 1;

        let sent_request = SentRequest {
            id,
            request,
        };

        self.outstanding_requests.insert(id, tx);

        self.send(ctx, sent_request, Box::pin(async {}));
    }

    fn response(&mut self, id: u64, value: serde_json::Value) {
        match (self.outstanding_requests.remove(&id), &self.lock_state) {
            (Some(tx), _) => {
                let _ = tx.send(value);
            },
            (None, LockState::Locked(_, locking_turtle)) => todo!("Pass on response to locking turtle"),
            (None, _) => warn!("Got response for unknown request {:?}", value),
        }
    }

    fn ok(&mut self, ctx: &mut Context<Self>, new_id: u64) {
        // If this sender is locked we need to pass the ok on to the locking sender.
        // A sender should only be able to be locked if it is in the ready state so
        // we don't care about the ok message.
        if let LockState::Locked(_, ref lock_addr) = self.lock_state {
            if let Err(e) = lock_addr.try_send(SetOk(new_id)) {
                // TODO handle this case. Maybe force unlock the turtle?
                error!("Turtle {} failed to pass ok message on to locker: {e}", self.name);
            }
            
            return;
        }
        
        match self.sender_state {
            SenderState::Ready => {
                warn!("Turtle {} sent ok while sender was ready", self.name);
            }
            SenderState::WaitingForOk { id, ok_watcher, ready_watcher } => {
                if new_id != id {
                    warn!("Turtle {} sent ok message with incorrect id {id}", self.name);
                    return;
                }

                ctx.cancel_future(ok_watcher);
                self.sender_state = SenderState::WaitingForReady { ready_watcher };
            }
            SenderState::WaitingForReady { .. } => {
                warn!("Turtle {} sent ok message while sender was expecting a ready", self.name);
            }
        }
    }

    fn ready(&mut self, ctx: &mut Context<Self>) {
        // If this sender is locked we need to pass the ready on to the locking sender.
        // A sender should only be able to be locked if it is in the ready state so
        // we don't care about the ready message.
        if let LockState::Locked(_, ref lock_addr) = self.lock_state {
            if let Err(e) = lock_addr.try_send(Ready) {
                // TODO handle this case. Maybe force unlock the turtle?
                error!("Turtle {} failed to pass ready message on to locker: {e}", self.name);
            }

            return;
        }
        
        match (&mut self.lock_state, &self.sender_state) {
            // If we are unlocked the locker or locking and we get a ready when we don't expect it the throw a warning.
            (LockState::Unlocked | LockState::Locker | LockState::Locking(_), SenderState::Ready) => {
                warn!("Turtle {} sent ready while sender was already ready", self.name);
            }
            (LockState::Unlocked | LockState::Locker | LockState::Locking(_), SenderState::WaitingForOk { .. }) => {
                warn!("Turtle {} sent ready while sender was expecting an ok", self.name);
            }
            
            // If we are unlocked or are the locker then cancel the timeout future and set our state to ready.
            (LockState::Unlocked | LockState::Locker, SenderState::WaitingForReady { ready_watcher }) => {
                ctx.cancel_future(*ready_watcher);
                self.sender_state = SenderState::Ready;
            }
            
            // If we are in the middle of locking then cancel the timeout future and set our state to ready as normal but 
            // also create another turtle sender and send it to whoever is next in line to lock.. 
            (LockState::Locking(lock_waiters), SenderState::WaitingForReady { ready_watcher }) => {
                ctx.cancel_future(*ready_watcher);
                self.sender_state =SenderState::Ready;
                
                if let Some(tx) = lock_waiters.pop_front() {
                    todo!("Construct locked sender")
                } else {
                    error!("Turtle {} was attempting to lock but there was nobody to send the lock to. Setting turtle to unlocked.", self.name);
                    self.lock_state = LockState::Unlocked;
                }
            }
            
            // If we are already locked then pass the ready onto the turtle that is locking.
            (LockState::Locked(_, lock_addr), _) => {
                if let Err(e) = lock_addr.try_send(Ready) {
                    // TODO handle this case. Maybe force unlock the turtle?
                    error!("Turtle {} failed to pass ready message on to locker: {e}", self.name);
                }
            }
        }
        
        match self.sender_state {
            SenderState::Ready => {
                warn!("Turtle {} sent ready while sender was already ready", self.name);
            }
            SenderState::WaitingForOk { .. } => {
                warn!("Turtle {} sent ready while sender was expecting an ok", self.name);
            }
            SenderState::WaitingForReady { ready_watcher } => {
                ctx.cancel_future(ready_watcher);
                self.sender_state = SenderState::Ready;
            }
        }

        if let Some((command, ready_watcher)) = self.sender_queue.ready() {
            self.do_send(ctx, command, ready_watcher);
        }
    }

    fn do_send(&mut self, ctx: &mut Context<Self>, command: serde_json::Value, ready_watcher: Pin<Box<dyn Future<Output = ()>>>)
    {
        #[derive(Debug, Serialize)]
        struct SentCommand{
            id: u64,
            command: serde_json::Value,
        }

        let id = self.next_id;
        self.next_id += 1;

        let sent_command = SentCommand { id, command };

        let message = serde_json::to_string(&sent_command).expect("Problem serializing command");

        debug!("Sending message to {}: {message}", self.name);
        if let Err(e) = self
            .connection
            .try_send(turtle_connection::SendMessage(message))
        {
            error!("Failed to send message to {}'s connection {e}", self.name);
            return;
        }
        
        let ok_watcher = ctx.run_later(Duration::from_secs(1), move |act, _| {
            warn!("Turtle {} failed to send a valid ok before timeout", act.name);
        });
        let ready_watcher = ctx.run_later(Duration::from_secs(5), |act, ctx| {
            warn!("Turtle {} failed to send a ready before timeout", act.name);
            ctx.spawn(wrap_future(ready_watcher));
        });

        self.sender_state = SenderState::WaitingForOk { id, ok_watcher, ready_watcher };


        // let handle = ctx.run_later(
        //     Duration::from_secs(5),
        //     |turtle: &mut TurtleSenderInner, _ctx| {
        //         warn!("Timed out waiting for turtle {} to send ok", turtle.name);
        //         received_notifier.send(Err(())).unwrap()
        //     },
        // );

        // self.sent_command = Some((sent_command.id, handle));
    }
}

impl Actor for TurtleSenderInner {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.connection.do_send(Close);
        debug!("TurtleSenderInner closed");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub turtle_scheme::Message);

impl Handler<SendMessage> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) -> Self::Result {
        let message = serde_json::to_string(&msg.0).expect("Problem serializing turtle message");

        self.connection
            .try_send(turtle_connection::SendMessage(message));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendCommand(pub TurtleCommand);

impl Handler<SendCommand> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: SendCommand, ctx: &mut Self::Context) -> Self::Result {
        self.send(ctx, msg.0);
    }
}

impl Handler<Close> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, _msg: Close, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Ready;

impl Handler<Ready> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, _msg: Ready, ctx: &mut Self::Context) -> Self::Result {
        self.ready(ctx);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetOk(pub u64);

impl Handler<SetOk> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: SetOk, ctx: &mut Self::Context) -> Self::Result {
        self.ok(ctx, msg.0);
    }
}

pub struct SendRequest<R: Request>(pub Request);

impl<R: Request> Message for SendRequest<R> {
    type Result = Result<R::Response, anyhow::Error>;
}

#[derive(Debug, Clone, Error)]
pub enum SendRequestError {
    #[error("Response sender was dropped before the response was received")]
    SenderDropped,

    #[error("Problem deserializing the response")]
    DeserializeError(serde_json::Value),
}

impl<R: Request> Handler<SendRequest<R>> for TurtleSenderInner {
    type Result = ResponseFuture<Result<R::Response, anyhow::Error>>;

    fn handle(&mut self, msg: SendRequest<R>, ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();

        self.request(ctx, msg.0, tx);

        let fut = async move {
            match rx.await {
                Ok(value) => {
                    serde_json::from_value(value).map_err(|_| SendRequestError::DeserializeError(value))
                }
                Err(_) => Err(SendRequestError::SenderDropped),
            }
        };
        Box::pin(fut)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifyResponse(pub turtle_scheme::Response);

impl Handler<NotifyResponse> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: NotifyResponse, _ctx: &mut Self::Context) -> Self::Result {
        self.response(msg.0);
    }
}

#[derive(Debug, Error)]
#[error("Turtle Sender Inner is already flushing")]
pub struct AlreadyFlushingError;

/// Flush will wait until there are no commands in queue or outstanding requests and then return.
/// It is important that there is only one flush called at a time. If a second flush is called it
/// will return an error immediately.
/// Also note that messages can still be sent while waiting for a flush so the caller of flush must
/// ensure that sending is blocked or flush may never return.
#[derive(Message)]
#[rtype(result = "Result<(), AlreadyFlushingError>")]
pub struct Flush;

impl Handler<Flush> for TurtleSenderInner {
    type Result = ResponseFuture<Result<(), AlreadyFlushingError>>;

    fn handle(&mut self, msg: Flush, _ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();

        if self.flush_sender.is_some() {
            //TODO make this more betterer.
            return Box::pin(async { Err(AlreadyFlushingError) });
        }

        self.flush_sender = Some(tx);

        Box::pin(rx.map_err(|_| ()))
    }
}
