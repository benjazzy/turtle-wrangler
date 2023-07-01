use tokio::sync::oneshot;

#[derive(Debug)]
pub enum TurtleSenderMessage {
    Close(oneshot::Sender<()>),
    Message(String),
}
