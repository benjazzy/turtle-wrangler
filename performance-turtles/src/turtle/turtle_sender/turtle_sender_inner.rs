use std::collections::HashMap;
use std::time::Duration;

use crate::turtle::turtle_connection::TurtleConnection;
use crate::turtle::{turtle_connection, Close};
use crate::turtle_scheme::{self, ResponseType};
use crate::turtle_scheme::{RequestType, TurtleCommand};
use actix::prelude::*;
use serde::Serialize;
use tokio::sync::oneshot;
use tracing::{debug, error, warn};
use turtle_sender_queue::SenderQueue;

#[derive(Serialize)]
struct SentCommand {
    pub id: u64,
    pub command: TurtleCommand,
}

pub struct TurtleSenderInner {
    connection: Addr<TurtleConnection>,
    sent_command: Option<(SentCommand, SpawnHandle)>,
    sender_queue: SenderQueue<TurtleCommand>,
    next_id: u64,
    outstanding_requests: HashMap<u64, oneshot::Sender<ResponseType>>,
    name: String,
}

impl TurtleSenderInner {
    pub fn new(connection: Addr<TurtleConnection>, name: String) -> Self {
        TurtleSenderInner {
            connection,
            sent_command: None,
            sender_queue: SenderQueue::new(),
            next_id: 0,
            outstanding_requests: HashMap::new(),
            name,
        }
    }

    fn send(&mut self, ctx: &mut Context<Self>, command: TurtleCommand) {
        if let Some(c) = self.sender_queue.send(command) {}
    }

    fn request(
        &mut self,
        ctx: &mut Context<Self>,
        request_type: RequestType,
        tx: oneshot::Sender<ResponseType>,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let request = turtle_scheme::Request {
            id,
            request: request_type,
        };

        self.outstanding_requests.insert(id, tx);

        self.send(ctx, TurtleCommand::Request(request));
    }

    fn response(&mut self, response: turtle_scheme::Response) {
        if let Some(tx) = self.outstanding_requests.remove(&response.id) {
            let _ = tx.send(response.response);
        } else {
            warn!("Got response for unknown request {:?}", response);
        }
    }

    fn ok(&mut self, ctx: &mut Context<Self>, id: u64) {
        match &self.sent_command {
            Some((command, handle)) => {
                if id != command.id {
                    error!("Got ok message for unkonw command from {}", self.name);
                    return;
                }

                ctx.cancel_future(*handle);
                self.sent_command = None;
            }
            None => error!("Got ok message form {} without sending command", self.name),
        }
    }

    fn ready(&mut self, ctx: &mut Context<Self>) {
        if let Some(c) = self.sender_queue.ready() {
            self.do_send(ctx, c);
        }
    }

    fn do_send(&mut self, ctx: &mut Context<Self>, command: TurtleCommand) {
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

        let handle = ctx.run_later(
            Duration::from_secs(5),
            |turtle: &mut TurtleSenderInner, ctx| {
                warn!("Timed out waiting for turtle {} to send ok", turtle.name);
            },
        );

        self.sent_command = Some((sent_command, handle));
    }
}

impl Actor for TurtleSenderInner {
    type Context = Context<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.connection.do_send(Close);
        debug!("TurtleSenderInner closed");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub turtle_scheme::Message);

impl Handler<SendMessage> for TurtleSenderInner {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
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

pub struct SendRequest(pub RequestType);

impl Message for SendRequest {
    type Result = Result<ResponseType, anyhow::Error>;
}

impl Handler<SendRequest> for TurtleSenderInner {
    type Result = ResponseFuture<Result<ResponseType, anyhow::Error>>;

    fn handle(&mut self, msg: SendRequest, ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = oneshot::channel();

        self.request(ctx, msg.0, tx);

        let fut = async move { rx.await.map_err(|e| anyhow::Error::new(e)) };
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
