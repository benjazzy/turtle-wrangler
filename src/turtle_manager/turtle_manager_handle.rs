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

        let inner = TurtleManagerInner::new(rx, TurtleManagerHandle { tx: tx.clone() });
        tokio::spawn(inner.run());

        TurtleManagerHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(TurtleManagerMessage::Close(tx)).await.is_err() {
            error!("Problem closing turtle manager");
        }

        let _ = rx.await;
    }

    pub async fn new_unknown_turtle(&self, turtle: UnknownTurtleConnection) {
        if self
            .tx
            .send(TurtleManagerMessage::UnknownTurtle(turtle))
            .await
            .is_err()
        {
            error!("Problem sending new turtle to turtle manager");
        }
    }

    pub async fn disconnect(&self, name: impl Into<String>) {
        if self
            .tx
            .send(TurtleManagerMessage::Disconnnect(name.into()))
            .await
            .is_err()
        {
            error!("Problem sending disconnect to turtle manager");
        }
    }

    pub async fn broadcast(&self, message: String) {
        if self
            .tx
            .send(TurtleManagerMessage::Broadcast(message))
            .await
            .is_err()
        {
            error!("Problem sending broadcast message to turtle manager");
        }
    }

    pub async fn get_status(&self) -> Option<String> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(TurtleManagerMessage::Status(tx))
            .await
            .is_err()
        {
            error!("Problem sending list message to turtle manager");
        }

        rx.await.ok()
    }
}
