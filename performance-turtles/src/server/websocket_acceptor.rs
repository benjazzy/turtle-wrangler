use crate::server::NewStream;
use actix::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tracing::{error, warn};

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewWebsocket<S>(WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Unpin;

impl<S> NewWebsocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn into_inner(self) -> WebSocketStream<S> {
        self.0
    }
}
pub struct WebsocketAcceptor {
    recipient: Recipient<NewWebsocket<TcpStream>>,
}

impl WebsocketAcceptor {
    pub fn new(recipient: Recipient<NewWebsocket<TcpStream>>) -> Self {
        WebsocketAcceptor { recipient }
    }
}

impl Actor for WebsocketAcceptor {
    type Context = Context<Self>;
}

impl Handler<NewStream> for WebsocketAcceptor {
    type Result = ();
    fn handle(&mut self, msg: NewStream, ctx: &mut Self::Context) -> Self::Result {
        let recipient = self.recipient.clone();
        let addr = ctx.address();
        let accept_fut = async move {
            let stream = match tokio_tungstenite::accept_async(msg.into_inner()).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Problem accepting ws stream {e}");
                    return;
                }
            };

            match recipient.try_send(NewWebsocket(stream)) {
                Err(SendError::Closed(mut stream)) => {
                    error!("Websocket recipient closed. Shutting down acceptor");
                    let _ = stream.0.close(None).await;
                    addr.do_send(AcceptCloseMessage);
                }
                Err(SendError::Full(mut stream)) => {
                    warn!("Websocket recipient full");
                    let _ = stream.0.close(None).await;
                }
                Ok(_) => {}
            }
        };
        let fut = fut::wrap_future(accept_fut);

        ctx.spawn(fut);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct AcceptCloseMessage;

impl Handler<AcceptCloseMessage> for WebsocketAcceptor {
    type Result = ();

    fn handle(&mut self, msg: AcceptCloseMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
