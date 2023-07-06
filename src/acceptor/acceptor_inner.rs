use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use tracing::{debug, info};

use crate::turtle_manager::{TurtleManagerHandle, UnknownTurtleConnection};

use super::acceptor_message::AcceptorMessage;

/// Listens for messages from its handles and tcp connections.
/// Upgrades the tcp connections and sends them to the TurtleManager.
pub struct AcceptorInner {
    /// Receives messages from our handles.
    rx: mpsc::Receiver<AcceptorMessage>,

    /// TurtleManager to send new websockets to.
    turtle_manager: TurtleManagerHandle,
}

impl AcceptorInner {
    /// Creates a new AcceptorInner.
    /// Meant to be run by AcceptorHandle.
    pub fn new(rx: mpsc::Receiver<AcceptorMessage>, turtle_manager: TurtleManagerHandle) -> Self {
        AcceptorInner { rx, turtle_manager }
    }

    /// Starts the AcceptorInner.
    /// Takes owernship of self
    /// Meant to be run by AcceptorHandle.
    pub async fn run(mut self, addr: String) {
        // If we receive a close message from a handle then close_tx will contain a Sender to
        // report when we have closed. If we close for any other reason then close_tx will stay
        // None
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
                        // If we get None from self.rx then all handles have been dropped and we
                        // should exit.
                        break;
                    }
                }

                connection = listener.accept() => {
                    if let Ok((stream, _)) = connection {
                        self.accept(stream).await;
                    } else {
                        // If our listener errors then we should exit.
                        break;
                    }
                }
            }
        }

        info!("Closing acceptor");

        // If we got a close message from a handle then report when we have stopped.
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    /// Upgrades a tcp stream to a websockt and sends it on to our TurtleManager.
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
