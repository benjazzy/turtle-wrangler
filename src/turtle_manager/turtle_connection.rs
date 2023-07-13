use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

use crate::turtle_scheme::{RequestType, ResponseType, TurtleCommand};

use super::{TurtleManagerHandle, TurtleReceiverHandle, TurtleSenderHandle};

/// Contains both the sender and receiver for a turtle websocket connection.
#[derive(Debug, Clone)]
pub struct TurtleConnection {
    /// Receiver of the turtle websocket.
    receiver: TurtleReceiverHandle,

    /// Sender of the turtle websocket.
    sender: TurtleSenderHandle,
}

impl TurtleConnection {
    /// Splits a WebSocketStream and creates a TurtleSender and TurtleReceiver from it.
    ///
    /// # Arguments
    /// * `ws_connection` - WebSocket of the connected turtle.
    /// * `manager` - TurtleManagerHandle so that the sender and receiver can notify a close.
    /// * `name` - Name of the turtle.
    pub fn new(
        ws_connection: WebSocketStream<TcpStream>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let (ws_sender, ws_receiver) = ws_connection.split();

        let sender = TurtleSenderHandle::new(ws_sender, manager.clone(), name);
        let receiver = TurtleReceiverHandle::new(ws_receiver, manager, sender.clone(), name);

        TurtleConnection { receiver, sender }
    }

    /// Send a message to the connected turtle
    ///
    /// # Arguments
    /// * `message` - The message to send.
    pub async fn send(&self, command: TurtleCommand) {
        self.sender.send(command).await;
    }

    pub async fn request(&self, request: RequestType) -> Result<ResponseType, ()> {
        self.sender.request(request).await
    }

    /// Closes both the sender and receiver.
    pub async fn close(&self) {
        self.receiver.close().await;
        self.sender.close().await;
    }
}
