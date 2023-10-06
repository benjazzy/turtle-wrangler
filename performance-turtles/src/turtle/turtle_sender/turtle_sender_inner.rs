use actix::prelude::*;
use futures_util::stream::SplitSink;
use std::cell::RefCell;
use std::rc::Rc;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::WebSocketStream;
use tracing::error;

type Sink = SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>;

#[derive(Debug, Clone)]
pub struct TurtleSenderInner {
    ws_sender: Rc<RefCell<Sink>>,
}

impl TurtleSenderInner {
    pub fn new(ws_sender: Sink) -> Self {
        let ws_sender = Rc::new(RefCell::new(ws_sender));

        TurtleSenderInner { ws_sender }
    }
}

impl Actor for TurtleSenderInner {
    type Context = Context<Self>;
}

pub struct SendMessage(String);

impl Message for SendMessage {
    type Result = Result<(), tungstenite::Error>;
}

impl Handler<SendMessage> for TurtleSenderInner {
    type Result = ResponseFuture<Result<(), tungstenite::Error>>;

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        let sender = self.ws_sender.clone();

        let send_fut = async move {
            // if let Ok(mut sender) = sender.try_borrow_mut() {
            //     sender.send(tungstenite::Message::Text(msg.0)).await;
            // } else {
            //     error!("Could not send message because there was already a mut ref");
            // }
            let mut sender = sender.borrow_mut();
            sender.send(tungstenite::Message::Text(msg.0)).await.map(|_| ())
        };

        Box::pin(send_fut)
    }
}
