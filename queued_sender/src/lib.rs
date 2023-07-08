pub mod error;

use error::AlreadyReadyError;
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SenderState {
    Ready,
    Sending,
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
            state: SenderState::Ready,
        }
    }

    pub fn get_state(&self) -> &SenderState {
        &self.state
    }

    pub fn send(&mut self, message: T) -> Option<T> {
        match self.state {
            SenderState::Ready => Some(message),
            SenderState::Sending => {
                self.queue.push_back(message);

                None
            }
        }
    }

    pub fn ready(&mut self) -> Result<Option<T>, AlreadyReadyError> {
        match self.state {
            SenderState::Ready => Err(AlreadyReadyError),
            SenderState::Sending => Ok(self.queue.pop_front()),
        }
    }
}

impl<T> Default for QueuedSender<T> {
    fn default() -> Self {
        QueuedSender::new()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn check_start_ready() {
//         let mut out: Vec<&'static str> = vec![];
//
//         let sender = QueuedSender::new(&mut out);
//         assert_eq!(sender.state, SenderState::Ready);
//     }
//
//     #[tokio::test]
//     async fn check_send() {
//         let message = "test_message";
//         let mut out: Vec<&'static str> = vec![];
//
//         let mut sender = QueuedSender::new(&mut out);
//         sender.send(message).await.expect("Problem sending message");
//         assert_eq!(sender.state, SenderState::Sending);
//
//         assert_eq!(out, vec![message]);
//     }
//
//     #[tokio::test]
//     async fn check_dont_send_when_sending() {
//         let message = "test_message";
//         let mut out: Vec<&'static str> = vec![];
//
//         let mut sender = QueuedSender::new(&mut out);
//         sender.state = SenderState::Sending;
//         sender.send(message).await.expect("Problem sending message");
//
//         assert_eq!(sender.state, SenderState::Sending);
//         assert_eq!(sender.queue.get(0), Some(&message));
//
//         assert!(out.is_empty());
//     }
//
//     #[tokio::test]
//     async fn check_ready() {
//         let mut out: Vec<&'static str> = vec![];
//
//         let mut sender = QueuedSender::new(&mut out);
//         sender.state = SenderState::Sending;
//
//         sender
//             .ready()
//             .await
//             .expect("Problem setting sender to ready");
//
//         assert_eq!(sender.state, SenderState::Ready);
//
//         assert!(out.is_empty());
//     }
//
//     #[tokio::test]
//     async fn check_double_ready() {
//         let mut out: Vec<&'static str> = vec![];
//
//         let mut sender = QueuedSender::new(&mut out);
//         assert_eq!(sender.state, SenderState::Ready);
//
//         assert_eq!(
//             sender
//                 .ready()
//                 .await
//                 .expect_err("Sender didn't return an error for double ready"),
//             ReadyError::AlreadyReady,
//         );
//
//         assert_eq!(sender.state, SenderState::Ready);
//
//         assert!(out.is_empty());
//     }
//
//     #[tokio::test]
//     async fn check_send_on_ready() {
//         let message = "test_message";
//         let mut out: Vec<&'static str> = vec![];
//
//         let mut sender = QueuedSender::new(&mut out);
//         sender.state = SenderState::Sending;
//
//         sender.send(message).await.expect("Problem sending message");
//         assert_eq!(sender.state, SenderState::Sending);
//         assert_eq!(sender.queue.len(), 1);
//
//         sender
//             .ready()
//             .await
//             .expect("Problem setting sender to ready");
//         assert_eq!(sender.state, SenderState::Sending);
//         assert!(sender.queue.is_empty());
//
//         assert_eq!(out, vec![message]);
//     }
//
//     #[tokio::test]
//     async fn check_order() {
//         let num_messages = 2;
//         assert!(num_messages > 1);
//
//         let mut out: Vec<i32> = vec![];
//
//         let mut sender = QueuedSender::new(&mut out);
//         sender.state = SenderState::Sending;
//
//         for i in 0..num_messages {
//             sender.send(i).await.expect("Problem sending message")
//         }
//
//         assert_eq!(sender.queue.len(), num_messages as usize);
//
//         for _ in 0..num_messages {
//             sender
//                 .ready()
//                 .await
//                 .expect("Problem setting sender to ready");
//         }
//
//         for i in 0..num_messages {
//             assert_eq!(out.get(i as usize), Some(&i));
//         }
//     }
// }
