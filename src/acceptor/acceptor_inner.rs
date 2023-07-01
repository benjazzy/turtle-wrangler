use std::{collections::HashMap, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info};

use crate::turtle_manager::{TurtleReceiverHandler, TurtleSenderHandler};

use super::acceptor_message::AcceptorMessage;

pub struct AcceptorInner {
    rx: mpsc::Receiver<AcceptorMessage>,
    senders: Arc<Mutex<Vec<TurtleSenderHandler>>>,
    receivers: Arc<Mutex<Vec<TurtleReceiverHandler>>>,

    names: HashMap<u64, &'static str>,
}

impl AcceptorInner {
    pub fn new(
        rx: mpsc::Receiver<AcceptorMessage>,
        senders: Arc<Mutex<Vec<TurtleSenderHandler>>>,
        receivers: Arc<Mutex<Vec<TurtleReceiverHandler>>>,
    ) -> Self {
        AcceptorInner {
            rx,
            senders,
            receivers,

            names: HashMap::new(),
        }
    }

    pub async fn run(mut self, addr: String) {
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        info!("Listening for connections at {addr}");
        loop {
            tokio::select! {
                message = self.rx.recv() => {
                    if let Some(message) = message {
                        info!("Got message {:?}", message);
                    } else {
                        break;
                    }
                }

                connection = listener.accept() => {
                    if let Ok((stream, _)) = connection {
                        self.accept(stream).await;
                    } else {
                        break;
                    }
                }
            }
        }

        info!("Closing listener");
    }

    async fn accept(&mut self, stream: TcpStream) {
        debug!(
            "Got connection {}",
            stream
                .peer_addr()
                .expect("Could not get peer address from stream")
        );
        let mut ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let id = if let Ok(Some(Ok(Message::Text(id)))) =
            tokio::time::timeout(Duration::from_millis(500), ws_stream.next()).await
        {
            id
        } else {
            error!("Turtle failed to send id");
            return;
        };

        let id: u64 = if let Ok(id) = id.parse::<f64>() {
            id as u64
        } else {
            error!("Turtle sent invalid id {id}");
            return;
        };

        debug!("Turtle has id {id}");

        let name = self.get_name(id);
        if let Err(e) = ws_stream.send(Message::Text(name.to_string())).await {
            error!("Problem sending turtle its name {e}");
            return;
        }

        info!("{name} connected");

        let (write, read) = ws_stream.split();

        let sender = TurtleSenderHandler::new(write, name);
        self.senders.lock().await.push(sender);

        let receiver = TurtleReceiverHandler::new(read, name);
        self.receivers.lock().await.push(receiver);
    }

    // TODO Change this to not be so weird.
    fn get_name(&mut self, id: u64) -> &'static str {
        if let Some(name) = self.names.get(&id) {
            return *name;
        } else {
            const NAMESLIST: NamesList = NamesList::new(include_str!("../../first-names.txt"));
            let name = NAMESLIST.get(id).unwrap_or("Turtle");
            self.names.insert(id, name);

            return name;
        };
    }
}

struct NamesList(&'static str);

impl NamesList {
    pub const fn new(names: &'static str) -> Self {
        NamesList(names)
    }

    pub fn len(&self) -> usize {
        self.0.split_whitespace().count()
    }

    pub fn get(&self, id: u64) -> Option<&'static str> {
        let name = if let Some(n) = self.0.split_whitespace().nth(id as usize) {
            n
        } else {
            return None;
        };

        Some(name)
    }
}
// const fn get_names() -> &'static Vec<&'static str> {
//
//     const list: &'static str = include_str!("../../first-names.txt");
//     let names: Vec<&'static str> = list.split_whitespace().collect();
// }
