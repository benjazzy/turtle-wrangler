use crate::{
    scheme::{Coordinates, Heading},
    turtle_scheme::{RequestType, Response, ResponseType, TurtleCommand},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum TurtleSenderMessage {
    Request(RequestType, oneshot::Sender<ResponseType>),
    Close(oneshot::Sender<()>),
    Command(TurtleCommand),
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
    UpdatePosition(Coordinates, Heading),
    Unlock,
}
