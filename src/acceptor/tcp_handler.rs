use tokio::net::TcpStream;

#[async_trait::async_trait]
pub trait TcpHandler {
    async fn handle_tcp(&mut self, stream: TcpStream);
}
