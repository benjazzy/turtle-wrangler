use futures_util::stream::SplitStream;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tracing::error;

use super::{
    turtle_receiver_inner::TurtleReceiverInner, turtle_receiver_message::TurtleReceiverMessage,
};

#[derive(Debug)]
pub struct TurtleReceiverHandle {
    tx: mpsc::Sender<TurtleReceiverMessage>,
}

impl TurtleReceiverHandle {
    pub fn new(ws_receiver: SplitStream<WebSocketStream<TcpStream>>, name: &'static str) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleReceiverInner::new(rx, ws_receiver, name);
        tokio::spawn(inner.run());

        TurtleReceiverHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();

        if let Err(_) = self.tx.send(TurtleReceiverMessage::Close(tx)).await {
            error!("Problem closing receiver");
        }

        let _ = rx.await;
    }
}
