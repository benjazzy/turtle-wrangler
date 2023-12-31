use crate::turtle_manager::TurtleConnectionMessage;
use crate::turtle_scheme::TurtleEvents;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum TurtleReceiverMessage {
    Close(oneshot::Sender<()>),
    ClientSubscribe(mpsc::UnboundedSender<TurtleConnectionMessage<'static>>),
}
