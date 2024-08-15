use std::time::Duration;

use crate::turtle_manager::TurtleConnectionMessage;
use crate::turtle_scheme::TurtleEvents;
use futures_util::stream::SplitStream;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tracing::error;

use super::{
    turtle_receiver_inner::TurtleReceiverInner, turtle_receiver_message::TurtleReceiverMessage,
    turtle_sender_handle::ReceiversSenderHandle, TurtleManagerHandle,
};

/// Communicates with a TurtleReceiverInner which listens for messages from turtles and forwards
/// them to where they need to go.
#[derive(Debug, Clone)]
pub struct TurtleReceiverHandle {
    /// Sender to send messages to a TurtleReceiverInner.
    tx: mpsc::Sender<TurtleReceiverMessage>,
}

impl TurtleReceiverHandle {
    /// Creates a new TurtleReceiverInner and start it.
    /// Returns a TurtleReceiverHandle connected to the TurtleReceiverInner.
    ///
    /// # Arguments
    ///
    /// * `ws_receiver` - Passed on to TurtleReceiverInner to listen for websocket messages.
    /// * `manager` - Passed on to TurtleReceiverInner to notify when the receiver has closed.
    /// * `sender` - Handle if of the sender connected to our turtle. Used to pass on ok and ready.
    /// * `name` - Name of the turtle for logging purposes.
    pub fn new(
        ws_receiver: SplitStream<WebSocketStream<TcpStream>>,
        manager: TurtleManagerHandle,
        sender: ReceiversSenderHandle,
        name: &'static str,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleReceiverInner::new(rx, ws_receiver, manager, sender, name);
        tokio::spawn(inner.run());

        TurtleReceiverHandle { tx }
    }

    /// Tells the TurtleReceiverInner to close and waits for it to do so.
    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();

        if self
            .tx
            .send(TurtleReceiverMessage::Close(tx))
            .await
            .is_err()
        {
            error!("Problem closing receiver");
        }

        if tokio::time::timeout(Duration::from_millis(100), rx)
            .await
            .is_err()
        {
            error!("Timeout closing receiver");
        }
    }

    pub async fn client_subscribe(
        &self,
        tx: mpsc::UnboundedSender<TurtleConnectionMessage<'static>>,
    ) {
        if self
            .tx
            .send(TurtleReceiverMessage::ClientSubscribe(tx))
            .await
            .is_err()
        {
            error!("Problem sending subscribe message to receiver");
        }
    }
}
