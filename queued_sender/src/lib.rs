pub mod error;

use error::{ReadyError, SendError};
use futures_util::SinkExt;
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SenderState {
    Ready,
    Sending,
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
    pub async fn close(&mut self) -> Result<(), S::Error> {
        self.sender.close().await
    }

    pub async fn send(&mut self, item: M) -> Result<(), SendError<S::Error>> {
        self.queue.push_back(item);

        if let SenderState::Ready = self.state {
            self.update_queue().await
        } else {
            Ok(())
        }
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
                self.state = SenderState::Sending;
                return self.sender.send(message).await.map_err(SendError);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_start_ready() {
        let mut out: Vec<&'static str> = vec![];

        let sender = QueuedSender::new(&mut out);
        assert_eq!(sender.state, SenderState::Ready);
    }

    #[tokio::test]
    async fn check_send() {
        let message = "test_message";
        let mut out: Vec<&'static str> = vec![];

        let mut sender = QueuedSender::new(&mut out);
        sender.send(message).await.expect("Problem sending message");
        assert_eq!(sender.state, SenderState::Sending);

        assert_eq!(out, vec![message]);
    }

    #[tokio::test]
    async fn check_dont_send_when_sending() {
        let message = "test_message";
        let mut out: Vec<&'static str> = vec![];

        let mut sender = QueuedSender::new(&mut out);
        sender.state = SenderState::Sending;
        sender.send(message).await.expect("Problem sending message");

        assert_eq!(sender.state, SenderState::Sending);
        assert_eq!(sender.queue.get(0), Some(&message));

        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn check_ready() {
        let mut out: Vec<&'static str> = vec![];

        let mut sender = QueuedSender::new(&mut out);
        sender.state = SenderState::Sending;

        sender
            .ready()
            .await
            .expect("Problem setting sender to ready");

        assert_eq!(sender.state, SenderState::Ready);

        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn check_double_ready() {
        let mut out: Vec<&'static str> = vec![];

        let mut sender = QueuedSender::new(&mut out);
        assert_eq!(sender.state, SenderState::Ready);

        assert_eq!(
            sender
                .ready()
                .await
                .expect_err("Sender didn't return an error for double ready"),
            ReadyError::AlreadyReady,
        );

        assert_eq!(sender.state, SenderState::Ready);

        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn check_send_on_ready() {
        let message = "test_message";
        let mut out: Vec<&'static str> = vec![];

        let mut sender = QueuedSender::new(&mut out);
        sender.state = SenderState::Sending;

        sender.send(message).await.expect("Problem sending message");
        assert_eq!(sender.state, SenderState::Sending);
        assert_eq!(sender.queue.len(), 1);

        sender
            .ready()
            .await
            .expect("Problem setting sender to ready");
        assert_eq!(sender.state, SenderState::Sending);
        assert!(sender.queue.is_empty());

        assert_eq!(out, vec![message]);
    }

    #[tokio::test]
    async fn check_order() {
        let num_messages = 2;
        assert!(num_messages > 1);

        let mut out: Vec<i32> = vec![];

        let mut sender = QueuedSender::new(&mut out);
        sender.state = SenderState::Sending;

        for i in 0..num_messages {
            sender.send(i).await.expect("Problem sending message")
        }

        assert_eq!(sender.queue.len(), num_messages as usize);

        for _ in 0..num_messages {
            sender
                .ready()
                .await
                .expect("Problem setting sender to ready");
        }

        for i in 0..num_messages {
            assert_eq!(out.get(i as usize), Some(&i));
        }
    }
}
