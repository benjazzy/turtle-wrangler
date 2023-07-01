use futures_util::{stream::SplitStream, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::WebSocketStream;
use tracing::debug;

use super::turtle_receiver_message::TurtleReceiverMessage;

pub struct TurtleReceiverInner {
    rx: mpsc::Receiver<TurtleReceiverMessage>,
    ws_receiver: SplitStream<WebSocketStream<TcpStream>>,

    name: &'static str,
}

impl TurtleReceiverInner {
    pub fn new(
        rx: mpsc::Receiver<TurtleReceiverMessage>,
        ws_receiver: SplitStream<WebSocketStream<TcpStream>>,
        name: &'static str,
    ) -> Self {
        TurtleReceiverInner {
            rx,
            ws_receiver,
            name,
        }
    }

    pub async fn run(mut self) {
        let mut close_tx = None;

        loop {
            tokio::select! {
                message = self.ws_receiver.next() => {
                    if let Some(Ok(message)) = message {
                        debug!("Got message {message} from {}", self.name)
                    } else {
                        break;
                    }
                }

                message = self.rx.recv() => {
                    if let Some(message) = message {
                        match message {
                            TurtleReceiverMessage::Close(tx) => {
                                close_tx = Some(tx);
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        debug!("Turtle Receiver shutting down for {}", self.name);
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }
}
