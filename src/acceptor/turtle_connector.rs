use crate::acceptor::tcp_handler::TcpHandler;
use crate::turtle_manager::{TurtleManagerHandle, UnknownTurtleConnection};
use tokio::net::TcpStream;

pub struct TurtleConnector {
    turtle_manager: TurtleManagerHandle,
}

impl TurtleConnector {
    pub fn new(turtle_manager: TurtleManagerHandle) -> Self {
        TurtleConnector { turtle_manager }
    }
}

#[async_trait::async_trait]
impl TcpHandler for TurtleConnector {
    async fn handle_tcp(&mut self, stream: TcpStream) {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let unknown_turtle = UnknownTurtleConnection::new(ws_stream);
        self.turtle_manager.new_unknown_turtle(unknown_turtle).await;
    }
}
