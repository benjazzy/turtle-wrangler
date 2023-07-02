use tokio::sync::{mpsc, oneshot};
use tracing::error;

use super::{
    turtle_manager_inner::TurtleManagerInner, turtle_manager_message::TurtleManagerMessage,
    unknown_turtle_connection::UnknownTurtleConnection,
};

#[derive(Debug, Clone)]
pub struct TurtleManagerHandle {
    tx: mpsc::Sender<TurtleManagerMessage>,
}

impl TurtleManagerHandle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleManagerInner::new(rx);
        tokio::spawn(inner.run());

        TurtleManagerHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx.send(TurtleManagerMessage::Close(tx)).await;

        let _ = rx.await;
    }

    pub async fn new_unknown_turtle(&self, turtle: UnknownTurtleConnection) {
        if let Err(_) = self
            .tx
            .send(TurtleManagerMessage::UnknownTurtle(turtle))
            .await
        {
            error!("Problem sending new turtle to turtle manager");
        }
    }

    pub async fn broadcast(&self, message: String) {
        if let Err(_) = self.tx.send(TurtleManagerMessage::Broadcast(message)).await {
            error!("Problem sending broadcast message to turtle manager");
        }
    }
}
