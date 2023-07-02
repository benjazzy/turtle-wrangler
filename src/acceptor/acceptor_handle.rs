

use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::turtle_manager::{TurtleManagerHandle};

use super::{acceptor_inner::AcceptorInner, acceptor_message::AcceptorMessage};

pub struct AcceptorHandle {
    tx: mpsc::Sender<AcceptorMessage>,
}

impl AcceptorHandle {
    pub fn new(addr: String, turtle_manager: TurtleManagerHandle) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = AcceptorInner::new(rx, turtle_manager);
        tokio::spawn(inner.run(addr));

        AcceptorHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();
        if let Err(e) = self.tx.send(AcceptorMessage::Close(tx)).await {
            error!("Error closing listener: {e}");
        }

        let _ = rx.await;
    }
}
