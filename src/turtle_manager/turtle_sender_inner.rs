use crate::turtle_manager::TurtleSenderHandle;
use crate::turtle_scheme::TurtleCommand;
use futures_util::{stream::SplitSink, SinkExt};
use queued_sender::QueuedSender;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::MissedTickBehavior;
use tokio::{net::TcpStream, select, sync::mpsc, time};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, warn};

use super::{turtle_sender_message::TurtleSenderMessage, TurtleManagerHandle};

#[derive(Serialize)]
struct SentCommand {
    pub id: u64,
    pub command: TurtleCommand,
}

pub struct TurtleSenderInner {
    rx: mpsc::Receiver<TurtleSenderMessage>,
    ws_sender: QueuedSender<SplitSink<WebSocketStream<TcpStream>, Message>, Message>,
    manager: TurtleManagerHandle,

    outstanding_commands: HashMap<u64, TurtleCommand>,
    next_id: u64,

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
            ws_sender: QueuedSender::new(ws_sender),
            manager,
            name,
            to_send: VecDeque::new(),
            sent_command: None,
            command_timeout,
            next_id: 0,
            is_ready: false,
        }
    }

    pub async fn run(mut self) {
        debug!("Starting Turtle Sender");
        let mut close_tx = None;

        loop {
            select! {
                _ = self.command_timeout.tick(), if self.sent_command.is_some() => {
                    warn!("Never got ok from turtle {}", self.name);
                }
                message = self.rx.recv() => {
                    if let Some(message) = message {
                        match message {
                            TurtleSenderMessage::Close(tx) => {
                                close_tx = Some(tx);
                                break;
                            }
                            TurtleSenderMessage::Message(message) => {
                                self.send_message(message).await;
                            }
                            TurtleSenderMessage::Command(command) => {
                                self.send_command(command).await;
                            }
                            TurtleSenderMessage::GotOk(id) => {
                                self.got_ok(id).await;
                            }
                            TurtleSenderMessage::Ready => {
                                self.got_ready().await;
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

    async fn send_command(&mut self, command: TurtleCommand) {
        self.to_send.push_back(command);
        self.update_command_queue().await;
    }

    async fn update_command_queue(&mut self) {
        let should_send = self.sent_command.is_none() && self.is_ready;
        if !should_send {
            return;
        }

        if let Some(command) = self.to_send.pop_front() {
            let command = SentCommand {
                id: self.next_id,
                command,
            };
            let message = serde_json::to_string(&command).expect("Failed to serialize command");
            self.send_message(message).await;
            self.sent_command = Some(command);
            self.command_timeout.reset();
        }
    }

    async fn send_message(&mut self, message: String) {
        debug!("Sending message to {}: {message}", self.name);
        if let Err(e) = self.ws_sender.send(Message::Text(message)).await {
            error!("Problem sending message to {} {e}", self.name);
        }
    }

    async fn got_ok(&mut self, id: u64) {
        if let Some(sent_command) = &self.sent_command {
            if id != sent_command.id {
                warn!("Turtle sent ok with wrong id");
            }

            self.sent_command = None;
            self.update_command_queue();
        } else {
            warn!("Got ok message without sending command");
        }
    }

    async fn got_ready(&mut self) {
        self.is_ready = true;
        self.update_command_queue().await;
    }
}
