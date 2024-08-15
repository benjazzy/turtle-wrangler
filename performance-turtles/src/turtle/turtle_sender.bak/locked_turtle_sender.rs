use crate::turtle::turtle_sender::turtle_sender_inner::TurtleSenderInner;
use actix::prelude::*;

use super::turtle_sender_container::DoUnlock;

pub struct LockedTurtleSender {
    inner: Addr<TurtleSenderInner>,
    unlock_sender: Option<Recipient<DoUnlock>>,
}

impl LockedTurtleSender {
    pub fn new(inner: Addr<TurtleSenderInner>, unlock_sender: Recipient<DoUnlock>) -> Self {
        LockedTurtleSender {
            inner,
            unlock_sender: Some(unlock_sender),
        }
    }
}

impl Drop for LockedTurtleSender {
    fn drop(&mut self) {
        let inner = self.inner.clone();
        let sender = std::mem::replace(&mut self.unlock_sender, None);
        sender
            .expect("Unlock sender was none when drop was called on LockedTurtleSender")
            .do_send(DoUnlock(inner));
    }
}
