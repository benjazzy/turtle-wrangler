use sqlx::SqlitePool;
use tokio::net::TcpStream;
use crate::acceptor::tcp_handler::TcpHandler;
use crate::client_manager::{ClientConnectionHandle, ClientManagerHandle};
use crate::turtle_manager::TurtleManagerHandle;

pub struct ClientConnector {
    client_manager: ClientManagerHandle,
    turtle_manager: TurtleManagerHandle,
    pool: SqlitePool,
    next_id: usize,
}

impl ClientConnector {
    pub fn new(client_manager: ClientManagerHandle, turtle_manager: TurtleManagerHandle, pool: SqlitePool) -> Self {
        ClientConnector { client_manager, turtle_manager, pool, next_id: 0 }
    }
}

#[async_trait::async_trait]
impl TcpHandler for ClientConnector {
    async fn handle_tcp(&mut self, stream: TcpStream) {
        let id = self.next_id;
        self.next_id += 1;
        
        let client  = ClientConnectionHandle::new(stream, self.turtle_manager.clone(), self.pool.clone(), id);
        self.client_manager.new_client(client).await;
    }
}