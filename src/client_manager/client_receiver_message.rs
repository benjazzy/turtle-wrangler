use tokio::sync::oneshot;

pub enum ClientReceiverMessage {
    Close(oneshot::Sender<()>),
}
