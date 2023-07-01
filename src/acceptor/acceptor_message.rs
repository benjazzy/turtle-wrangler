use tokio::sync::oneshot;

#[derive(Debug)]
pub enum AcceptorMessage {
    Close(oneshot::Sender<()>),
}
