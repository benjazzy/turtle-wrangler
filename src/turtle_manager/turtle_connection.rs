use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

use super::{TurtleManagerHandle, TurtleReceiverHandle, TurtleSenderHandle};

#[derive(Debug)]
pub struct TurtleConnection {
    receiver: TurtleReceiverHandle,
    sender: TurtleSenderHandle,
}

impl TurtleConnection {
    pub fn new(
        ws_connection: WebSocketStream<TcpStream>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let (ws_sender, ws_receiver) = ws_connection.split();

        let receiver = TurtleReceiverHandle::new(ws_receiver, manager.clone(), name);
        let sender = TurtleSenderHandle::new(ws_sender, manager.clone(), name);

        TurtleConnection { receiver, sender }
    }

    pub async fn send(&self, message: String) {
        self.sender.send(message).await;
    }

    pub async fn close(&self) {
        self.receiver.close().await;
        self.sender.close().await;
    }
}
