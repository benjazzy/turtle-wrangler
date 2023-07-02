use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{protocol::CloseFrame, Message},
    WebSocketStream,
};
use tracing::{debug, error, info};

use super::turtle_connection::TurtleConnection;

pub struct UnknownTurtleConnection {
    ws_stream: WebSocketStream<TcpStream>,
}

impl UnknownTurtleConnection {
    pub fn new(ws_stream: WebSocketStream<TcpStream>) -> Self {
        UnknownTurtleConnection { ws_stream }
    }

    pub async fn auth(mut self) -> Option<(&'static str, TurtleConnection)> {
        let id = if let Ok(Some(Ok(Message::Text(id)))) =
            tokio::time::timeout(Duration::from_millis(500), self.ws_stream.next()).await
        {
            id
        } else {
            error!("Turtle failed to send id");
            let _ = self.ws_stream.close(None).await;
            return None;
        };

        let id: u64 = if let Ok(id) = id.parse::<f64>() {
            id as u64
        } else {
            error!("Turtle sent invalid id {id}");
            let _ = self.ws_stream.close(None).await;
            return None;
        };

        debug!("Turtle has id {id}");

        let name = Self::get_name(id);
        if let Err(e) = self.ws_stream.send(Message::Text(name.to_string())).await {
            error!("Problem sending turtle its name {e}");
            let _ = self.ws_stream.close(None).await;
            return None;
        }

        info!("{name} connected");

        Some((name, TurtleConnection::new(self.ws_stream, name)))
    }

    fn get_name(id: u64) -> &'static str {
        const NAMESLIST: NamesList = NamesList::new(include_str!("../../first-names.txt"));

        if let Some(n) = NAMESLIST.get(id) {
            return n;
        } else {
            return "Turtle";
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
