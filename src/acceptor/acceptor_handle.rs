use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tracing::error;

use crate::turtle_manager::{TurtleReceiverHandler, TurtleSenderHandler};

use super::{acceptor_inner::AcceptorInner, acceptor_message::AcceptorMessage};

pub struct AcceptorHandle {
    tx: mpsc::Sender<AcceptorMessage>,
}

impl AcceptorHandle {
    pub fn new(
        addr: String,
        senders: Arc<Mutex<Vec<TurtleSenderHandler>>>,
        receivers: Arc<Mutex<Vec<TurtleReceiverHandler>>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = AcceptorInner::new(rx, senders, receivers);
        tokio::spawn(inner.run(addr));

        AcceptorHandle { tx }
    }

    pub async fn close(&self) {
        if let Err(e) = self.tx.send(AcceptorMessage::Close).await {
            error!("Error closing listener: {e}");
        }
    }
}
