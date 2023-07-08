use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub struct SendError<T>(pub T);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReadyError<T> {
    #[error("there was a problem sending the message")]
    Send(SendError<T>),

    #[error("the sender was already ready")]
    AlreadyReady,
}

// #[derive(Debug, Error, PartialEq, Eq)]
// pub enum OkError<T> {
//     #[error("there was a problem sending the message")]
//     Send(SendError<T>),
// }
