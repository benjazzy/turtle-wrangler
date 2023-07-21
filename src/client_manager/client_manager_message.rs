use crate::client_manager::client_connection_handle::ClientConnectionHandle;
use tokio::sync::oneshot;

pub enum ClientManagerMessage {
    Close(oneshot::Sender<()>),
    NewClient(ClientConnectionHandle),
}
