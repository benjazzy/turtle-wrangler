use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

pub struct UnknownTurtleConnection {
    ws_stream: WebSocketStream<TcpStream>,
}

pub struct TurtleConnection {}
