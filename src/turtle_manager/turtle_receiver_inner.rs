use futures_util::{stream::SplitStream, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, warn};

use crate::turtle_scheme::TurtleEvents;

use super::{
    turtle_receiver_message::TurtleReceiverMessage, TurtleManagerHandle, TurtleSenderHandle,
};

///
pub struct TurtleReceiverInner {
    rx: mpsc::Receiver<TurtleReceiverMessage>,
    ws_receiver: SplitStream<WebSocketStream<TcpStream>>,
    manager: TurtleManagerHandle,
    sender: TurtleSenderHandle,

    name: &'static str,
}

impl TurtleReceiverInner {
    pub fn new(
        rx: mpsc::Receiver<TurtleReceiverMessage>,
        ws_receiver: SplitStream<WebSocketStream<TcpStream>>,
        manager: TurtleManagerHandle,
        sender: TurtleSenderHandle,
        name: &'static str,
    ) -> Self {
        TurtleReceiverInner {
            rx,
            ws_receiver,
            manager,
            sender,
            name,
        }
    }

    pub async fn run(mut self) {
        let mut close_tx = None;

        loop {
            tokio::select! {
                message = self.ws_receiver.next() => {
                    if let Some(Ok(message)) = message {
                        self.handle_turtle_message(message.to_string()).await;
                    } else {
                        self.manager.disconnect(self.name).await;
                        break;
                    }
                }

                message = self.rx.recv() => {
                    if let Some(message) = message {
                        match message {
                            TurtleReceiverMessage::Close(tx) => {
                                close_tx = Some(tx);
                                break;
                            }
                        }
                    } else {
                        self.manager.disconnect(self.name).await;
                        break;
                    }
                }
            }
        }

        debug!("Turtle Receiver shutting down for {}", self.name);
        if let Some(tx) = close_tx {
            let _ = tx.send(());
        }
    }

    async fn handle_turtle_message(&mut self, message: String) {
        let event: TurtleEvents = match serde_json::from_str(message.as_str()) {
            Ok(e) => e,
            Err(e) => {
                warn!(
                    "Got invalid event from {}. Event: {message} Error: {e}",
                    self.name
                );
                return;
            }
        };
        debug!("Got message {:?} from {}", event, self.name);
        self.handle_turtle_event(event).await;
    }

    async fn handle_turtle_event(&mut self, event: TurtleEvents) {
        match event {
            TurtleEvents::Report {
                position,
                heading,
                fuel,
            } => {
                self.manager
                    .update_turtle_position(self.name, position)
                    .await;
                self.manager.update_turtle_heading(self.name, heading).await;
                self.manager.update_turtle_fuel(self.name, fuel).await;
            }
            TurtleEvents::Inspection { block } => {}
            TurtleEvents::Ok { id } => self.sender.ok(id).await,
            TurtleEvents::Ready => self.sender.ready().await,
        }
    }
}
