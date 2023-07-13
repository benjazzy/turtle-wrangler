use crate::turtle_manager::TurtleSenderHandle;
use crate::turtle_scheme::{Request, RequestType, Response, ResponseType, TurtleCommand};
use futures_util::{stream::SplitSink, SinkExt};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::MissedTickBehavior;
use tokio::{net::TcpStream, select, sync::mpsc, time};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, warn};
use turtle_sender_queue::SenderQueue;

use super::{turtle_sender_message::TurtleSenderMessage, TurtleManagerHandle};

#[derive(Serialize)]
struct SentCommand {
    pub id: u64,
    pub command: TurtleCommand,
}

pub struct TurtleSenderInner {
    rx: mpsc::Receiver<TurtleSenderMessage>,
    ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    manager: TurtleManagerHandle,

    sent_command: Option<SentCommand>,
    sender_queue: SenderQueue<TurtleCommand>,
    command_timeout: time::Interval,
    next_id: u64,

    outstanding_requests: HashMap<u64, oneshot::Sender<ResponseType>>,

    name: &'static str,
}

impl TurtleSenderInner {
    pub fn new(
        rx: mpsc::Receiver<TurtleSenderMessage>,
        ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let mut command_timeout = time::interval(Duration::from_secs(5));
        command_timeout.set_missed_tick_behavior(MissedTickBehavior::Skip);

        TurtleSenderInner {
            rx,
            ws_sender,
            manager,
            name,
            sent_command: None,
            sender_queue: SenderQueue::new(),
            command_timeout: time::interval(Duration::from_secs(5)),
            next_id: 0,
            outstanding_requests: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        debug!("Starting Turtle Sender");
        let mut close_tx = None;

        loop {
            select! {
                _ = self.command_timeout.tick(), if self.sent_command.is_some() => {
                    warn!("Failed to get ok from turtle {} before timeout. Retrying command", self.name);
                    if let Some(c) = &self.sent_command {
                        self.send_command(c.command.clone()).await;
                    }
                }
                message = self.rx.recv() => {
                    if let Some(message) = message {
                        match message {
                            TurtleSenderMessage::Close(tx) => {
                                close_tx = Some(tx);
                                break;
                            }
                            TurtleSenderMessage::Request(request, tx) => {
                                self.request(request, tx).await;
                            }
                            TurtleSenderMessage::Response(response) => {
                                self.response(response).await;
                            }
                            TurtleSenderMessage::Message(message) => {
                                self.send_message(message).await;
                            }
                            TurtleSenderMessage::Command(command) => {
                                self.send(command).await;
                            }
                            TurtleSenderMessage::GotOk(id) => {
                                self.ok(id).await;
                            }
                            TurtleSenderMessage::Ready => {
                                self.ready().await;
                            }
                        }
                    } else {
                        self.manager.disconnect(self.name).await;
                        break;
                    }
                }
            }
        }

        debug!("Turtle sender shutting down for {}", self.name);
        let _ = self.ws_sender.close().await;
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    async fn request(&mut self, request_type: RequestType, tx: oneshot::Sender<ResponseType>) {
        let id = self.next_id;
        self.next_id += 1;
        let request = Request {
            id,
            request: request_type,
        };

        self.outstanding_requests.insert(id, tx);

        self.send_command(TurtleCommand::Request(request)).await;
    }

    async fn response(&mut self, response: Response) {
        if let Some(tx) = self.outstanding_requests.remove(&response.id) {
            tx.send(response.response);
        } else {
            warn!("Got response for unknown request {:?}", response);
        }
    }

    async fn send(&mut self, command: TurtleCommand) {
        if let Some(c) = self.sender_queue.send(command) {
            self.send_command(c).await;
        }
    }

    async fn ready(&mut self) {
        if let Some(c) = self.sender_queue.ready() {
            self.send_command(c).await;
        }
        // match self.sender_queue.ready() {
        //     Ok(Some(c)) => self.send_command(c).await,
        //     Err(_) => warn!("Got ready message when sender is already ready"),
        //     _ => {}
        // }
    }

    async fn ok(&mut self, id: u64) {
        if let Some(command) = &self.sent_command {
            if id != command.id {
                error!("Got ok message for unknown command");
                return;
            }

            self.sent_command = None;
        } else {
            error!("Got ok message without sending command");
        }
    }

    async fn send_command(&mut self, command: TurtleCommand) {
        // if self.sent_command.is_some() {
        //     error!("send_command called while there is still a command outstanding");
        //     return;
        // }

        let sent_command = SentCommand {
            id: self.next_id,
            command,
        };
        self.next_id += 1;
        self.send_message(
            serde_json::to_string(&sent_command).expect("Problem serializing command"),
        )
        .await;
        self.sent_command = Some(sent_command);

        self.command_timeout.reset();
    }

    async fn send_message(&mut self, message: String) {
        debug!("Sending message to {}: {message}", self.name);
        if let Err(e) = self.ws_sender.send(Message::Text(message)).await {
            error!("Problem sending message to {} {e}", self.name);
        }
    }
}
