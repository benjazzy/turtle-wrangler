use tokio::sync::oneshot;
use crate::turtle_scheme::TurtleCommand;

#[derive(Debug)]
pub enum TurtleSenderMessage {
    Close(oneshot::Sender<()>),
    GotOk(u64),
    Ready,
    Command(TurtleCommand),
    Message(String),
}
