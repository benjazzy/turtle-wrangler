use crate::client_manager::client_connection_message::ClientConnectionMessage;
use crate::client_scheme::{Command, Event};
use crate::db::turtle_operations;
use crate::scheme::Direction;
use crate::turtle_manager::{ConnectionMessageType, TurtleConnectionMessage, TurtleManagerHandle};
use futures_util::sink::drain;
use sqlx::SqlitePool;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, Interest};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

pub struct ClientConnectionInner {
    rx: mpsc::Receiver<ClientConnectionMessage>,
    stream: TcpStream,
    turtle_manager: TurtleManagerHandle,
    pool: SqlitePool,
    // connection_manager: Connecti
    message_buffer: Vec<u8>,
    id: usize,
}

impl ClientConnectionInner {
    pub fn new(
        rx: mpsc::Receiver<ClientConnectionMessage>,
        stream: TcpStream,
        turtle_manager: TurtleManagerHandle,
        pool: SqlitePool,
        id: usize,
    ) -> Self {
        ClientConnectionInner {
            rx,
            stream,
            turtle_manager,
            pool,
            message_buffer: vec![],
            id,
        }
    }

    pub async fn run(mut self) {
        let mut close_tx = None;
        // let (mut reader, writer) = stream.split();
        // let mut reader = BufReader::new(reader);
        let (tx, mut turtle_event_rx) = mpsc::unbounded_channel();
        self.turtle_manager.client_subscribe(tx).await;

        loop {
            let mut buffer = [0; 1024];

            tokio::select! {
                message = self.rx.recv() => {
                    if let Some(message) = message {
                        match message {
                            ClientConnectionMessage::Close(tx) => {
                                close_tx = Some(tx);
                                break;
                            }
                        }
                    }
                }
                Some(message) = turtle_event_rx.recv() => {
                    self.handle_turtle_connection_message(message).await;
                }
                read_result = self.stream.read(&mut buffer) =>  {
                    match read_result {
                        Ok(n) if n == 0 => {
                            error!("Zero bytes");
                            break;
                        }
                        Ok(n) => {
                            self.read(&buffer, n).await;
                        }
                        Err(e) => {
                            error!("Problem reading from client stream {e}");
                            break;
                        }
                    }
                }
            }
        }

        self.stream.shutdown().await;
        debug!("Client connection closing");

        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    async fn read(&mut self, data: &[u8], n: usize) {
        let data = &data[0..n];
        self.message_buffer.extend_from_slice(data);

        let requests: Vec<Command> = turtle_tcp::parse_buffer(&mut self.message_buffer);

        for request in requests {
            self.handle_request(request).await;
        }
    }

    async fn handle_request(&mut self, request: Command) {
        match request {
            Command::GetTurtles => {
                debug!("Sending all turtles to client");
                self.send_turtles().await;
            }
            Command::Move { name, direction } => {
                debug!("Moving turtle {name} in direction {:?}", direction);
                self.move_turtle(name, direction).await;
            }
        }
    }

    async fn handle_turtle_connection_message<'a>(&mut self, message: TurtleConnectionMessage<'a>) {
        let name = message.name.to_string();
        match message.message_type {
            ConnectionMessageType::TurtleEvent(event) => {
                self.send_event(&Event::TurtleEvent { name, event }).await
            }
            ConnectionMessageType::Connected => {
                self.send_event(&Event::TurtleConnected { name }).await;
            }
            ConnectionMessageType::Disconnected => {
                self.send_event(&Event::TurtleDisconnected { name }).await;
            }
        }
    }

    async fn send_turtles(&mut self) {
        let turtles = match turtle_operations::get_turtles(&self.pool).await {
            Ok(t) => t,
            Err(e) => {
                error!("Problem getting turtles from database");
                return;
            }
        };

        self.send_event(&Event::Turtles { turtles }).await;
    }

    async fn send_event(&mut self, event: &Event) {
        let buffer = turtle_tcp::message_to_bytes(event).unwrap();

        if let Err(e) = self.stream.write(&buffer).await {
            error!("Problem sending event to client {e}");
        };
    }

    async fn move_turtle(&self, name: String, direction: Direction) {
        let turtle = match self.turtle_manager.get_turtle(name).await {
            Some(t) => t,
            None => return,
        };

        turtle.move_turtle(direction).await;
    }
}
