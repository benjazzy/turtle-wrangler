use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

type Sink = SplitSink<WebSocketStream<TcpStream>, Message>;
pub enum SinkState {
    Unlocked()
}