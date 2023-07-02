use futures_util::{stream::SplitSink, SinkExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error};

use super::turtle_sender_message::TurtleSenderMessage;

pub struct TurtleSenderInner {
    rx: mpsc::Receiver<TurtleSenderMessage>,
    ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,

    name: &'static str,
}

impl TurtleSenderInner {
    pub fn new(
        rx: mpsc::Receiver<TurtleSenderMessage>,
        ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        name: &'static str,
    ) -> Self {
        TurtleSenderInner {
            rx,
            ws_sender,
            name,
        }
    }

    pub async fn run(mut self) {
        debug!("Starting Turtle Sender");
        let mut close_tx = None;

        loop {
            if let Some(message) = self.rx.recv().await {
                match message {
                    TurtleSenderMessage::Close(tx) => {
                        close_tx = Some(tx);
                        break;
                    }
                    TurtleSenderMessage::Message(message) => {
                        debug!("Sending message to {}: {message}", self.name);
                        if let Err(e) = self.ws_sender.send(Message::Text(message)).await {
                            error!("Problem sending message to {} {e}", self.name);
                        }
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
}
