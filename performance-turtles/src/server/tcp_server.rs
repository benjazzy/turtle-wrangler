use actix::prelude::*;
use actix_server::{Server, ServerHandle};
use actix_service::{fn_service, ServiceFactoryExt};

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::error;

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewStream(TcpStream);

impl NewStream {
    pub fn into_inner(self) -> TcpStream {
        self.0
    }
}

pub struct TcpServer {
    addr: String,
    recipient: Recipient<NewStream>,
    handle: Option<ServerHandle>,
}

impl TcpServer {
    pub fn new(addr: impl Into<String>, recipient: Recipient<NewStream>) -> Self {
        let handle = None;
        let addr = addr.into();

        TcpServer {
            addr,
            recipient,
            handle,
        }
    }
}

impl Actor for TcpServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = self.recipient.clone();
        let server = Server::build()
            .bind("listen", self.addr.clone(), move || {
                let recipient = recipient.clone();
                fn_service(move |stream: TcpStream| {
                    let recipient = recipient.clone();
                    async move {
                        if let Err(err) = recipient.try_send(NewStream(stream)) {
                            let mut stream = err.into_inner();
                            let _ = stream.0.shutdown().await;
                        };

                        Ok(())
                    }
                })
                .map_err(|_: ()| error!("Tcp Server error"))
            })
            .unwrap()
            .run();

        let handle = server.handle();
        let future = fut::wrap_future::<_, Self>(async {
            let _ = server.await;
        });
        self.handle = Some(handle);

        ctx.spawn(future);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        if let Some(handle) = &self.handle {
            let stop_fut = handle.stop(true);
            let future = fut::wrap_future::<_, Self>(stop_fut);
            ctx.spawn(future);
        } else {
            error!("Stopped called on TcpServer while there is no server handle");
        }
    }
}

pub struct ServerStopMessage;

impl Message for ServerStopMessage {
    type Result = ();
}

impl Handler<ServerStopMessage> for TcpServer {
    type Result = ();

    fn handle(&mut self, _: ServerStopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
