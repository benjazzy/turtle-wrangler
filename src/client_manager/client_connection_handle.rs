use crate::client_manager::client_connection_inner::ClientConnectionInner;
use crate::client_manager::client_connection_message::ClientConnectionMessage;
use crate::turtle_manager::TurtleManagerHandle;
use sqlx::SqlitePool;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};

pub struct ClientConnectionHandle {
    tx: mpsc::Sender<ClientConnectionMessage>,
}

impl ClientConnectionHandle {
    pub fn new(
        stream: TcpStream,
        turtle_manager: TurtleManagerHandle,
        pool: SqlitePool,
        id: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = ClientConnectionInner::new(rx, stream, turtle_manager, pool, id);
        tokio::spawn(inner.run());

        ClientConnectionHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(ClientConnectionMessage::Close(tx))
            .await
            .is_ok()
        {
            let _ = rx.await;
        };
    }
}
