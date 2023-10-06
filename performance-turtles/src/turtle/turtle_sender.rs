mod turtle_sender_inner;

use actix::prelude::*;
use futures_util::stream::SplitSink;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

type Sink = SplitSink<WebSocketStream<TcpStream>, Message>;

enum SenderState {
    Normal(TurtleSenderInner),
    Locked(Vec<String>),
    Transitioning,
}

impl SenderState {
    pub fn new(sender: TurtleSenderInner) -> Self {
        SenderState::Normal(sender)
    }

    pub fn lock(&mut self) -> Result<TurtleSenderInner, ()> {
        let state = std::mem::replace(self, SenderState::Transitioning);
        match state {
            SenderState::Normal(sender) => {
                *self = SenderState::Locked(Vec::new());

                Ok(sender)
            }
            SenderState::Locked(queue) => {
                *self = SenderState::Locked(queue);

                Err(())
            }
            SenderState::Transitioning => {
                panic!("Sender was in invalid state Transitioning when lock was called")
            }
        }
    }

    pub fn unlock(&mut self, inner: TurtleSenderInner) -> Result<Vec<String>, ()> {
        let state = std::mem::replace(self, SenderState::Transitioning);
        match state {
            SenderState::Normal(sender) => {
                *self = SenderState::Normal(sender);

                Err(())
            }
            SenderState::Locked(queue) => {
                *self = SenderState::Normal(inner);

                Ok(queue)
            }
            SenderState::Transitioning => {
                panic!("Sender was in invalid state Transitioning when lock was called")
            }
        }
    }

    pub async fn send(&mut self, message: String) {
        match self {
            SenderState::Normal(sender) => sender.send(message).await,
            SenderState::Locked(queue) => queue.push(message),
            SenderState::Transitioning => {
                panic!("Send was in invalid state Transition when send was called")
            }
        }
    }
}

struct TurtleSenderInner {
    ws_sink: Sink,
}

impl TurtleSenderInner {
    pub fn new(ws_sink: Sink) -> Self {
        TurtleSenderInner { ws_sink }
    }

    pub async fn send(&mut self, message: String) {
        todo!()
    }
}

pub struct LockedTurtleSender {
    inner: TurtleSenderInner,
}

impl Actor for LockedTurtleSender {
    type Context = Context<Self>;
}

impl LockedTurtleSender {
    fn new(inner: TurtleSenderInner) -> Self {
        LockedTurtleSender { inner }
    }
}

pub struct TurtleSender {
    sender: Rc<RefCell<SenderState>>,
}

impl TurtleSender {
    pub fn new(ws_sink: Sink) -> Self {
        let sender = Rc::new(RefCell::new(SenderState::new(TurtleSenderInner::new(
            ws_sink,
        ))));
        TurtleSender { sender }
    }
}

impl Actor for TurtleSender {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(String);

impl Handler<SendMessage> for TurtleSender {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        let sender = self.sender.clone();

        let send_fut = async move {
            let mut sender = sender.borrow_mut();
            sender.send(msg.0).await;
        };

        Box::pin(send_fut)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Addr<LockedTurtleSender>, ()>")]
pub struct LockSender;

impl Handler<LockSender> for TurtleSender {
    type Result = Result<Addr<LockedTurtleSender>, ()>;

    fn handle(&mut self, _msg: LockSender, _ctx: &mut Self::Context) -> Self::Result {
        let mut sender = self.sender.borrow_mut();
        let result = sender.lock();

        result.map(|sender| LockedTurtleSender::new(sender).start())
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct UnlockSender(TurtleSenderInner);

impl Handler<UnlockSender> for TurtleSender {
    type Result = ();

    fn handle(&mut self, msg: UnlockSender, ctx: &mut Self::Context) -> Self::Result {
        let mut sender = self.sender.borrow_mut();
        let result = sender.unlock(msg.0);

        let queue = match result {
            Ok(queue) => queue,
            Err(_) => return,
        };

        for message in queue {
            ctx.address().do_send(SendMessage(message));
        }
    }
}
