use futures_util::stream::SplitSink;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::error;

use super::{turtle_sender_inner::TurtleSenderInner, turtle_sender_message::TurtleSenderMessage};

pub struct TurtleSenderHandler {
    tx: mpsc::Sender<TurtleSenderMessage>,
}

impl TurtleSenderHandler {
    pub fn new(
        ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        name: &'static str,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleSenderInner::new(rx, ws_sender, name);
        tokio::spawn(inner.run());

        TurtleSenderHandler { tx }
    }

    pub async fn close(&self) {
        if let Err(_) = self.tx.send(TurtleSenderMessage::Close).await {
            error!("Problem sending close message");
        }
    }

    pub async fn send(&self, message: String) {
        if let Err(m) = self.tx.send(TurtleSenderMessage::Message(message)).await {
            error!("Problem sending message {m}");
        }
    }
}
