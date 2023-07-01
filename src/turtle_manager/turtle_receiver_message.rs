use tokio::sync::oneshot;

#[derive(Debug)]
pub enum TurtleReceiverMessage {
    Close(oneshot::Sender<()>),
}
