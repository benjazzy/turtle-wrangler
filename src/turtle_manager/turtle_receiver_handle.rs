use std::time::Duration;

use futures_util::stream::SplitStream;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tracing::error;

use super::{
    turtle_receiver_inner::TurtleReceiverInner, turtle_receiver_message::TurtleReceiverMessage,
    TurtleManagerHandle,
};

#[derive(Debug)]
pub struct TurtleReceiverHandle {
    tx: mpsc::Sender<TurtleReceiverMessage>,
}

impl TurtleReceiverHandle {
    pub fn new(
        ws_receiver: SplitStream<WebSocketStream<TcpStream>>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleReceiverInner::new(rx, ws_receiver, manager, name);
        tokio::spawn(inner.run());

        TurtleReceiverHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();

        if let Err(_) = self.tx.send(TurtleReceiverMessage::Close(tx)).await {
            error!("Problem closing receiver");
        }

        if tokio::time::timeout(Duration::from_millis(100), rx)
            .await
            .is_err()
        {
            error!("Timeout closing receiver");
        }
    }
}
