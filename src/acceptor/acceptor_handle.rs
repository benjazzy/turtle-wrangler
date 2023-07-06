use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::turtle_manager::TurtleManagerHandle;

use super::{acceptor_inner::AcceptorInner, acceptor_message::AcceptorMessage};

/// Handle for communicating with AcceptorInner.
/// The Acceptor listens for tcp connections and upgrades them to websockets.
/// It then sends those websockets to a turtle_manager.
pub struct AcceptorHandle {
    /// Sender to send messages to AcceptorInner.
    tx: mpsc::Sender<AcceptorMessage>,
}

impl AcceptorHandle {
    /// Creates a new AcceptorHandle and AcceptorInner and starts the AcceptorInner.
    ///
    /// # Arguments
    /// * `addr` - Address to listen for tcp connections on.
    /// * `turtle_manager` - TurtleManager to send websockets on to.
    pub fn new(addr: String, turtle_manager: TurtleManagerHandle) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = AcceptorInner::new(rx, turtle_manager);
        tokio::spawn(inner.run(addr));

        AcceptorHandle { tx }
    }

    /// Sends a close message to AcceptorInner and waits for the AcceptorInner to close.
    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();

        // Send the close message
        if let Err(e) = self.tx.send(AcceptorMessage::Close(tx)).await {
            error!("Error closing listener: {e}");
        }

        // Wait for the AcceptorInner to close.
        let _ = rx.await;
    }
}
