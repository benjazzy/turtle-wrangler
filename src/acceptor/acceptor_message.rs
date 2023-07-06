use tokio::sync::oneshot;

/// Messages that can be sent by an AcceptorHandle to its AcceptorInner.
#[derive(Debug)]
pub enum AcceptorMessage {
    /// Tells the AcceptorInner to stop listening for connections and exit.
    /// Contains a channel for the AcceptorInner to send a message on when it is closed.
    Close(oneshot::Sender<()>),
}
