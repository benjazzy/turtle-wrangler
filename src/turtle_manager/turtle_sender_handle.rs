use std::time::Duration;

use futures_util::stream::SplitSink;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::error;

use crate::turtle_scheme::{RequestType, Response, ResponseType, TurtleCommand};

use super::{
    turtle_sender_inner::TurtleSenderInner, turtle_sender_message::TurtleSenderMessage,
    TurtleManagerHandle,
};

#[derive(Debug, Clone)]
pub struct TurtleSenderHandle {
    tx: mpsc::Sender<TurtleSenderMessage>,
}

impl TurtleSenderHandle {
    pub fn new(
        ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
        manager: TurtleManagerHandle,
        name: &'static str,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let inner = TurtleSenderInner::new(rx, ws_sender, manager, name);
        tokio::spawn(inner.run());

        TurtleSenderHandle { tx }
    }

    pub async fn close(&self) {
        let (tx, rx) = oneshot::channel();

        if self.tx.send(TurtleSenderMessage::Close(tx)).await.is_err() {
            error!("Problem sending close message");
            return;
        }

        if tokio::time::timeout(Duration::from_millis(100), rx)
            .await
            .is_err()
        {
            error!("Timeout closing sender");
        }
    }

    pub async fn send(&self, command: TurtleCommand) {
        if let Err(m) = self.tx.send(TurtleSenderMessage::Command(command)).await {
            error!("Problem sending message {m}");
        }
    }

    pub async fn request(&self, request: RequestType) -> Result<ResponseType, ()> {
        let (tx, rx) = oneshot::channel();

        if self
            .tx
            .send(TurtleSenderMessage::Request(request, tx))
            .await
            .is_err()
        {
            error!("Problem sending request");
            return Err(());
        }

        if let Ok(r) = rx.await {
            Ok(r)
        } else {
            Err(())
        }
    }

    pub async fn ok(&self, id: u64) {
        if let Err(_) = self.tx.send(TurtleSenderMessage::GotOk(id)).await {
            error!("Problem sending got ok");
        }
    }

    pub async fn ready(&self) {
        if let Err(_) = self.tx.send(TurtleSenderMessage::Ready).await {
            error!("Problem sending ready");
        }
    }

    pub async fn got_response(&self, response: Response) {
        if self
            .tx
            .send(TurtleSenderMessage::Response(response))
            .await
            .is_err()
        {
            error!("Problem sending got response");
        }
    }
}
