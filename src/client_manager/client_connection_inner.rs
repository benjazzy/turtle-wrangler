use futures_util::sink::drain;
use sqlx::SqlitePool;
use tokio::io;
use crate::client_manager::client_connection_message::ClientConnectionMessage;
use crate::turtle_manager::TurtleManagerHandle;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, Interest};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use crate::client_scheme::{Reply, Request};
use crate::db::turtle_operations;

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
        let data = &data[0 .. n];
        self.message_buffer.extend_from_slice(data);

        let requests: Vec<Request> = turtle_tcp::parse_buffer(&mut self.message_buffer);

        for request in requests {
            self.handle_request(request).await;
        }
    }


    async fn handle_request(&mut self, request: Request) {
        match request {
            Request::GetTurtles => {
                debug!("Sending all turtles to client");
                self.send_turtles().await;
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

        let buffer = turtle_tcp::message_to_bytes(&Reply::Turtles { turtles }).unwrap();

        if let Err(e) = self.stream.write(&buffer).await {
            error!("Problem serializing turtles reply {e}");
        };
    }
}
