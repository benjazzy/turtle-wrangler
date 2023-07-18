use tokio::sync::{mpsc, oneshot};
use tracing::error;
use crate::client_manager::client_manager_inner::ClientManagerInner;
use crate::client_manager::client_manager_message::ClientManagerMessage;
use crate::client_manager::ClientConnectionHandle;

#[derive(Debug, Clone)]
pub struct ClientManagerHandle {
    tx: mpsc::Sender<ClientManagerMessage>,
}

impl ClientManagerHandle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        
        let inner = ClientManagerInner::new(rx);
        tokio::spawn(inner.run());
        
        ClientManagerHandle { tx }
    }
    
    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();
        
        if self.tx.send(ClientManagerMessage::Close(tx)).await.is_ok() {
            let _ = rx.await;
        }
        
    }
    
    pub async fn new_client(&self, client: ClientConnectionHandle) {
        if self.tx.send(ClientManagerMessage::NewClient(client)).await.is_err() {
            error!("Problem sending new client to client manager");
        }
    }
}
