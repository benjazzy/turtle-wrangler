pub mod error;

use error::AlreadyReadyError;
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SenderState {
    Ready,
    Waiting,
}

#[derive(Debug)]
pub struct QueuedSender<T> {
    queue: VecDeque<T>,
    state: SenderState,
}

impl<T> QueuedSender<T> {
    pub fn new() -> Self {
        QueuedSender {
            queue: VecDeque::new(),
            state: SenderState::Waiting,
        }
    }

    pub fn get_state(&self) -> &SenderState {
        &self.state
    }

    pub fn send(&mut self, message: T) -> Option<T> {
        match self.state {
            SenderState::Ready => {
                self.queue.push_back(message);

                self.pop_send()
            }
            SenderState::Waiting => {
                self.queue.push_back(message);

                None
            }
        }
    }

    pub fn ready(&mut self) -> Option<T> {
        match self.state {
            SenderState::Ready => {}
            SenderState::Waiting => {
                self.state = SenderState::Ready;
            }
        }

        self.pop_send()
    }

    fn pop_send(&mut self) -> Option<T> {
        let message = self.queue.pop_front();
        if message.is_some() {
            self.state = SenderState::Waiting;
        }

        message
    }
}

impl<T> Default for QueuedSender<T> {
    fn default() -> Self {
        QueuedSender::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_start_waiting() {
        let queue = QueuedSender::<&str>::new();
        assert_eq!(queue.state, SenderState::Waiting);
    }

    #[test]
    fn check_send_waiting() {
        let message = "test_message";

        let mut queue = QueuedSender::new();
        assert_eq!(queue.send(message), None);
        assert_eq!(queue.queue.to_owned(), vec![message]);
    }

    #[test]
    fn check_send_ready() {
        let message = "test_message";

        let mut queue = QueuedSender::new();
        queue.state = SenderState::Ready;
        assert_eq!(queue.send(message), Some(message));
        assert_eq!(queue.state, SenderState::Waiting);
    }

    #[test]
    fn check_empty_ready() {
        let mut queue = QueuedSender::<&str>::new();
        assert_eq!(queue.ready(), None);
        assert_eq!(queue.state, SenderState::Ready);
    }

    #[test]
    fn check_some_ready() {
        let message = "test_message";

        let mut queue = QueuedSender::new();
        queue.queue.push_back(message);
        assert_eq!(queue.ready(), Some(message));
        assert!(queue.queue.is_empty());
    }
}
