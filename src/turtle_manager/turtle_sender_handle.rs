use std::time::Duration;

use futures_util::stream::SplitSink;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::error;

use super::{
    turtle_sender_inner::TurtleSenderInner, turtle_sender_message::TurtleSenderMessage,
    TurtleManagerHandle,
};

#[derive(Debug)]
pub struct TurtleSenderHandle {
    tx: mpsc::Sender<TurtleSenderMessage>,
}

impl TurtleSenderHandle {
    pub fn new(
        ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleSenderInner::new(rx, ws_sender, manager, name);
        tokio::spawn(inner.run());

        TurtleSenderHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();

        if self.tx.send(TurtleSenderMessage::Close(tx)).await.is_err() {
            error!("Problem sending close message");
        }

        if tokio::time::timeout(Duration::from_millis(100), rx)
            .await
            .is_err()
        {
            error!("Timeout closing sender");
        }
    }

    pub async fn send(&self, message: String) {
        if let Err(m) = self.tx.send(TurtleSenderMessage::Message(message)).await {
            error!("Problem sending message {m}");
        }
    }
}
