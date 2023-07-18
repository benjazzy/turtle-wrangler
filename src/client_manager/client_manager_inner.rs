use tokio::sync::mpsc;
use tracing::debug;
use crate::client_manager::client_connection_handle::ClientConnectionHandle;
use crate::client_manager::client_manager_message::ClientManagerMessage;

pub struct ClientManagerInner {
    rx: mpsc::Receiver<ClientManagerMessage>,
    clients: Vec<ClientConnectionHandle>,
}

impl ClientManagerInner {
    pub fn new(rx: mpsc::Receiver<ClientManagerMessage>) -> Self {
        ClientManagerInner { rx, clients: vec![] }
    }

    pub async fn run(mut self) {
        let mut close_tx = None;

        while let Some(message) = self.rx.recv().await {
            match message {
                ClientManagerMessage::Close(tx) => {
                    close_tx = Some(tx);
                    break;
                }
                ClientManagerMessage::NewClient(client) => {
                    self.clients.push(client);
                }
            }
        }

        for client in self.clients.iter() {
            client.close().await;
        }

        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }

        debug!("Client manager closing");
    }
}
