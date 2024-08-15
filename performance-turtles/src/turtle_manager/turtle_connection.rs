use actix::prelude::*;
use tokio::net::TcpListener;
use tracing::info;

pub struct TurtleConnection {
    listener: TcpListener,
}

impl TurtleConnection {
    pub async fn new(addr: &str) -> Self {
        let try_socket = TcpListener::bind(addr).await;
        let listener = try_socket.expect("Failed to bind to {addr}");

        info!("Listening for connections at {addr}");

        TurtleConnection { listener }
    }
}

impl Actor for TurtleConnection {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        todo!()
    }
}
