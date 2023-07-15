use crate::{
    scheme::{Heading, Position},
    turtle_scheme::{RequestType, Response, ResponseType, TurtleCommand},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum TurtleSenderMessage {
    Request(RequestType, oneshot::Sender<ResponseType>),
    Response(Response),
    Close(oneshot::Sender<()>),
    GotOk(u64),
    Ready,
    Command(TurtleCommand),
    Message(String),
    Lock(
        mpsc::Receiver<LockedSenderMessage>,
        oneshot::Sender<Result<(), ()>>,
    ), // UpdatePosition(Position, oneshot::Sender<Result<(), ()>>),
}

#[derive(Debug)]
pub enum ReceiversSenderMessage {
    GotOk(u64),
    Ready,
    Response(Response),
}

#[derive(Debug)]
pub enum LockedSenderMessage {
    Request(RequestType, oneshot::Sender<ResponseType>),
    UpdatePosition(Position, Heading),
    Unlock,
}
