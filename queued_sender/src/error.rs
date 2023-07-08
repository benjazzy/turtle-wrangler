use thiserror::Error;

#[derive(Debug, Error)]
pub struct SendError<T>(pub T);

#[derive(Debug, Error)]
pub enum ReadyError<T> {
    #[error("there was a problem sending the message")]
    Send(SendError<T>),

    #[error("the sender was already ready")]
    AlreadyReady,

    // #[error("the sender is still waiting for an ok from the last message")]
    // Waiting,
}

#[derive(Debug, Error)]
pub enum OkError<T> {
    #[error("there was a problem sending the message")]
    Send(SendError<T>),
    
    
}
