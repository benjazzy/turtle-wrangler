pub mod error;

use std::collections::VecDeque;
use futures_util::SinkExt;
use thiserror::Error;
use error::{ReadyError, SendError};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SenderState {
    Ready,
    Sending,
    // Waiting,
}

#[derive(Debug)]
pub struct QueuedSender<S, M>
where
    S: SinkExt<M>,
{
    sender: S,
    queue: VecDeque<M>,
    state: SenderState,
}

impl<S, M> QueuedSender<S, M>
where
    S: SinkExt<M>,
{
    pub fn new(sender: S) -> Self {
        QueuedSender {
            sender,
            queue: VecDeque::new(),
            state: SenderState::Ready,
        }
    }

    pub fn get_state(&self) -> &SenderState {
        &self.state
    }
}

impl<S, M> QueuedSender<S, M>
    where
        S: SinkExt<M> + Unpin,
{
    pub async fn send(&mut self, item: M) -> Result<(), SendError<S::Error>> {
        self.queue.push_back(item);

        self.update_queue().await
    }

    pub async fn ready(&mut self) -> Result<(), ReadyError<S::Error>> {
        match self.state {
            SenderState::Ready => Err(ReadyError::AlreadyReady),
            // SenderState::Sending => Err(ReadyError::Waiting),
            SenderState::Sending => {
                self.state = SenderState::Ready;

                self.update_queue().await.map_err(ReadyError::Send)
            }
        }
    }

    async fn update_queue(&mut self) -> Result<(), SendError<S::Error>> {
        if let SenderState::Ready = self.state {
            if let Some(message) = self.queue.pop_front() {
                return self.sender.send(message).await.map_err(SendError);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};
    use tokio::sync::Mutex;
    use futures_util::Sink;
    use super::*;

    enum MockSinkInner {
        Ready
    }
    struct MockSink<'a, T>(&'a mut Vec<T>);

    impl<'a, T> Sink<T> for MockSink<'a, T> {
        type Error = ();

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
            (*self).0.push(item);

            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn check_send_and_ready() {
        let message = "test_message";
        let mut out: Vec<&'static str> = vec![];

        let mut sender = QueuedSender::new(MockSink(&mut out));
        assert_eq!(sender.state, SenderState::Ready);

        sender.send(message).await.expect("Problem sending message");
        assert_eq!(sender.state, SenderState::Sending);

        assert_eq!(out.len(), 1);

    }
}
