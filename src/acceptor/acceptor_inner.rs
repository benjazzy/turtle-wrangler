use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use crate::acceptor::tcp_handler::TcpHandler;
use tracing::{debug, info};

use crate::turtle_manager::{TurtleManagerHandle, UnknownTurtleConnection};

use super::acceptor_message::AcceptorMessage;

/// Listens for messages from its handles and tcp connections.
/// Calls handler.handle_tcp() with the connection.
pub struct AcceptorInner<H>
where
    H: TcpHandler,
{
    /// Receives messages from our handles.
    rx: mpsc::Receiver<AcceptorMessage>,

    handler: H,
}

impl<H> AcceptorInner<H>
where
    H: TcpHandler,
{
    /// Creates a new AcceptorInner.
    /// Meant to be run by AcceptorHandle.
    pub fn new(rx: mpsc::Receiver<AcceptorMessage>, handler: H) -> Self {
        AcceptorInner { rx, handler }
    }

    /// Starts the AcceptorInner.
    /// Takes ownership of self
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

    /// Upgrades a tcp stream to a websocket and sends it on to our TurtleManager.
    async fn accept(&mut self, stream: TcpStream) {
        debug!(
            "Got connection {}",
            stream
                .peer_addr()
                .expect("Could not get peer address from stream")
        );
        self.handler.handle_tcp(stream).await;
    }
}
