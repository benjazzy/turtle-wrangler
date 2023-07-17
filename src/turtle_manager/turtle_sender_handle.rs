use std::time::Duration;

use futures_util::stream::SplitSink;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::error;

use crate::{
    scheme::{Coordinates, Heading},
    turtle_scheme::{RequestType, Response, ResponseType, TurtleCommand},
};

use super::{
    turtle_sender_inner::TurtleSenderInner,
    turtle_sender_message::{LockedSenderMessage, ReceiversSenderMessage, TurtleSenderMessage},
    TurtleManagerHandle,
};

pub fn sender(
    ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    manager: TurtleManagerHandle,
    name: &'static str,
) -> (TurtleSenderHandle, ReceiversSenderHandle) {
    let (main_tx, main_rx) = mpsc::channel(1);
    let (receiver_tx, receiver_rx) = mpsc::channel(1);

    let inner = TurtleSenderInner::new(main_rx, receiver_rx, ws_sender, manager, name);
    tokio::spawn(inner.run());

    (
        TurtleSenderHandle { tx: main_tx },
        ReceiversSenderHandle { tx: receiver_tx },
    )
}

#[derive(Debug, Clone)]
pub struct TurtleSenderHandle {
    tx: mpsc::Sender<TurtleSenderMessage>,
}

impl TurtleSenderHandle {
    // pub fn new(
    //     ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    //     manager: TurtleManagerHandle,
    //     name: &'static str,
    // ) -> Self {
    //     let (tx, rx) = mpsc::channel(1);
    //
    //     let inner = TurtleSenderInner::new(rx, ws_sender, manager, name);
    //     tokio::spawn(inner.run());
    //
    //     TurtleSenderHandle { tx }
    // }

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

    pub async fn lock(&self) -> Result<LockedSenderHandle, ()> {
        let (mpsc_tx, mpsc_rx) = mpsc::channel(1);
        let (oneshot_tx, oneshot_rx) = oneshot::channel();

        if self
            .tx
            .send(TurtleSenderMessage::Lock(mpsc_rx, oneshot_tx))
            .await
            .is_err()
        {
            return Err(());
        }

        match oneshot_rx.await {
            Ok(Ok(_)) => Ok(LockedSenderHandle { tx: mpsc_tx }),
            _ => Err(()),
        }
    }
}

pub struct ReceiversSenderHandle {
    tx: mpsc::Sender<ReceiversSenderMessage>,
}

impl ReceiversSenderHandle {
    pub async fn ok(&self, id: u64) {
        if self.tx.send(ReceiversSenderMessage::GotOk(id)).await.is_err() {
            error!("Problem sending got ok");
        }
    }

    pub async fn ready(&self) {
        if self.tx.send(ReceiversSenderMessage::Ready).await.is_err() {
            error!("Problem sending ready");
        }
    }

    pub async fn got_response(&self, response: Response) {
        if self
            .tx
            .send(ReceiversSenderMessage::Response(response))
            .await
            .is_err()
        {
            error!("Problem sending got response");
        }
    }
}

pub struct LockedSenderHandle {
    tx: mpsc::Sender<LockedSenderMessage>,
}

impl LockedSenderHandle {
    pub async fn unlock(&self) {
        if self.tx.send(LockedSenderMessage::Unlock).await.is_err() {
            error!("Couldn't send unlock message to sender");
        }
    }

    pub async fn request(&self, request: RequestType) -> Result<ResponseType, ()> {
        let (tx, rx) = oneshot::channel();

        if self
            .tx
            .send(LockedSenderMessage::Request(request, tx))
            .await
            .is_err()
        {
            error!("Problem sending request to locked sender");
            return Err(());
        }

        if let Ok(r) = rx.await {
            Ok(r)
        } else {
            Err(())
        }
    }

    pub async fn send_position_update(&self, position: Coordinates, heading: Heading) {
        if self
            .tx
            .send(LockedSenderMessage::UpdatePosition(position, heading))
            .await
            .is_err()
        {
            error!("Problem sending update positon to locked sender");
        }
    }
}
