use crate::turtle_scheme::{RequestType, Response, ResponseType, TurtleCommand};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum TurtleSenderMessage {
    Request(RequestType, oneshot::Sender<ResponseType>),
    Response(Response),
    Close(oneshot::Sender<()>),
    GotOk(u64),
    Ready,
    Command(TurtleCommand),
    Message(String),
}
