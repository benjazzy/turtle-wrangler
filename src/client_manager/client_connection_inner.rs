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
    message_buffer: Option<Vec<u8>>,
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
            message_buffer: None,
            id,
        }
    }

    pub async fn run(mut self) {
        let mut close_tx = None;
        // let (mut reader, writer) = stream.split();
        // let mut reader = BufReader::new(reader);

        loop {
            let mut buffer = [0; 1024];

            debug!("{:?}", self.message_buffer);
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
        match &mut self.message_buffer {
            Some(b) => {
                b.extend_from_slice(data);
            }
            None => {
                let mut buffer = Vec::from(data);
                self.message_buffer = Some(buffer);
            }
        }

        while let Some(message) = self.parse_message() {
            debug!("Handling message {:?}", message);
            if message.is_empty() {
                continue;
            }

            match serde_json::from_slice(message.as_slice()) {
                Ok(r) => self.handle_request(r).await,
                Err(e) => {
                    error!("Problem parsing client message {e}");
                }
            }
        }

        // let message = match std::str::from_utf8(data) {
        //     Ok(m) => m,
        //     Err(e) => {
        //         error!("Problem converting bytes to string {e}");
        //         return;
        //     }
        // };
        // info!("Read {n} bytes. Message: {message}");
    }


    fn parse_message(&mut self) -> Option<Vec<u8>> {
        let data = match &mut self.message_buffer {
            Some(d) => d,
            None => return None,
        };

        const MARK_SIZE: usize = 4;
        const MARK: [u8; MARK_SIZE] = [0xa; MARK_SIZE];

        let pos = data.windows(MARK_SIZE).position(|d| { d == MARK })?;
        // let message = data.drain(0 .. pos).enumerate().filter(|(i, _)| *i < pos - MARK_SIZE).map(|(_, d)| d).collect();
        let message = data.drain(0 .. pos).collect();
        if data.len() >= MARK_SIZE {
            data.drain(0..=MARK_SIZE);
        }

        Some(message)
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

        if let Err(e) = self.stream.write(&serde_json::to_vec(&Reply::Turtles { turtles, }).unwrap()).await {
            error!("Problem serializing turtles reply {e}");
        };
    }
}
