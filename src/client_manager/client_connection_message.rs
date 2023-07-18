use tokio::sync::oneshot;

pub enum ClientConnectionMessage {
    Close(oneshot::Sender<()>),
}
