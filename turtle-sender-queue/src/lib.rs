use std::collections::VecDeque;

/// Used to store the state of the queue.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum QueueState {
    /// Indicates that the queue is ready to send a message.
    Ready,

    /// Indicates that the queue is waiting for a ready message.
    Waiting,
}

/// Manages a queue of messages only yeilding a message after ready() is called.
#[derive(Debug)]
pub struct SenderQueue<T> {
    /// Contains the queue of messages to be sent.
    queue: VecDeque<T>,

    /// State of the SenderQueue.
    state: QueueState,
}

impl<T> SenderQueue<T> {
    /// Always start with a empty queue and a Waiting state.
    /// This means that ready must be called before any message is sent.
    pub fn new() -> Self {
        SenderQueue {
            queue: VecDeque::new(),
            state: QueueState::Waiting,
        }
    }

    /// Gets the state of the queue.
    pub fn get_state(&self) -> &QueueState {
        &self.state
    }

    /// If the sender is ready to send a message then return the next message in queue.
    /// If the sender is waiting then add the message to queue and return None.
    pub fn send(&mut self, message: T) -> Option<T> {
        self.queue.push_back(message);

        match self.state {
            QueueState::Ready => self.pop_send(),
            QueueState::Waiting => None,
        }
    }

    /// Sets the state to ready and returns the next message in queue if there is one.
    pub fn ready(&mut self) -> Option<T> {
        self.state = QueueState::Ready;

        self.pop_send()
    }

    /// Used internally by ready() and send().
    /// If there is a message in queue then set state to waiting and return the message.
    /// Otherwise return None.
    fn pop_send(&mut self) -> Option<T> {
        let message = self.queue.pop_front();
        if message.is_some() {
            self.state = QueueState::Waiting;
        }

        message
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, QueueState::Ready)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl<T> Default for SenderQueue<T> {
    fn default() -> Self {
        SenderQueue::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that SenderQueue starts out waiting for a ready.
    #[test]
    fn check_start_waiting() {
        let queue = SenderQueue::<&str>::new();
        assert_eq!(queue.state, QueueState::Waiting);
    }

    // Check that if the queue is waiting then calling send will add the message to the queue.
    #[test]
    fn check_send_waiting() {
        let message = "test_message";

        let mut queue = SenderQueue::new();
        assert_eq!(queue.send(message), None);
        assert_eq!(queue.queue.to_owned(), vec![message]);
    }

    // Check that if the queue is ready then calling send will return the message.
    #[test]
    fn check_send_ready() {
        let message = "test_message";

        let mut queue = SenderQueue::new();
        queue.state = QueueState::Ready;
        assert_eq!(queue.send(message), Some(message));
        assert_eq!(queue.state, QueueState::Waiting);
    }

    // Check that a ready call with an empty queue returns none.
    #[test]
    fn check_empty_ready() {
        let mut queue = SenderQueue::<&str>::new();
        assert_eq!(queue.ready(), None);
        assert_eq!(queue.state, QueueState::Ready);
    }

    // Checks that a ready call with something in queue returns the next item in queue.
    #[test]
    fn check_some_ready() {
        let message = "test_message";

        let mut queue = SenderQueue::new();
        queue.queue.push_back(message);
        assert_eq!(queue.ready(), Some(message));
        assert_eq!(queue.state, QueueState::Waiting);
        assert!(queue.queue.is_empty());
    }

    // Checks that a ready call with an empty queue and a ready state returns None.
    #[test]
    fn check_empty_double_ready() {
        let mut queue = SenderQueue::<&str>::new();
        queue.state = QueueState::Ready;
        assert_eq!(queue.ready(), None);
        assert_eq!(queue.state, QueueState::Ready);
        assert!(queue.queue.is_empty());
    }
}
