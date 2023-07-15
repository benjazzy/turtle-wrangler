use crate::turtle_scheme::{Request, RequestType, Response, ResponseType, TurtleCommand};
use futures_util::{stream::SplitSink, SinkExt, Sink};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::MissedTickBehavior;
use tokio::{net::TcpStream, select, sync::mpsc, time};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};
use turtle_sender_queue::SenderQueue;

use super::turtle_sender_message::{LockedSenderMessage, ReceiversSenderMessage};
use super::{turtle_sender_message::TurtleSenderMessage, TurtleManagerHandle};

#[derive(Serialize)]
struct SentCommand {
    pub id: u64,
    pub command: TurtleCommand,
}

pub struct TurtleSenderInner {
    rx: mpsc::Receiver<TurtleSenderMessage>,
    receiver_rx: mpsc::Receiver<ReceiversSenderMessage>,
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
        receiver_rx: mpsc::Receiver<ReceiversSenderMessage>,
        ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let mut command_timeout = time::interval(Duration::from_secs(5));
        command_timeout.set_missed_tick_behavior(MissedTickBehavior::Skip);

        TurtleSenderInner {
            rx,
            receiver_rx,
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
        let mut lock_queue: VecDeque<(
            mpsc::Receiver<LockedSenderMessage>,
            oneshot::Sender<Result<(), ()>>,
        )> = VecDeque::new();

        loop {
            if self.sent_command.is_none() {
                if let Some((rx, tx)) = lock_queue.pop_front() {
                    let locked_sender = LockedSender::new(
                        rx,
                        &mut self.receiver_rx,
                        &mut self.ws_sender,
                        &mut self.next_id,
                        self.sender_queue.is_ready(),
                        self.name,
                    );
                    locked_sender.run(tx).await;
                }
            }

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
                            TurtleSenderMessage::Lock(rx, tx) => lock_queue.push_back((rx, tx)),
                        }
                    } else {
                        self.manager.disconnect(self.name).await;
                        break;
                    }
                }
                message = self.receiver_rx.recv() => {
                    if let Some(message) = message {
                        self.handle_receiver_message(message).await;
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

    async fn handle_receiver_message(&mut self, message: ReceiversSenderMessage) {
        match message {
            ReceiversSenderMessage::GotOk(id) => self.ok(id).await,
            ReceiversSenderMessage::Ready => self.ready().await,
            ReceiversSenderMessage::Response(response) => self.response(response).await,
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

        self.send(TurtleCommand::Request(request)).await;
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

struct LockedSender<'a> {
    rx: mpsc::Receiver<LockedSenderMessage>,
    receiver_rx: &'a mut mpsc::Receiver<ReceiversSenderMessage>,
    ws_sender: &'a mut SplitSink<WebSocketStream<TcpStream>, Message>,
    sent_command: Option<SentCommand>,
    sender_queue: SenderQueue<TurtleCommand>,
    command_timeout: time::Interval,
    next_id: &'a mut u64,
    outstanding_requests: HashMap<u64, oneshot::Sender<ResponseType>>,
    name: &'a str,
}

impl<'a> LockedSender<'a> {
    pub fn new(
        rx: mpsc::Receiver<LockedSenderMessage>,
        receiver_rx: &'a mut mpsc::Receiver<ReceiversSenderMessage>,
        ws_sender: &'a mut SplitSink<WebSocketStream<TcpStream>, Message>,
        next_id: &'a mut u64,
        ready: bool,
        name: &'a str,
    ) -> Self {
        let mut sender_queue = SenderQueue::new();
        if ready {
            sender_queue.ready();
        }

        LockedSender {
            rx,
            receiver_rx,
            ws_sender,
            sent_command: None,
            sender_queue,
            command_timeout: time::interval(Duration::from_secs(10)),
            next_id,
            outstanding_requests: HashMap::new(),
            name,
        }
    }

    pub async fn run(mut self, tx: oneshot::Sender<Result<(), ()>>) {
        debug!("Sender for {} is locking", self.name);

        let (ping_tx, ping_rx) = oneshot::channel();
        self.request(RequestType::Ping, ping_tx).await;

        tokio::spawn(async move {
            let response = ping_rx.await;
            if let Ok(response) = response {
                if !matches!(response, ResponseType::Pong) {
                    error!("Got incorrect response type to ping :{:?}", response);
                    let _ = tx.send(Err(()));
                    return;
                }
            } else {
                let _ = tx.send(Err(()));
                return;
            }

            let _ = tx.send(Ok(()));
        });

        debug!("Starting lock loop");

        let start_time = tokio::time::Instant::now();
        let mut should_exit = false;

        loop {
            let timeout = tokio::time::sleep(Duration::from_secs(10) - start_time.elapsed());
            tokio::select! {
                _ = timeout => {
                    warn!("Timeout during lock");
                    break;
                }
                message = self.rx.recv(), if !should_exit => {
                    if let Some(message) = message {
                        match message {
                            LockedSenderMessage::Request(request, tx) => {
                                self.request(request, tx).await;
                            }
                            LockedSenderMessage::UpdatePosition(position, heading) => {
                                info!("Updating position of {}", self.name);
                                self.send(TurtleCommand::UpdatePosition { coords: position, heading }).await;
                            }
                            LockedSenderMessage::Unlock => {
                                should_exit = true;
                            }
                        }
                    } else {
                        should_exit = true;
                    }
                }
                message = self.receiver_rx.recv() => {
                    if let Some(message) = message {
                        self.handle_receiver_message(message).await;
                    } else {
                        break;
                    }
                }

            }

            if should_exit && self.sent_command.is_none() && self.sender_queue.is_empty() {
                break;
            }
        }

        debug!("Sender for {} is unlocking", self.name);
    }

    async fn handle_receiver_message(&mut self, message: ReceiversSenderMessage) {
        match message {
            ReceiversSenderMessage::GotOk(id) => self.ok(id).await,
            ReceiversSenderMessage::Ready => self.ready().await,
            ReceiversSenderMessage::Response(response) => self.response(response).await,
        }
    }

    async fn request(&mut self, request_type: RequestType, tx: oneshot::Sender<ResponseType>) {
        let id = *self.next_id;
        *self.next_id += 1;
        let request = Request {
            id,
            request: request_type,
        };

        self.outstanding_requests.insert(id, tx);

        self.send(TurtleCommand::Request(request)).await;
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
            id: *self.next_id,
            command,
        };
        *self.next_id += 1;
        // self.send_message(
        //     serde_json::to_string(&sent_command).expect("Problem serializing command"),
        // )
        send_message(
            self.ws_sender,
            self.name,
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

async fn send_message(mut ws_sender: impl Sink<Message>, name: &str, message: String) {
    debug!("Sending message to {}: {message}", name);
    if let Err(e) = ws_sender.send(Message::Text(message)).await {
        error!("Problem sending message to {} {e}", self.name);
    }
}