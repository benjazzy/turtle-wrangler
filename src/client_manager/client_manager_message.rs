use tokio::sync::oneshot;
use crate::client_manager::client_connection_handle::ClientConnectionHandle;

pub enum ClientManagerMessage {
    Close(oneshot::Sender<()>),
    NewClient(ClientConnectionHandle),
}
