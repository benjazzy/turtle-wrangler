use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use tracing::{debug, info};

use crate::turtle_manager::{TurtleManagerHandle, UnknownTurtleConnection};

use super::acceptor_message::AcceptorMessage;

pub struct AcceptorInner {
    rx: mpsc::Receiver<AcceptorMessage>,
    turtle_manager: TurtleManagerHandle,
}

impl AcceptorInner {
    pub fn new(rx: mpsc::Receiver<AcceptorMessage>, turtle_manager: TurtleManagerHandle) -> Self {
        AcceptorInner { rx, turtle_manager }
    }

    pub async fn run(mut self, addr: String) {
        let mut close_tx = None;

        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        info!("Listening for connections at {addr}");
        loop {
            tokio::select! {
                message = self.rx.recv() => {
                    if let Some(message) = message {
                        info!("Got message {:?}", message);
                        match message {
                            AcceptorMessage::Close(tx) => {
                                close_tx = Some(tx);
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                connection = listener.accept() => {
                    if let Ok((stream, _)) = connection {
                        self.accept(stream).await;
                    } else {
                        break;
                    }
                }
            }
        }

        info!("Closing acceptor");
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    async fn accept(&self, stream: TcpStream) {
        debug!(
            "Got connection {}",
            stream
                .peer_addr()
                .expect("Could not get peer address from stream")
        );
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let unknown_turtle = UnknownTurtleConnection::new(ws_stream);
        self.turtle_manager.new_unknown_turtle(unknown_turtle).await;
    }
}
